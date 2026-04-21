use std::process::{Command, Stdio, Child};
use std::io::{Write, BufRead, BufReader};
use std::sync::Mutex;
use serde_json::{json, Value};
use crate::mcp::{McpServerProxy, McpTool};

pub struct StdioMcpServer {
    name: String,
    child: Mutex<Child>,
    cached_tools: std::sync::RwLock<Vec<McpTool>>,
}

impl StdioMcpServer {
    pub fn spawn(name: String, command: String, args: Vec<String>, env: std::collections::HashMap<String, String>) -> Result<Self, String> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());
        for (k, v) in env {
            cmd.env(k, v);
        }

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn MCP server {}: {}", name, e))?;
        
        let server = Self {
            name,
            child: Mutex::new(child),
            cached_tools: std::sync::RwLock::new(Vec::new()),
        };

        // Handshake: initialize
        let _ = server.send_request("initialize", json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "brazen", "version": "0.1.0" }
        }))?;

        // Initial tool fetch
        server.refresh_tools();

        Ok(server)
    }

    pub fn refresh_tools(&self) {
        if let Ok(val) = self.send_request("tools/list", json!({})) {
            if let Some(tools_array) = val.get("tools").and_then(|t| t.as_array()) {
                let tools = tools_array.iter().map(|t| McpTool {
                    name: t.get("name").and_then(|n| n.as_str()).unwrap_or_default().to_string(),
                    description: t.get("description").and_then(|d| d.as_str()).unwrap_or_default().to_string(),
                    input_schema: t.get("inputSchema").cloned().unwrap_or(json!({})),
                }).collect();
                if let Ok(mut cache) = self.cached_tools.write() {
                    *cache = tools;
                }
            }
        }
    }

    fn send_request(&self, method: &str, params: Value) -> Result<Value, String> {
        let mut child = self.child.lock().map_err(|_| "Mutex poisoned")?;
        
        let stdin = child.stdin.as_mut().ok_or("No stdin")?;
        let request = json!({
            "jsonrpc": "2.0",
            "id": 0,
            "method": method,
            "params": params
        });
        let mut line = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        line.push('\n');
        stdin.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;

        let stdout = child.stdout.as_mut().ok_or("No stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).map_err(|e| e.to_string())?;
        
        let response: Value = serde_json::from_str(&response_line).map_err(|e| format!("Invalid JSON from server: {}. Raw: {}", e, response_line))?;
        if let Some(error) = response.get("error") {
            return Err(format!("MCP Error: {:?}", error));
        }
        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }
}

impl McpServerProxy for StdioMcpServer {
    fn name(&self) -> &str {
        &self.name
    }

    fn list_tools(&self) -> Vec<McpTool> {
        self.cached_tools.read().map(|c| c.clone()).unwrap_or_default()
    }

    fn call_tool(&self, name: &str, args: Value) -> Result<Value, String> {
        match self.send_request("tools/call", json!({
            "name": name,
            "arguments": args
        })) {
            Ok(val) => Ok(val.get("content").cloned().unwrap_or(json!({"error": "No content in response"}))),
            Err(e) => Err(e),
        }
    }
}

impl Drop for StdioMcpServer {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}

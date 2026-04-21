use std::sync::{Arc, RwLock, OnceLock};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub tools: Vec<McpTool>,
}

pub trait McpServerProxy: Send + Sync {
    fn name(&self) -> &str;
    fn list_tools(&self) -> Vec<McpTool>;
    fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, String>;
}

struct McpRegistry {
    servers: Vec<Box<dyn McpServerProxy>>,
}

static REGISTRY: OnceLock<Arc<RwLock<McpRegistry>>> = OnceLock::new();

fn get_registry() -> Arc<RwLock<McpRegistry>> {
    REGISTRY.get_or_init(|| {
        let registry = McpRegistry {
            servers: vec![Box::new(MockServerProxy)],
        };
        Arc::new(RwLock::new(registry))
    }).clone()
}

pub struct McpBroker;

impl McpBroker {
    pub fn list_tools() -> Vec<McpTool> {
        let registry = get_registry();
        let registry = registry.read().unwrap();
        registry.servers.iter()
            .flat_map(|s| s.list_tools())
            .collect()
    }

    pub fn call_tool(name: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
        let registry = get_registry();
        let registry = registry.read().unwrap();
        for server in &registry.servers {
            let tools = server.list_tools();
            if tools.iter().any(|t| t.name == name) {
                return server.call_tool(name, args);
            }
        }
        Err(format!("Tool not found: {}", name))
    }

    pub fn register_server(server: Box<dyn McpServerProxy>) {
        let registry = get_registry();
        let mut registry = registry.write().unwrap();
        registry.servers.push(server);
    }
}

struct MockServerProxy;

impl McpServerProxy for MockServerProxy {
    fn name(&self) -> &str {
        "internal"
    }

    fn list_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: "google_search".to_string(),
                description: "Search the web using Google".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    }
                }),
            },
            McpTool {
                name: "read_file".to_string(),
                description: "Read a file from the local filesystem".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    }
                }),
            }
        ]
    }

    fn call_tool(&self, name: &str, _args: serde_json::Value) -> Result<serde_json::Value, String> {
        match name {
            "google_search" => Ok(serde_json::json!({"results": ["Brazen Browser Architecture", "Servo Engine Embedding"]})),
            "read_file" => Ok(serde_json::json!({"content": "Brazen Virtual Resource Protocol v1.0"})),
            _ => Err(format!("Tool not found: {}", name)),
        }
    }
}

use std::process::Stdio;
use tokio::process::Command;
use serde::{Serialize, Deserialize};
use tokio::time::{timeout, Duration};

use crate::config::TerminalConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminalRequest {
    pub cmd: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub truncated_stdout: bool,
    pub truncated_stderr: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub enum TerminalLine {
    Stdout(String),
    Stderr(String),
    Status(String),
    Done(bool),
}

/// A simple broker for executing terminal commands.
pub struct TerminalBroker;

impl TerminalBroker {
    pub async fn execute(config: &TerminalConfig, request: TerminalRequest) -> TerminalResponse {
        let started = std::time::Instant::now();
        if !config.allowlist.is_empty() {
            let allowed = config
                .allowlist
                .iter()
                .any(|allowed| allowed == &request.cmd);
            if !allowed {
                return TerminalResponse {
                    success: false,
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: None,
                    error: Some("command not allowlisted".to_string()),
                    truncated_stdout: false,
                    truncated_stderr: false,
                    duration_ms: started.elapsed().as_millis() as u64,
                };
            }
        }
        if request.args.len() > config.max_args {
            return TerminalResponse {
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                error: Some("too many args".to_string()),
                truncated_stdout: false,
                truncated_stderr: false,
                duration_ms: started.elapsed().as_millis() as u64,
            };
        }

        let mut command = Command::new(&request.cmd);
        command.args(&request.args);
        
        if let Some(cwd) = &request.cwd {
            command.current_dir(cwd);
        }
        
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        match command.spawn() {
            Ok(child) => {
                match timeout(Duration::from_millis(config.timeout_ms), child.wait_with_output()).await {
                    Ok(waited) => match waited {
                    Ok(output) => TerminalResponse {
                        success: output.status.success(),
                        stdout: {
                            let slice = if output.stdout.len() > config.max_stdout_bytes {
                                &output.stdout[..config.max_stdout_bytes]
                            } else {
                                &output.stdout
                            };
                            String::from_utf8_lossy(slice).to_string()
                        },
                        stderr: {
                            let slice = if output.stderr.len() > config.max_stderr_bytes {
                                &output.stderr[..config.max_stderr_bytes]
                            } else {
                                &output.stderr
                            };
                            String::from_utf8_lossy(slice).to_string()
                        },
                        exit_code: output.status.code(),
                        error: None,
                        truncated_stdout: output.stdout.len() > config.max_stdout_bytes,
                        truncated_stderr: output.stderr.len() > config.max_stderr_bytes,
                        duration_ms: started.elapsed().as_millis() as u64,
                    },
                    Err(e) => TerminalResponse {
                        success: false,
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: None,
                        error: Some(format!("Failed to wait for process: {}", e)),
                        truncated_stdout: false,
                        truncated_stderr: false,
                        duration_ms: started.elapsed().as_millis() as u64,
                    },
                    },
                    Err(_) => TerminalResponse {
                        success: false,
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: None,
                        error: Some("timeout".to_string()),
                        truncated_stdout: false,
                        truncated_stderr: false,
                        duration_ms: started.elapsed().as_millis() as u64,
                    },
                }
            }
            Err(e) => TerminalResponse {
                success: false,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                error: Some(format!("Failed to spawn process: {}", e)),
                truncated_stdout: false,
                truncated_stderr: false,
                duration_ms: started.elapsed().as_millis() as u64,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn terminal_enforces_allowlist() {
        let config = TerminalConfig {
            allowlist: vec!["echo".to_string()],
            ..TerminalConfig::default()
        };
        let res = TerminalBroker::execute(
            &config,
            TerminalRequest {
                cmd: "sh".to_string(),
                args: vec!["-c".to_string(), "echo hi".to_string()],
                cwd: None,
            },
        )
        .await;
        assert!(!res.success);
        assert!(res.error.unwrap_or_default().contains("allowlisted"));
    }

    #[tokio::test]
    async fn terminal_truncates_stdout() {
        let config = TerminalConfig {
            allowlist: vec!["echo".to_string()],
            max_stdout_bytes: 4,
            ..TerminalConfig::default()
        };
        let res = TerminalBroker::execute(
            &config,
            TerminalRequest {
                cmd: "echo".to_string(),
                args: vec!["hello".to_string()],
                cwd: None,
            },
        )
        .await;
        assert!(res.success);
        assert!(res.truncated_stdout);
        assert!(res.stdout.len() <= 4);
    }
}

use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::io::Write;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub user_agent: Option<String>,
    pub client_ip: Option<String>,
    pub outcome: String,
}

pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    pub fn new(log_path: PathBuf) -> Self {
        Self { log_path }
    }

    pub fn log(&self, entry: AuditEntry) -> Result<(), String> {
        tracing::info!(target: "brazen::audit", path = %self.log_path.display(), "writing audit entry");
        if let Some(parent) = self.log_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| e.to_string())?;
        
        writeln!(file, "{}", json).map_err(|e| e.to_string())?;
        Ok(())
    }
}

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use url::Url;

use crate::permissions::PermissionPolicy;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BrazenConfig {
    pub app: AppConfig,
    pub window: WindowConfig,
    pub logging: LoggingConfig,
    pub directories: DirectoryRootsConfig,
    pub cache: CacheConfig,
    pub permissions: PermissionPolicy,
    pub automation: AutomationConfig,
    pub extraction: ExtractionConfig,
    pub media: MediaConfig,
    pub features: FeatureFlags,
}

impl BrazenConfig {
    pub fn load_with_defaults(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| ConfigError::CreateConfigDir {
                    path: parent.display().to_string(),
                    source,
                })?;
            }
            write_default_config(path)?;
        }

        let raw = fs::read_to_string(path).map_err(|source| ConfigError::ReadConfig {
            path: path.display().to_string(),
            source,
        })?;
        let merged = merge_defaults(&raw)?;
        let config: BrazenConfig = merged.try_into().map_err(ConfigError::Deserialize)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.window.initial_width < 640.0 {
            return Err(ConfigError::Validation(
                "window.initial_width must be at least 640".to_string(),
            ));
        }
        if self.window.initial_height < 480.0 {
            return Err(ConfigError::Validation(
                "window.initial_height must be at least 480".to_string(),
            ));
        }
        if self.cache.max_entry_bytes == 0 {
            return Err(ConfigError::Validation(
                "cache.max_entry_bytes must be greater than zero".to_string(),
            ));
        }
        if self.automation.enabled {
            let url = Url::parse(&self.automation.bind).map_err(|error| {
                ConfigError::Validation(format!("automation.bind must be a valid URL: {error}"))
            })?;
            if url.scheme() != "ws" && url.scheme() != "wss" {
                return Err(ConfigError::Validation(
                    "automation.bind must use ws or wss".to_string(),
                ));
            }
        }

        Ok(())
    }
}

pub fn write_default_config(path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ConfigError::CreateConfigDir {
            path: parent.display().to_string(),
            source,
        })?;
    }
    fs::write(path, default_config_toml()).map_err(|source| ConfigError::WriteConfig {
        path: path.display().to_string(),
        source,
    })
}

fn merge_defaults(raw: &str) -> Result<toml::Value, ConfigError> {
    let mut base =
        toml::Value::try_from(BrazenConfig::default()).map_err(ConfigError::Serialize)?;
    let overlay = toml::from_str::<toml::Value>(raw).map_err(ConfigError::Parse)?;
    merge_value(&mut base, overlay);
    Ok(base)
}

fn merge_value(base: &mut toml::Value, overlay: toml::Value) {
    match (base, overlay) {
        (toml::Value::Table(base_table), toml::Value::Table(overlay_table)) => {
            for (key, value) in overlay_table {
                match base_table.get_mut(&key) {
                    Some(existing) => merge_value(existing, value),
                    None => {
                        base_table.insert(key, value);
                    }
                }
            }
        }
        (base_slot, overlay_value) => *base_slot = overlay_value,
    }
}

pub fn default_config_toml() -> &'static str {
    include_str!("../config/brazen.toml")
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to create config directory {path}: {source}")]
    CreateConfigDir {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read config {path}: {source}")]
    ReadConfig {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write config {path}: {source}")]
    WriteConfig {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize default config: {0}")]
    Serialize(#[source] toml::ser::Error),
    #[error("failed to deserialize merged config: {0}")]
    Deserialize(#[source] toml::de::Error),
    #[error("invalid config: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub name: String,
    pub tagline: String,
    pub homepage: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Brazen".to_string(),
            tagline: "Capability Browser Platform".to_string(),
            homepage: "https://example.invalid/brazen".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub initial_width: f32,
    pub initial_height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub show_log_panel_on_startup: bool,
    pub show_permission_panel_on_startup: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            initial_width: 1440.0,
            initial_height: 920.0,
            min_width: 960.0,
            min_height: 640.0,
            show_log_panel_on_startup: true,
            show_permission_panel_on_startup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub console_filter: String,
    pub file_filter: String,
    pub file_name_prefix: String,
    pub ansi: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            console_filter: "info,brazen=debug".to_string(),
            file_filter: "debug,brazen=trace".to_string(),
            file_name_prefix: "brazen.log".to_string(),
            ansi: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DirectoryConfig {
    #[default]
    Default,
    Path(PathBuf),
}

impl Serialize for DirectoryConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Default => serializer.serialize_str("default"),
            Self::Path(path) => serializer.serialize_str(&path.display().to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for DirectoryConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == "default" {
            Ok(Self::Default)
        } else {
            Ok(Self::Path(PathBuf::from(value)))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DirectoryRootsConfig {
    pub data_dir: DirectoryConfig,
    pub logs_dir: DirectoryConfig,
    pub profiles_dir: DirectoryConfig,
    pub cache_dir: DirectoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    pub metadata_capture: bool,
    pub selective_body_capture: bool,
    pub archive_replay_mode: bool,
    pub max_entry_bytes: u64,
    pub third_party_mode: String,
    pub mime_allowlist: Vec<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            metadata_capture: true,
            selective_body_capture: true,
            archive_replay_mode: false,
            max_entry_bytes: 10 * 1024 * 1024,
            third_party_mode: "metadata-only".to_string(),
            mime_allowlist: vec![
                "text/html".to_string(),
                "application/json".to_string(),
                "text/css".to_string(),
                "application/javascript".to_string(),
                "image/png".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutomationConfig {
    pub enabled: bool,
    pub bind: String,
    pub expose_tab_api: bool,
    pub expose_cache_api: bool,
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: "ws://127.0.0.1:7942".to_string(),
            expose_tab_api: true,
            expose_cache_api: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtractionConfig {
    pub article_processing_enabled: bool,
    pub ontology_capture_enabled: bool,
    pub rss_rehydration_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaConfig {
    pub default_tts_provider: String,
    pub auto_queue_reader_mode: bool,
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            default_tts_provider: "none".to_string(),
            auto_queue_reader_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FeatureFlags {
    pub shell_status_panel: bool,
    pub cache_inspector: bool,
    pub automation_server: bool,
    pub servo_backend: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            shell_status_panel: true,
            cache_inspector: true,
            automation_server: false,
            servo_backend: cfg!(feature = "servo"),
        }
    }
}

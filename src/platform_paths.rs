use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use thiserror::Error;

use crate::config::{BrazenConfig, DirectoryConfig};

#[derive(Debug, Clone)]
pub struct PlatformPaths {
    config_root: PathBuf,
    data_root: PathBuf,
    cache_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RuntimePaths {
    pub config_path: PathBuf,
    pub data_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub profiles_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub downloads_dir: PathBuf,
    pub crash_dumps_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum PathsError {
    #[error("failed to derive project directories for brazen")]
    MissingProjectDirs,
}

impl PlatformPaths {
    pub fn detect() -> Result<Self, PathsError> {
        if let Some(project_dirs) = ProjectDirs::from("io", "brazen", "brazen") {
            Ok(Self {
                config_root: project_dirs.config_dir().to_path_buf(),
                data_root: project_dirs.data_dir().to_path_buf(),
                cache_root: project_dirs.cache_dir().to_path_buf(),
            })
        } else {
            let fallback = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(".brazen");
            Ok(Self {
                config_root: fallback.join("config"),
                data_root: fallback.join("data"),
                cache_root: fallback.join("cache"),
            })
        }
    }

    pub fn from_roots(
        config_root: impl Into<PathBuf>,
        data_root: impl Into<PathBuf>,
        cache_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            config_root: config_root.into(),
            data_root: data_root.into(),
            cache_root: cache_root.into(),
        }
    }

    pub fn default_config_path(&self) -> PathBuf {
        self.config_root.join("brazen.toml")
    }

    pub fn resolve_runtime_paths(
        &self,
        config: &BrazenConfig,
        config_path: &Path,
    ) -> Result<RuntimePaths, PathsError> {
        let base_dir = config_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.config_root.clone());
        let dirs = &config.directories;

        let data_dir = self.resolve_dir(&base_dir, &self.data_root, &dirs.data_dir);
        let logs_dir = self.resolve_dir(&base_dir, &data_dir, &dirs.logs_dir);
        let profiles_dir = self.resolve_dir(&base_dir, &data_dir, &dirs.profiles_dir);
        let cache_dir = self.resolve_dir(&base_dir, &self.cache_root, &dirs.cache_dir);
        let downloads_dir = self.resolve_dir(&base_dir, &data_dir, &dirs.downloads_dir);
        let crash_dumps_dir = self.resolve_dir(&base_dir, &data_dir, &dirs.crash_dumps_dir);

        Ok(RuntimePaths {
            config_path: config_path.to_path_buf(),
            data_dir,
            logs_dir,
            profiles_dir,
            cache_dir,
            downloads_dir,
            crash_dumps_dir,
        })
    }

    fn resolve_dir(
        &self,
        base_dir: &Path,
        default_root: &Path,
        value: &DirectoryConfig,
    ) -> PathBuf {
        match value {
            DirectoryConfig::Default => default_root.to_path_buf(),
            DirectoryConfig::Path(path) if path.is_absolute() => path.clone(),
            DirectoryConfig::Path(path) => base_dir.join(path),
        }
    }
}

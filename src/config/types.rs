use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CudaMgrConfig {
    pub install_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_level: LogLevel,
    pub auto_cleanup: bool,
    pub verify_downloads: bool,
    pub parallel_downloads: bool,
    pub max_concurrent_downloads: usize,
    pub default_cuda_version: Option<String>,
    pub proxy_settings: Option<ProxyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for CudaMgrConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let cudamgr_dir = home_dir.join(".cudamgr");

        Self {
            install_dir: cudamgr_dir.join("versions"),
            cache_dir: cudamgr_dir.join("cache"),
            log_level: LogLevel::Info,
            auto_cleanup: true,
            verify_downloads: true,
            parallel_downloads: true,
            max_concurrent_downloads: 3,
            default_cuda_version: None,
            proxy_settings: None,
        }
    }
}

impl CudaMgrConfig {
    pub fn load() -> crate::error::CudaMgrResult<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Self = serde_json::from_str(&content).map_err(|e| {
                crate::error::ConfigError::Environment(format!("Failed to parse config: {}", e))
            })?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> crate::error::CudaMgrResult<()> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            crate::error::ConfigError::Environment(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.join(".cudamgr").join("config.json")
    }

    pub fn ensure_directories(&self) -> crate::error::CudaMgrResult<()> {
        std::fs::create_dir_all(&self.install_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }
}

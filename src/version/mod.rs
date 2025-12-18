pub mod registry;
pub mod resolver;
pub mod switcher;

use crate::error::{CudaMgrResult, VersionError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Version information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub install_path: PathBuf,
    pub is_active: bool,
    pub install_date: DateTime<Utc>,
    pub size_bytes: u64,
}

/// Version manager trait
pub trait VersionManager {
    async fn list_installed(&self) -> CudaMgrResult<Vec<VersionInfo>>;
    async fn list_available(&self) -> CudaMgrResult<Vec<String>>;
    async fn switch_version(&self, version: &str) -> CudaMgrResult<()>;
    async fn get_active_version(&self) -> CudaMgrResult<Option<String>>;
}

/// Default version manager implementation
pub struct DefaultVersionManager;

impl VersionManager for DefaultVersionManager {
    async fn list_installed(&self) -> CudaMgrResult<Vec<VersionInfo>> {
        // TODO: Implement installed version listing
        Err(VersionError::Registry("Version listing not yet implemented".to_string()).into())
    }

    async fn list_available(&self) -> CudaMgrResult<Vec<String>> {
        // TODO: Implement available version listing
        Err(
            VersionError::Registry("Available version listing not yet implemented".to_string())
                .into(),
        )
    }

    async fn switch_version(&self, _version: &str) -> CudaMgrResult<()> {
        // TODO: Implement version switching
        Err(VersionError::SwitchFailed("Version switching not yet implemented".to_string()).into())
    }

    async fn get_active_version(&self) -> CudaMgrResult<Option<String>> {
        // TODO: Implement active version detection
        Err(
            VersionError::Registry("Active version detection not yet implemented".to_string())
                .into(),
        )
    }
}

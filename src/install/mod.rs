pub mod downloader;
pub mod installer;
pub mod validator;
pub mod cleanup;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::error::{InstallError, CudaMgrResult};

/// Installation plan containing all necessary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationPlan {
    pub cuda_version: String,
    pub download_url: String,
    pub install_path: PathBuf,
    pub required_driver: Option<String>,
    pub dependencies: Vec<Dependency>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub required: bool,
}

/// Installer trait for platform-specific installation
pub trait Installer {
    async fn create_plan(&self, version: &str) -> CudaMgrResult<InstallationPlan>;
    async fn execute_plan(&self, plan: &InstallationPlan) -> CudaMgrResult<()>;
    async fn validate_installation(&self, path: &std::path::Path) -> CudaMgrResult<bool>;
}

/// Default installer implementation
pub struct DefaultInstaller;

impl Installer for DefaultInstaller {
    async fn create_plan(&self, _version: &str) -> CudaMgrResult<InstallationPlan> {
        // TODO: Implement installation plan creation
        Err(InstallError::Installation("Installation planning not yet implemented".to_string()).into())
    }

    async fn execute_plan(&self, _plan: &InstallationPlan) -> CudaMgrResult<()> {
        // TODO: Implement installation execution
        Err(InstallError::Installation("Installation execution not yet implemented".to_string()).into())
    }

    async fn validate_installation(&self, _path: &std::path::Path) -> CudaMgrResult<bool> {
        // TODO: Implement installation validation
        Err(InstallError::Validation("Installation validation not yet implemented".to_string()).into())
    }
}
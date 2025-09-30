pub mod gpu;
pub mod driver;
pub mod compiler;
pub mod distro;
pub mod storage;
pub mod security;

use serde::{Deserialize, Serialize};
use crate::error::{SystemError, CudaMgrResult};

/// Complete system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub gpu: Option<gpu::GpuInfo>,
    pub driver: Option<driver::DriverInfo>,
    pub compiler: Option<compiler::CompilerInfo>,
    pub distro: distro::DistroInfo,
    pub storage: storage::StorageInfo,
    pub security: security::SecurityInfo,
}

/// System checker trait for validating compatibility
pub trait SystemChecker {
    async fn check_system(&self) -> CudaMgrResult<SystemInfo>;
    async fn validate_compatibility(&self, cuda_version: &str) -> CudaMgrResult<bool>;
}

/// Default system checker implementation
pub struct DefaultSystemChecker;

impl SystemChecker for DefaultSystemChecker {
    async fn check_system(&self) -> CudaMgrResult<SystemInfo> {
        // TODO: Implement system checking logic
        Err(SystemError::CompatibilityCheck("System checking not yet implemented".to_string()).into())
    }

    async fn validate_compatibility(&self, _cuda_version: &str) -> CudaMgrResult<bool> {
        // TODO: Implement compatibility validation
        Err(SystemError::CompatibilityCheck("Compatibility validation not yet implemented".to_string()).into())
    }
}
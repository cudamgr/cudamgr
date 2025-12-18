pub mod compatibility;
pub mod compiler;
pub mod cuda;
pub mod distro;
pub mod driver;
pub mod gpu;
pub mod report;
pub mod security;
pub mod storage;
pub mod visual_studio;
pub mod wsl;

pub use compatibility::*;
pub use compiler::*;
pub use cuda::*;
pub use distro::*;
pub use driver::*;
pub use gpu::*;
pub use report::*;
pub use security::*;
pub use storage::*;
pub use visual_studio::*;
pub use wsl::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod platform_tests;

use crate::error::CudaMgrResult;
use serde::{Deserialize, Serialize};

/// Complete system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub gpu: Option<gpu::GpuInfo>,
    pub driver: Option<driver::DriverInfo>,
    pub compiler: Option<compiler::CompilerInfo>,
    pub distro: distro::DistroInfo,
    pub storage: storage::StorageInfo,
    pub security: security::SecurityInfo,
    pub wsl: Option<wsl::WslInfo>,
    pub visual_studio: Option<visual_studio::VisualStudioInfo>,
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
        // Detect GPU information
        let gpu = gpu::GpuInfo::detect().unwrap_or(None);

        // Detect driver information
        let driver = driver::DriverInfo::detect().ok().flatten();

        // Detect compiler information
        let compiler = compiler::CompilerInfo::detect()
            .ok()
            .and_then(|compilers| compilers.into_iter().find(|c| c.is_compatible));

        // Detect distribution information
        let distro = distro::DistroInfo::detect()?;

        // Detect storage information
        let storage_path = storage::StorageInfo::get_default_cuda_path();
        let storage = storage::StorageInfo::detect(&storage_path)?;

        // Detect security information
        // Detect security information
        let security = security::SecurityInfo::detect()?;

        // Detect WSL information
        let wsl = wsl::WslInfo::detect().ok();

        // Detect Visual Studio (Windows only)
        let visual_studio = visual_studio::VisualStudioInfo::detect().unwrap_or(None);

        Ok(SystemInfo {
            gpu,
            driver,
            compiler,
            distro,
            storage,
            security,
            wsl,
            visual_studio,
        })
    }

    async fn validate_compatibility(&self, cuda_version: &str) -> CudaMgrResult<bool> {
        let system_info = self.check_system().await?;

        // Check GPU compatibility
        if let Some(gpu) = &system_info.gpu {
            if !gpu.supports_cuda() {
                return Ok(false);
            }
        } else {
            // No GPU detected
            return Ok(false);
        }

        // Check driver compatibility
        if let Some(driver) = &system_info.driver {
            if !driver.supports_cuda_version(cuda_version) {
                return Ok(false);
            }
        } else {
            // No driver detected
            return Ok(false);
        }

        // Check compiler compatibility
        if system_info.compiler.is_none() {
            // No compatible compiler found
            return Ok(false);
        }

        // Check storage space
        if !system_info.storage.has_sufficient_space {
            return Ok(false);
        }

        Ok(true)
    }
}

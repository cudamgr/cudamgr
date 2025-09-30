use serde::{Deserialize, Serialize};

/// NVIDIA driver information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    pub version: String,
    pub is_installed: bool,
    pub supports_cuda: bool,
    pub max_cuda_version: Option<String>,
}

impl DriverInfo {
    /// Create a new DriverInfo instance
    pub fn new(
        version: String,
        is_installed: bool,
        supports_cuda: bool,
        max_cuda_version: Option<String>,
    ) -> Self {
        Self {
            version,
            is_installed,
            supports_cuda,
            max_cuda_version,
        }
    }

    /// Check if driver supports the given CUDA version
    pub fn supports_cuda_version(&self, _cuda_version: &str) -> bool {
        if !self.supports_cuda {
            return false;
        }
        
        // TODO: Implement version comparison logic
        true
    }
}
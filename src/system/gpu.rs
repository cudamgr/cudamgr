use serde::{Deserialize, Serialize};

/// GPU information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub memory_mb: u64,
    pub compute_capability: (u32, u32),
    pub driver_version: Option<String>,
    pub cuda_version: Option<String>,
}

impl GpuInfo {
    /// Create a new GpuInfo instance
    pub fn new(
        name: String,
        memory_mb: u64,
        compute_capability: (u32, u32),
        driver_version: Option<String>,
        cuda_version: Option<String>,
    ) -> Self {
        Self {
            name,
            memory_mb,
            compute_capability,
            driver_version,
            cuda_version,
        }
    }

    /// Check if GPU supports the given CUDA compute capability
    pub fn supports_compute_capability(&self, required: (u32, u32)) -> bool {
        self.compute_capability.0 > required.0 || 
        (self.compute_capability.0 == required.0 && self.compute_capability.1 >= required.1)
    }
}
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub memory_mb: Option<u64>,
    pub compute_capability: Option<(u32, u32)>,
    pub driver_version: Option<String>,
    pub pci_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown(String),
}

impl GpuInfo {
    pub fn new(name: String, vendor: GpuVendor) -> Self {
        Self {
            name,
            vendor,
            memory_mb: None,
            compute_capability: None,
            driver_version: None,
            pci_id: None,
        }
    }

    pub fn is_cuda_compatible(&self) -> bool {
        matches!(self.vendor, GpuVendor::Nvidia) && self.compute_capability.is_some()
    }

    pub fn supports_compute_capability(&self, required: (u32, u32)) -> bool {
        match self.compute_capability {
            Some(cap) => cap.0 > required.0 || (cap.0 == required.0 && cap.1 >= required.1),
            None => false,
        }
    }
}
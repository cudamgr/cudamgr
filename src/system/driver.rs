use serde::{Deserialize, Serialize};
use std::process::Command;
use crate::error::{SystemError, CudaMgrResult};

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

    /// Detect NVIDIA driver information
    pub fn detect() -> CudaMgrResult<Option<Self>> {
        // Try nvidia-smi first (most reliable)
        if let Ok(info) = Self::detect_via_nvidia_smi() {
            return Ok(Some(info));
        }

        // Try modinfo on Linux
        #[cfg(target_os = "linux")]
        if let Ok(info) = Self::detect_via_modinfo() {
            return Ok(Some(info));
        }

        // Try registry on Windows
        #[cfg(target_os = "windows")]
        if let Ok(info) = Self::detect_via_registry() {
            return Ok(Some(info));
        }

        // No driver found
        Ok(None)
    }

    /// Detect driver via nvidia-smi command
    fn detect_via_nvidia_smi() -> CudaMgrResult<Self> {
        let output = Command::new("nvidia-smi")
            .arg("--query-gpu=driver_version")
            .arg("--format=csv,noheader,nounits")
            .output()
            .map_err(|e| SystemError::DriverDetection(format!("Failed to run nvidia-smi: {}", e)))?;

        if !output.status.success() {
            return Err(SystemError::DriverDetection("nvidia-smi command failed".to_string()).into());
        }

        let version = String::from_utf8(output.stdout)
            .map_err(|e| SystemError::DriverDetection(format!("Invalid nvidia-smi output: {}", e)))?
            .trim()
            .to_string();

        if version.is_empty() {
            return Err(SystemError::DriverDetection("Empty driver version from nvidia-smi".to_string()).into());
        }

        let max_cuda_version = Self::get_max_cuda_version(&version);

        Ok(Self::new(
            version,
            true,
            true,
            max_cuda_version,
        ))
    }

    /// Detect driver via modinfo on Linux
    #[cfg(target_os = "linux")]
    fn detect_via_modinfo() -> CudaMgrResult<Self> {
        let output = Command::new("modinfo")
            .arg("nvidia")
            .output()
            .map_err(|e| SystemError::DriverDetection(format!("Failed to run modinfo: {}", e)))?;

        if !output.status.success() {
            return Err(SystemError::DriverDetection("modinfo nvidia command failed".to_string()).into());
        }

        let output_str = String::from_utf8(output.stdout)
            .map_err(|e| SystemError::DriverDetection(format!("Invalid modinfo output: {}", e)))?;

        // Parse version from modinfo output
        let version = output_str
            .lines()
            .find(|line| line.starts_with("version:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| SystemError::DriverDetection("Could not parse driver version from modinfo".to_string()))?
            .to_string();

        let max_cuda_version = Self::get_max_cuda_version(&version);

        Ok(Self::new(
            version,
            true,
            true,
            max_cuda_version,
        ))
    }

    /// Detect driver via Windows registry
    #[cfg(target_os = "windows")]
    fn detect_via_registry() -> CudaMgrResult<Self> {
        // This is a simplified implementation
        // In a real implementation, you'd use winreg crate to read registry
        Err(SystemError::DriverDetection("Windows registry detection not implemented".to_string()).into())
    }

    /// Get maximum supported CUDA version for a driver version
    pub fn get_max_cuda_version(driver_version: &str) -> Option<String> {
        // Parse driver version and map to CUDA version
        // This is a simplified mapping - real implementation would use NVIDIA's compatibility matrix
        let version_parts: Vec<&str> = driver_version.split('.').collect();
        if let Ok(major) = version_parts.get(0).unwrap_or(&"0").parse::<u32>() {
            match major {
                525.. => Some("12.0".to_string()),
                520..=524 => Some("11.8".to_string()),
                515..=519 => Some("11.7".to_string()),
                510..=514 => Some("11.6".to_string()),
                495..=509 => Some("11.5".to_string()),
                470..=494 => Some("11.4".to_string()),
                460..=469 => Some("11.2".to_string()),
                450..=459 => Some("11.0".to_string()),
                440..=449 => Some("10.2".to_string()),
                410..=439 => Some("10.1".to_string()),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Check if driver supports the given CUDA version
    pub fn supports_cuda_version(&self, cuda_version: &str) -> bool {
        if !self.supports_cuda {
            return false;
        }
        
        if let Some(max_version) = &self.max_cuda_version {
            Self::compare_versions(cuda_version, max_version) <= 0
        } else {
            false
        }
    }

    /// Compare two version strings (returns -1, 0, or 1)
    pub fn compare_versions(version1: &str, version2: &str) -> i32 {
        let v1_parts: Vec<u32> = version1.split('.').filter_map(|s| s.parse().ok()).collect();
        let v2_parts: Vec<u32> = version2.split('.').filter_map(|s| s.parse().ok()).collect();
        
        let max_len = v1_parts.len().max(v2_parts.len());
        
        for i in 0..max_len {
            let v1_part = v1_parts.get(i).unwrap_or(&0);
            let v2_part = v2_parts.get(i).unwrap_or(&0);
            
            match v1_part.cmp(v2_part) {
                std::cmp::Ordering::Less => return -1,
                std::cmp::Ordering::Greater => return 1,
                std::cmp::Ordering::Equal => continue,
            }
        }
        
        0
    }
}
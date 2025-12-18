use crate::error::{CudaMgrResult, SystemError};
use serde::{Deserialize, Serialize};
use std::process::Command;

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
        // Run nvidia-smi without arguments to get the full status table which includes "CUDA Version: XX.X"
        let output = Command::new("nvidia-smi").output().map_err(|e| {
            SystemError::DriverDetection(format!("Failed to run nvidia-smi: {}", e))
        })?;

        if !output.status.success() {
            return Err(
                SystemError::DriverDetection("nvidia-smi command failed".to_string()).into(),
            );
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse Driver Version
        // Look for: "Driver Version: 576.52"
        let driver_version = output_str
            .lines()
            .find(|line| line.contains("Driver Version:"))
            .and_then(|line| {
                let parts: Vec<&str> = line.split("Driver Version:").collect();
                parts
                    .get(1)
                    .map(|v| v.split_whitespace().next().unwrap_or("").trim().to_string())
            })
            .ok_or_else(|| {
                SystemError::DriverDetection("Could not parse driver version".to_string())
            })?;

        // Parse CUDA Version
        // Look for: "CUDA Version: 12.9"
        let max_cuda_version = output_str
            .lines()
            .find(|line| line.contains("CUDA Version:"))
            .and_then(|line| {
                let parts: Vec<&str> = line.split("CUDA Version:").collect();
                parts
                    .get(1)
                    .map(|v| v.split_whitespace().next().unwrap_or("").trim().to_string())
            })
            .or_else(|| {
                // Fallback to table mapping if parsing fails
                Self::get_max_cuda_version(&driver_version)
            });

        Ok(Self::new(
            driver_version,
            true,
            true, // Assuming modern drivers support CUDA
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
            return Err(
                SystemError::DriverDetection("modinfo nvidia command failed".to_string()).into(),
            );
        }

        let output_str = String::from_utf8(output.stdout)
            .map_err(|e| SystemError::DriverDetection(format!("Invalid modinfo output: {}", e)))?;

        // Parse version from modinfo output
        let version = output_str
            .lines()
            .find(|line| line.starts_with("version:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| {
                SystemError::DriverDetection(
                    "Could not parse driver version from modinfo".to_string(),
                )
            })?
            .to_string();

        let max_cuda_version = Self::get_max_cuda_version(&version);

        Ok(Self::new(version, true, true, max_cuda_version))
    }

    /// Detect driver via Windows registry
    #[cfg(target_os = "windows")]
    fn detect_via_registry() -> CudaMgrResult<Self> {
        use winreg::enums::*;
        use winreg::RegKey;

        // Try to find NVIDIA driver version in registry
        // Common location: HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}
        // We need to search through subkeys (0000, 0001, etc.) to find the one with "DriverVersion"

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let video_class_path =
            "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}";

        let video_class = hklm.open_subkey(video_class_path).map_err(|e| {
            SystemError::DriverDetection(format!("Failed to open video class registry key: {}", e))
        })?;

        for key_name in video_class.enum_keys().map(|x| x.unwrap_or_default()) {
            if let Ok(subkey) = video_class.open_subkey(&key_name) {
                // Check if this is an NVIDIA adapter
                let provider: String = subkey.get_value("ProviderName").unwrap_or_default();
                if provider.to_lowercase().contains("nvidia") {
                    if let Ok(version_raw) = subkey.get_value::<String, _>("DriverVersion") {
                        // Windows driver version format: XX.XX.XX.XXXX
                        // The last 5 digits usually contain the user-facing version
                        // Example: 31.0.15.3623 -> 536.23

                        let clean_version = version_raw.replace(".", "");
                        if clean_version.len() >= 5 {
                            let len = clean_version.len();
                            let last_five = &clean_version[len - 5..];
                            // Format as XXX.XX
                            let major = &last_five[..3];
                            let minor = &last_five[3..];

                            // Remove leading zero if present (e.g. 053 -> 53)
                            let major_trimmed = major.strip_prefix('0').unwrap_or(major);

                            let version = format!("{}.{}", major_trimmed, minor);
                            let max_cuda_version = Self::get_max_cuda_version(&version);

                            return Ok(Self::new(version, true, true, max_cuda_version));
                        }
                    }
                }
            }
        }

        // Fallback: Try NVTweak location
        if let Ok(nv_key) = hklm.open_subkey("SOFTWARE\\NVIDIA Corporation\\Global\\NVTweak") {
            if let Ok(version) = nv_key.get_value::<String, _>("DriverVersion") {
                let max_cuda_version = Self::get_max_cuda_version(&version);
                return Ok(Self::new(version, true, true, max_cuda_version));
            }
        }

        Err(SystemError::DriverDetection("NVIDIA driver not found in registry".to_string()).into())
    }

    /// Get maximum supported CUDA version for a driver version
    pub fn get_max_cuda_version(driver_version: &str) -> Option<String> {
        use crate::system::compatibility::REGISTRY;
        REGISTRY.get_max_cuda_version(driver_version)
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

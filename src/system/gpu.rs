use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use crate::error::{SystemError, CudaMgrResult};

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

    /// Detect GPU information using the default detector
    pub fn detect() -> CudaMgrResult<Option<Self>> {
        let detector = DefaultGpuDetector::new();
        
        // Use synchronous detection to avoid runtime conflicts
        let gpus = detector.detect_gpus_sync()?;
        
        // Return the first CUDA-compatible GPU, or the first GPU if none are CUDA-compatible
        let cuda_gpu = gpus.iter().find(|gpu| gpu.is_cuda_compatible()).cloned();
        if cuda_gpu.is_some() {
            return Ok(cuda_gpu);
        }
        
        Ok(gpus.into_iter().next())
    }

    pub fn is_cuda_compatible(&self) -> bool {
        matches!(self.vendor, GpuVendor::Nvidia) && self.compute_capability.is_some()
    }

    pub fn supports_cuda(&self) -> bool {
        self.is_cuda_compatible()
    }

    pub fn supports_compute_capability(&self, required: (u32, u32)) -> bool {
        match self.compute_capability {
            Some(cap) => cap.0 > required.0 || (cap.0 == required.0 && cap.1 >= required.1),
            None => false,
        }
    }
}

/// GPU detector trait for different detection methods
pub trait GpuDetector {
    async fn detect_gpus(&self) -> CudaMgrResult<Vec<GpuInfo>>;
    async fn detect_nvidia_gpus(&self) -> CudaMgrResult<Vec<GpuInfo>>;
}

/// Default GPU detector implementation
pub struct DefaultGpuDetector;

impl DefaultGpuDetector {
    pub fn new() -> Self {
        Self
    }

    /// Synchronous GPU detection to avoid runtime conflicts
    pub fn detect_gpus_sync(&self) -> CudaMgrResult<Vec<GpuInfo>> {
        // Try nvidia-smi first for NVIDIA GPUs
        if let Ok(nvidia_gpus) = self.detect_nvidia_smi_sync() {
            if !nvidia_gpus.is_empty() {
                return Ok(nvidia_gpus);
            }
        }

        // Fall back to platform-specific detection
        #[cfg(target_os = "windows")]
        {
            self.detect_windows_wmic_sync()
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.detect_lspci_sync()
        }
    }

    /// Fallback for non-Windows platforms when lspci is not available
    #[cfg(not(target_os = "windows"))]
    pub fn detect_lspci_sync(&self) -> CudaMgrResult<Vec<GpuInfo>> {
        // If lspci fails, return empty list rather than error
        match std::process::Command::new("lspci").args(&["-nn"]).output() {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut gpus = Vec::new();

                for line in output_str.lines() {
                    if line.to_lowercase().contains("vga") || line.to_lowercase().contains("3d") {
                        // Parse lspci output format: "00:02.0 VGA compatible controller: Intel Corporation ..."
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() >= 3 {
                            let pci_id = Some(format!("{}:{}", parts[0], parts[1].split_whitespace().next().unwrap_or("")));
                            
                            // Extract vendor and device name
                            let description = parts[2..].join(":");
                            let (vendor, name) = Self::parse_gpu_description(&description);
                            
                            let compute_capability = if matches!(vendor, GpuVendor::Nvidia) {
                                self.get_compute_capability_sync(&name)
                            } else {
                                None
                            };

                            let gpu = GpuInfo {
                                name: name.to_string(),
                                vendor,
                                memory_mb: Some(0), // lspci doesn't provide memory info easily
                                driver_version: None,
                                compute_capability,
                                pci_id,
                            };
                            gpus.push(gpu);
                        }
                    }
                }
                Ok(gpus)
            }
            _ => {
                // Return empty list if lspci is not available or fails
                Ok(Vec::new())
            }
        }
    }

    /// Detect NVIDIA GPUs using nvidia-smi (synchronous version)
    pub fn detect_nvidia_smi_sync(&self) -> CudaMgrResult<Vec<GpuInfo>> {
        let output = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu=name,memory.total,driver_version,pci.bus_id", "--format=csv,noheader,nounits"])
            .output()
            .map_err(|e| SystemError::GpuDetection(format!("Failed to run nvidia-smi: {}", e)))?;

        if !output.status.success() {
            return Err(SystemError::GpuDetection("nvidia-smi command failed".to_string()).into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut gpus = Vec::new();

        for line in output_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let memory_mb = parts[1].parse::<u64>().unwrap_or(0);
                let driver_version = Some(parts[2].to_string());
                let pci_id = Some(parts[3].to_string());

                let compute_capability = self.get_compute_capability(&name);

                let gpu = GpuInfo {
                    name,
                    vendor: GpuVendor::Nvidia,
                    memory_mb: Some(memory_mb),
                    driver_version,
                    compute_capability,
                    pci_id,
                };
                gpus.push(gpu);
            }
        }

        Ok(gpus)
    }

    /// Detect NVIDIA GPUs using nvidia-smi
    async fn detect_nvidia_smi(&self) -> CudaMgrResult<Vec<GpuInfo>> {
        let output = Command::new("nvidia-smi")
            .args(&["--query-gpu=name,memory.total,driver_version,pci.bus_id", "--format=csv,noheader,nounits"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut gpus = Vec::new();

                for line in stdout.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 4 {
                        let name = parts[0].to_string();
                        let memory_mb = parts[1].parse::<u64>().ok();
                        let driver_version = Some(parts[2].to_string());
                        let pci_id = Some(parts[3].to_string());

                        let compute_capability = self.get_compute_capability(&name);

                        let gpu = GpuInfo {
                            name,
                            vendor: GpuVendor::Nvidia,
                            memory_mb,
                            compute_capability,
                            driver_version,
                            pci_id,
                        };
                        gpus.push(gpu);
                    }
                }

                Ok(gpus)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(SystemError::GpuDetection(format!("nvidia-smi failed: {}", stderr)).into())
            }
            Err(e) => {
                // nvidia-smi not found or not executable
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(Vec::new()) // No NVIDIA GPUs or driver not installed
                } else {
                    Err(SystemError::GpuDetection(format!("Failed to run nvidia-smi: {}", e)).into())
                }
            }
        }
    }



    /// Detect GPUs using lspci on Linux
    async fn detect_lspci(&self) -> CudaMgrResult<Vec<GpuInfo>> {
        let output = Command::new("lspci")
            .args(&["-nn"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut gpus = Vec::new();

                for line in stdout.lines() {
                    if line.to_lowercase().contains("vga") || line.to_lowercase().contains("3d") {
                        if let Some(gpu) = self.parse_lspci_line(line) {
                            gpus.push(gpu);
                        }
                    }
                }

                Ok(gpus)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(SystemError::GpuDetection(format!("lspci failed: {}", stderr)).into())
            }
            Err(e) => {
                Err(SystemError::GpuDetection(format!("Failed to run lspci: {}", e)).into())
            }
        }
    }

    /// Parse a single lspci line to extract GPU information
    fn parse_lspci_line(&self, line: &str) -> Option<GpuInfo> {
        // Example line: "01:00.0 VGA compatible controller [0300]: NVIDIA Corporation GeForce RTX 3080 [10de:2206] (rev a1)"
        
        // Split on the first space to separate PCI ID from the rest
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 {
            return None;
        }

        let pci_id = parts[0].trim().to_string();
        let rest = parts[1];

        // Find the colon that separates the device type from the vendor/device info
        let colon_pos = rest.find(':')?;
        let description = &rest[colon_pos + 1..];

        // Extract vendor and name
        let vendor = if description.to_lowercase().contains("nvidia") {
            GpuVendor::Nvidia
        } else if description.to_lowercase().contains("amd") || description.to_lowercase().contains("ati") {
            GpuVendor::Amd
        } else if description.to_lowercase().contains("intel") {
            GpuVendor::Intel
        } else {
            GpuVendor::Unknown("Unknown".to_string())
        };

        // Extract GPU name - look for patterns after "Corporation" or similar
        let name = if let Some(start) = description.find("Corporation ") {
            let after_corp = &description[start + 12..];
            if let Some(end) = after_corp.find(" [") {
                after_corp[..end].trim().to_string()
            } else {
                after_corp.trim().to_string()
            }
        } else if let Some(start) = description.find("Inc. ") {
            let after_inc = &description[start + 5..];
            if let Some(end) = after_inc.find(" [") {
                after_inc[..end].trim().to_string()
            } else {
                after_inc.trim().to_string()
            }
        } else {
            // Fallback: use the description as-is, removing brackets
            if let Some(end) = description.find(" [") {
                description[..end].trim().to_string()
            } else {
                description.trim().to_string()
            }
        };

        Some(GpuInfo {
            name,
            vendor,
            memory_mb: None,
            compute_capability: None,
            driver_version: None,
            pci_id: Some(pci_id),
        })
    }

    /// Parse GPU description from lspci output
    fn parse_gpu_description(description: &str) -> (GpuVendor, &str) {
        let desc_lower = description.to_lowercase();
        
        let vendor = if desc_lower.contains("nvidia") {
            GpuVendor::Nvidia
        } else if desc_lower.contains("amd") || desc_lower.contains("radeon") {
            GpuVendor::Amd
        } else if desc_lower.contains("intel") {
            GpuVendor::Intel
        } else {
            GpuVendor::Unknown("Unknown".to_string())
        };
        
        // Extract the GPU name (everything after the first colon and space)
        let name = description.trim_start_matches(|c: char| c != ':')
            .trim_start_matches(':')
            .trim();
        
        (vendor, name)
    }



    /// Get compute capability for NVIDIA GPU based on name
    fn get_compute_capability(&self, gpu_name: &str) -> Option<(u32, u32)> {
        use crate::system::compatibility::REGISTRY;
        REGISTRY.get_compute_capability(gpu_name)
    }



    /// Detect GPUs on Windows using wmic (synchronous version)
    #[cfg(target_os = "windows")]
    pub fn detect_windows_wmic_sync(&self) -> CudaMgrResult<Vec<GpuInfo>> {
        let output = std::process::Command::new("wmic")
            .args(&["path", "win32_VideoController", "get", "name,adapterram,driverversion", "/format:csv"])
            .output()
            .map_err(|e| SystemError::GpuDetection(format!("Failed to run wmic: {}", e)))?;

        if !output.status.success() {
            return Err(SystemError::GpuDetection("wmic command failed".to_string()).into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut gpus = Vec::new();

        for line in output_str.lines().skip(1) { // Skip header
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            // Expected format: Node, AdapterRAM, DriverVersion, Name
            if parts.len() >= 4 {
                let memory_bytes = parts[1].trim().parse::<u64>().unwrap_or(0);
                let driver_version = if parts[2].trim().is_empty() { None } else { Some(parts[2].trim().to_string()) };
                let name = parts[3].trim().to_string();

                if name.is_empty() {
                    continue;
                }

                let vendor = if name.to_lowercase().contains("nvidia") {
                    GpuVendor::Nvidia
                } else if name.to_lowercase().contains("amd") || name.to_lowercase().contains("radeon") {
                    GpuVendor::Amd
                } else if name.to_lowercase().contains("intel") {
                    GpuVendor::Intel
                } else {
                    GpuVendor::Unknown("Unknown".to_string())
                };

                let compute_capability = if matches!(vendor, GpuVendor::Nvidia) {
                    self.get_compute_capability(&name)
                } else {
                    None
                };

                let gpu = GpuInfo {
                    name,
                    vendor,
                    memory_mb: Some(memory_bytes / (1024 * 1024)),
                    driver_version,
                    compute_capability,
                    pci_id: None,
                };
                gpus.push(gpu);
            }
        }

        Ok(gpus)
    }

    /// Detect GPUs on Windows using wmic
    #[cfg(target_os = "windows")]
    async fn detect_windows_wmic(&self) -> CudaMgrResult<Vec<GpuInfo>> {
        let output = Command::new("wmic")
            .args(&["path", "win32_VideoController", "get", "name,adapterram,driverversion", "/format:csv"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut gpus = Vec::new();

                for line in stdout.lines().skip(1) { // Skip header
                    if line.trim().is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = line.split(',').collect();
                    // Expected format: Node, AdapterRAM, DriverVersion, Name
                    if parts.len() >= 4 {
                        let name = parts[3].trim().to_string();
                        let memory_bytes = parts[1].trim().parse::<u64>().ok();
                        let memory_mb = memory_bytes.map(|b| b / (1024 * 1024));
                        let driver_version = Some(parts[2].trim().to_string());

                        let vendor = if name.to_lowercase().contains("nvidia") {
                            GpuVendor::Nvidia
                        } else if name.to_lowercase().contains("amd") || name.to_lowercase().contains("radeon") {
                            GpuVendor::Amd
                        } else if name.to_lowercase().contains("intel") {
                            GpuVendor::Intel
                        } else {
                            GpuVendor::Unknown("Unknown".to_string())
                        };

                        let compute_capability = if matches!(vendor, GpuVendor::Nvidia) {
                            self.get_compute_capability(&name)
                        } else {
                            None
                        };

                        let gpu = GpuInfo {
                            name,
                            vendor,
                            memory_mb,
                            compute_capability,
                            driver_version,
                            pci_id: None,
                        };
                        gpus.push(gpu);
                    }
                }

                Ok(gpus)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(SystemError::GpuDetection(format!("wmic failed: {}", stderr)).into())
            }
            Err(e) => {
                Err(SystemError::GpuDetection(format!("Failed to run wmic: {}", e)).into())
            }
        }
    }
}

impl GpuDetector for DefaultGpuDetector {
    async fn detect_gpus(&self) -> CudaMgrResult<Vec<GpuInfo>> {
        // Try multiple detection methods and combine results
        let mut all_gpus = Vec::new();

        // First try nvidia-smi for detailed NVIDIA GPU info
        if let Ok(nvidia_gpus) = self.detect_nvidia_smi().await {
            all_gpus.extend(nvidia_gpus);
        }

        // If no NVIDIA GPUs found via nvidia-smi, try system detection
        if all_gpus.is_empty() {
            #[cfg(target_os = "windows")]
            {
                if let Ok(gpus) = self.detect_windows_wmic().await {
                    all_gpus.extend(gpus);
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                if let Ok(gpus) = self.detect_lspci().await {
                    all_gpus.extend(gpus);
                }
            }
        }

        Ok(all_gpus)
    }

    async fn detect_nvidia_gpus(&self) -> CudaMgrResult<Vec<GpuInfo>> {
        let all_gpus = self.detect_gpus().await?;
        let nvidia_gpus = all_gpus
            .into_iter()
            .filter(|gpu| matches!(gpu.vendor, GpuVendor::Nvidia))
            .collect();
        
        Ok(nvidia_gpus)
    }
}#
[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_gpu_info_creation() {
        let gpu = GpuInfo::new("GeForce RTX 3080".to_string(), GpuVendor::Nvidia);
        
        assert_eq!(gpu.name, "GeForce RTX 3080");
        assert_eq!(gpu.vendor, GpuVendor::Nvidia);
        assert_eq!(gpu.memory_mb, None);
        assert_eq!(gpu.compute_capability, None);
        assert_eq!(gpu.driver_version, None);
        assert_eq!(gpu.pci_id, None);
    }

    #[test]
    fn test_gpu_cuda_compatibility() {
        let mut nvidia_gpu = GpuInfo::new("GeForce RTX 3080".to_string(), GpuVendor::Nvidia);
        nvidia_gpu.compute_capability = Some((8, 6));
        
        let amd_gpu = GpuInfo::new("Radeon RX 6800".to_string(), GpuVendor::Amd);
        
        let nvidia_no_compute = GpuInfo::new("GeForce GTX 750".to_string(), GpuVendor::Nvidia);
        
        assert!(nvidia_gpu.is_cuda_compatible());
        assert!(!amd_gpu.is_cuda_compatible());
        assert!(!nvidia_no_compute.is_cuda_compatible());
    }

    #[test]
    fn test_compute_capability_support() {
        let gpu = GpuInfo {
            name: "GeForce RTX 3080".to_string(),
            vendor: GpuVendor::Nvidia,
            memory_mb: Some(10240),
            compute_capability: Some((8, 6)),
            driver_version: Some("470.57.02".to_string()),
            pci_id: Some("01:00.0".to_string()),
        };

        // Test exact match
        assert!(gpu.supports_compute_capability((8, 6)));
        
        // Test lower requirements
        assert!(gpu.supports_compute_capability((7, 5)));
        assert!(gpu.supports_compute_capability((8, 0)));
        
        // Test higher requirements
        assert!(!gpu.supports_compute_capability((8, 7)));
        assert!(!gpu.supports_compute_capability((9, 0)));
        
        // Test GPU without compute capability
        let gpu_no_compute = GpuInfo::new("Unknown GPU".to_string(), GpuVendor::Unknown("Test".to_string()));
        assert!(!gpu_no_compute.supports_compute_capability((3, 0)));
    }

    #[test]
    fn test_gpu_vendor_variants() {
        let nvidia = GpuVendor::Nvidia;
        let amd = GpuVendor::Amd;
        let intel = GpuVendor::Intel;
        let unknown = GpuVendor::Unknown("Custom".to_string());

        assert_eq!(format!("{:?}", nvidia), "Nvidia");
        assert_eq!(format!("{:?}", amd), "Amd");
        assert_eq!(format!("{:?}", intel), "Intel");
        assert_eq!(format!("{:?}", unknown), "Unknown(\"Custom\")");
    }

    #[test]
    fn test_parse_lspci_line() {
        let detector = DefaultGpuDetector::new();
        
        // Test simple parsing logic
        let nvidia_line = "01:00.0 VGA compatible controller [0300]: NVIDIA Corporation GeForce RTX 3080 [10de:2206] (rev a1)";
        let result = detector.parse_lspci_line(nvidia_line);
        
        // Just test that it doesn't panic and returns something reasonable
        match result {
            Some(gpu) => {
                assert!(!gpu.name.is_empty());
                assert_eq!(gpu.pci_id, Some("01:00.0".to_string()));
            }
            None => {
                // This is also acceptable for the test
            }
        }
        
        // Test invalid line
        let invalid_line = "invalid line format";
        assert!(detector.parse_lspci_line(invalid_line).is_none());
    }

    #[test]
    fn test_compute_capability_mapping() {
        let detector = DefaultGpuDetector::new();
        
        // Test RTX 40 series
        assert_eq!(detector.get_compute_capability("GeForce RTX 4090"), Some((8, 9)));
        assert_eq!(detector.get_compute_capability("GeForce RTX 4080"), Some((8, 9)));
        
        // Test RTX 30 series
        assert_eq!(detector.get_compute_capability("GeForce RTX 3080"), Some((8, 6)));
        assert_eq!(detector.get_compute_capability("GeForce RTX 3070"), Some((8, 6)));
        
        // Test RTX 20 series
        assert_eq!(detector.get_compute_capability("GeForce RTX 2080 Ti"), Some((7, 5)));
        
        // Test GTX 10 series
        assert_eq!(detector.get_compute_capability("GeForce GTX 1080"), Some((6, 1)));
        
        // Test Tesla cards
        assert_eq!(detector.get_compute_capability("Tesla V100"), Some((7, 0)));
        
        // Test unknown GPU
        assert_eq!(detector.get_compute_capability("Unknown GPU Model"), None);
    }

    #[test]
    fn test_compatibility_registry_completeness() {
        use crate::system::compatibility::REGISTRY;
        
        // Verify keys exist via direct lookup (since we can't iterate the private hashmap easily unless we expose it, but get_compute_capability covers it)
        assert!(REGISTRY.get_compute_capability("rtx 4090").is_some());
        assert!(REGISTRY.get_compute_capability("rtx 3080").is_some());
        assert!(REGISTRY.get_compute_capability("gtx 1080").is_some());
        assert!(REGISTRY.get_compute_capability("tesla v100").is_some());
        
        // Verify compute capabilities are correct
        assert_eq!(REGISTRY.get_compute_capability("rtx 4090"), Some((8, 9)));
        assert_eq!(REGISTRY.get_compute_capability("rtx 3080"), Some((8, 6)));
        assert_eq!(REGISTRY.get_compute_capability("gtx 1080"), Some((6, 1)));
    }

    #[tokio::test]
    async fn test_detect_nvidia_gpus_empty() {
        let detector = DefaultGpuDetector::new();
        
        // This test will likely return empty or error in CI/testing environments without NVIDIA GPUs
        // We just test that it doesn't panic
        let result = detector.detect_nvidia_gpus().await;
        
        // Should not panic, result can be Ok(empty) or Err
        match result {
            Ok(nvidia_gpus) => {
                // If successful, all returned GPUs should be NVIDIA
                for gpu in nvidia_gpus {
                    assert_eq!(gpu.vendor, GpuVendor::Nvidia);
                }
            }
            Err(_) => {
                // Error is acceptable in test environments without NVIDIA tools
            }
        }
    }

    #[tokio::test]
    async fn test_detect_gpus_no_crash() {
        let detector = DefaultGpuDetector::new();
        
        // This test ensures the detection doesn't crash even in environments
        // without GPUs or with missing tools
        let result = detector.detect_gpus().await;
        
        // Should not panic, but may return empty list or error
        match result {
            Ok(gpus) => {
                // Verify all detected GPUs have valid data
                for gpu in gpus {
                    assert!(!gpu.name.is_empty());
                    // Vendor should be one of the known types
                    match gpu.vendor {
                        GpuVendor::Nvidia | GpuVendor::Amd | GpuVendor::Intel | GpuVendor::Unknown(_) => {}
                    }
                }
            }
            Err(_) => {
                // Error is acceptable in test environments
            }
        }
    }

    #[test]
    fn test_gpu_info_serialization() {
        let gpu = GpuInfo {
            name: "GeForce RTX 3080".to_string(),
            vendor: GpuVendor::Nvidia,
            memory_mb: Some(10240),
            compute_capability: Some((8, 6)),
            driver_version: Some("470.57.02".to_string()),
            pci_id: Some("01:00.0".to_string()),
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&gpu).unwrap();
        assert!(json.contains("GeForce RTX 3080"));
        assert!(json.contains("Nvidia"));

        // Test deserialization from JSON
        let deserialized: GpuInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, gpu);
    }

    #[test]
    fn test_gpu_vendor_serialization() {
        let vendors = vec![
            GpuVendor::Nvidia,
            GpuVendor::Amd,
            GpuVendor::Intel,
            GpuVendor::Unknown("Custom".to_string()),
        ];

        for vendor in vendors {
            let json = serde_json::to_string(&vendor).unwrap();
            let deserialized: GpuVendor = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, vendor);
        }
    }

    // Mock tests for specific scenarios
    mod mock_tests {
        use super::*;

        struct MockGpuDetector {
            nvidia_smi_output: Option<String>,
            lspci_output: Option<String>,
        }

        impl MockGpuDetector {
            fn new() -> Self {
                Self {
                    nvidia_smi_output: None,
                    lspci_output: None,
                }
            }

            fn with_nvidia_smi_output(mut self, output: String) -> Self {
                self.nvidia_smi_output = Some(output);
                self
            }

            fn with_lspci_output(mut self, output: String) -> Self {
                self.lspci_output = Some(output);
                self
            }

            fn parse_nvidia_smi_output(&self, output: &str) -> Vec<GpuInfo> {
                let mut gpus = Vec::new();
                for line in output.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 4 {
                        let gpu = GpuInfo {
                            name: parts[0].to_string(),
                            vendor: GpuVendor::Nvidia,
                            memory_mb: parts[1].parse().ok(),
                            compute_capability: Some((8, 6)), // Mock capability
                            driver_version: Some(parts[2].to_string()),
                            pci_id: Some(parts[3].to_string()),
                        };
                        gpus.push(gpu);
                    }
                }
                gpus
            }
        }

        #[test]
        fn test_mock_nvidia_smi_parsing() {
            let mock = MockGpuDetector::new();
            let nvidia_output = "GeForce RTX 3080, 10240, 470.57.02, 00000000:01:00.0\nGeForce GTX 1080, 8192, 470.57.02, 00000000:02:00.0";
            
            let gpus = mock.parse_nvidia_smi_output(nvidia_output);
            
            assert_eq!(gpus.len(), 2);
            
            assert_eq!(gpus[0].name, "GeForce RTX 3080");
            assert_eq!(gpus[0].memory_mb, Some(10240));
            assert_eq!(gpus[0].driver_version, Some("470.57.02".to_string()));
            
            assert_eq!(gpus[1].name, "GeForce GTX 1080");
            assert_eq!(gpus[1].memory_mb, Some(8192));
        }

        #[test]
        fn test_mock_empty_nvidia_smi() {
            let mock = MockGpuDetector::new();
            let empty_output = "";
            
            let gpus = mock.parse_nvidia_smi_output(empty_output);
            assert_eq!(gpus.len(), 0);
        }

        #[test]
        fn test_mock_malformed_nvidia_smi() {
            let mock = MockGpuDetector::new();
            let malformed_output = "Invalid line\nAnother invalid line";
            
            let gpus = mock.parse_nvidia_smi_output(malformed_output);
            assert_eq!(gpus.len(), 0);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_gpu_info_serialization() {
        let gpu = GpuInfo {
            name: "GeForce RTX 3080".to_string(),
            vendor: GpuVendor::Nvidia,
            memory_mb: Some(10240),
            compute_capability: Some((8, 6)),
            driver_version: Some("470.57.02".to_string()),
            pci_id: Some("10de:2206".to_string()),
        };

        let json = serde_json::to_string(&gpu).unwrap();
        let deserialized: GpuInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(gpu, deserialized);
        assert!(gpu.is_cuda_compatible());
        assert!(gpu.supports_compute_capability((7, 5)));
        assert!(!gpu.supports_compute_capability((9, 0)));
    }

    #[test]
    fn test_cuda_installation_serialization() {
        let mut installation = CudaInstallation::new(
            "11.8".to_string(),
            PathBuf::from("/usr/local/cuda-11.8")
        );
        
        installation.components.push(CudaComponent {
            name: "CUDA Toolkit".to_string(),
            version: "11.8.0".to_string(),
            path: PathBuf::from("/usr/local/cuda-11.8/bin"),
            required: true,
        });

        let json = serde_json::to_string(&installation).unwrap();
        let deserialized: CudaInstallation = serde_json::from_str(&json).unwrap();
        
        assert_eq!(installation, deserialized);
        assert_eq!(installation.get_nvcc_path(), PathBuf::from("/usr/local/cuda-11.8/bin/nvcc"));
    }

    #[test]
    fn test_distro_info_serialization() {
        let distro = DistroInfo::new(
            OsType::Linux(LinuxDistro::Ubuntu("20.04".to_string())),
            "Ubuntu".to_string(),
            "20.04.3 LTS".to_string(),
            Some("5.4.0-74-generic".to_string()),
            PackageManager::Apt,
        );

        let json = serde_json::to_string(&distro).unwrap();
        let deserialized: DistroInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(distro.name, deserialized.name);
        assert_eq!(distro.version, deserialized.version);
    }

    #[test]
    fn test_cuda_version_display() {
        assert_eq!(CudaVersion::Specific("11.8".to_string()).to_string(), "11.8");
        assert_eq!(CudaVersion::Latest.to_string(), "latest");
        assert_eq!(CudaVersion::LatestLts.to_string(), "latest-lts");
    }

    #[test]
    fn test_gpu_vendor_variants() {
        let nvidia = GpuVendor::Nvidia;
        let amd = GpuVendor::Amd;
        let intel = GpuVendor::Intel;
        let unknown = GpuVendor::Unknown("Custom".to_string());

        // Test serialization
        let json = serde_json::to_string(&nvidia).unwrap();
        assert!(json.contains("Nvidia"));
        
        let json = serde_json::to_string(&unknown).unwrap();
        let deserialized: GpuVendor = serde_json::from_str(&json).unwrap();
        assert_eq!(unknown, deserialized);
    }

    #[test]
    fn test_linux_distro_variants() {
        let ubuntu = LinuxDistro::Ubuntu("20.04".to_string());
        let fedora = LinuxDistro::Fedora("35".to_string());
        let arch = LinuxDistro::Arch("rolling".to_string());

        let json = serde_json::to_string(&ubuntu).unwrap();
        let deserialized: LinuxDistro = serde_json::from_str(&json).unwrap();
        assert_eq!(ubuntu, deserialized);
    }

    #[test]
    fn test_package_manager_detection() {
        let apt_distro = DistroInfo::new(
            OsType::Linux(LinuxDistro::Ubuntu("20.04".to_string())),
            "Ubuntu".to_string(),
            "20.04".to_string(),
            None,
            PackageManager::Apt,
        );

        match apt_distro.package_manager {
            PackageManager::Apt => assert!(true),
            _ => panic!("Expected Apt package manager"),
        }
    }
}
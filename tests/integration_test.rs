use cudamgr::config::CudaMgrConfig;
use cudamgr::system::{CudaInstallation, GpuInfo, GpuVendor};
use std::path::PathBuf;

#[test]
fn test_data_model_integration() {
    // Test GPU info
    let gpu = GpuInfo::new("RTX 3080".to_string(), GpuVendor::Nvidia);
    assert!(matches!(gpu.vendor, GpuVendor::Nvidia));
    // let installation = CudaInstallation::new(
    //     "11.8".to_string(),
    //     PathBuf::from("/usr/local/cuda-11.8")
    // );

    // Test CUDA installation
    let install_path = if cfg!(windows) {
        PathBuf::from("C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA\\v11.8")
    } else {
        PathBuf::from("/usr/local/cuda-11.8")
    };
    let installation = CudaInstallation::new("11.8".to_string(), install_path.clone());
    assert_eq!(installation.version, "11.8");
    // assert_eq!(installation.get_nvcc_path(), PathBuf::from("/usr/local/cuda-11.8/bin/nvcc"));
    let binary_name = if cfg!(windows) { "nvcc.exe" } else { "nvcc" };
    let expected_path = install_path.join("bin").join(binary_name);

    assert_eq!(installation.get_nvcc_path(), expected_path);
    // Test config
    let config = CudaMgrConfig::default();
    assert!(config.auto_cleanup);
    assert!(config.verify_downloads);
}

#[test]
fn test_serialization_roundtrip() {
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
}

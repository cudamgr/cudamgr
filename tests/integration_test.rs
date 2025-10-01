use cudamgr::system::{GpuInfo, GpuVendor, CudaInstallation};
use cudamgr::config::CudaMgrConfig;
use std::path::PathBuf;

#[test]
fn test_data_model_integration() {
    // Test GPU info
    let gpu = GpuInfo::new("RTX 3080".to_string(), GpuVendor::Nvidia);
    assert!(matches!(gpu.vendor, GpuVendor::Nvidia));
    
    // Test CUDA installation
    let installation = CudaInstallation::new(
        "11.8".to_string(),
        PathBuf::from("/usr/local/cuda-11.8")
    );
    assert_eq!(installation.version, "11.8");
    assert_eq!(installation.get_nvcc_path(), PathBuf::from("/usr/local/cuda-11.8/bin/nvcc"));
    
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
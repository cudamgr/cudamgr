#[cfg(test)]
mod tests {
    use crate::system::*;

    #[test]
    fn test_gpu_info_creation() {
        let gpu = GpuInfo::new("GeForce RTX 3080".to_string(), GpuVendor::Nvidia);
        assert_eq!(gpu.name, "GeForce RTX 3080");
        assert_eq!(gpu.vendor, GpuVendor::Nvidia);
    }

    #[test]
    fn test_gpu_vendor_variants() {
        let nvidia = GpuVendor::Nvidia;
        let amd = GpuVendor::Amd;
        let intel = GpuVendor::Intel;
        let unknown = GpuVendor::Unknown("Custom".to_string());

        // Test that they can be created and matched
        assert!(matches!(nvidia, GpuVendor::Nvidia));
        assert!(matches!(amd, GpuVendor::Amd));
        assert!(matches!(intel, GpuVendor::Intel));
        assert!(matches!(unknown, GpuVendor::Unknown(_)));
    }

    #[test]
    fn test_system_checker_trait() {
        let _checker = DefaultSystemChecker;
        // Just test that the struct can be created
        assert!(true);
    }
}
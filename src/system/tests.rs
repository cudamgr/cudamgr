#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::{driver::*, compiler::*, distro::*, storage::*, DefaultSystemChecker};

    #[test]
    fn test_system_checker_creation() {
        let _checker = DefaultSystemChecker;
        // Just test that the struct can be created
        assert!(true);
    }

    #[test]
    fn test_driver_info_creation() {
        let driver = DriverInfo::new(
            "470.86".to_string(),
            true,
            true,
            Some("11.4".to_string()),
        );
        
        assert_eq!(driver.version, "470.86");
        assert!(driver.is_installed);
        assert!(driver.supports_cuda);
        assert_eq!(driver.max_cuda_version, Some("11.4".to_string()));
    }

    #[test]
    fn test_driver_cuda_version_support() {
        let driver = DriverInfo::new(
            "470.86".to_string(),
            true,
            true,
            Some("11.4".to_string()),
        );
        
        // Should support older versions
        assert!(driver.supports_cuda_version("11.0"));
        assert!(driver.supports_cuda_version("11.4"));
        
        // Should not support newer versions
        assert!(!driver.supports_cuda_version("12.0"));
    }

    #[test]
    fn test_driver_no_cuda_support() {
        let driver = DriverInfo::new(
            "390.48".to_string(),
            true,
            false,
            None,
        );
        
        assert!(!driver.supports_cuda_version("11.0"));
    }

    #[test]
    fn test_driver_version_comparison() {
        assert_eq!(DriverInfo::compare_versions("11.0", "11.4"), -1);
        assert_eq!(DriverInfo::compare_versions("11.4", "11.0"), 1);
        assert_eq!(DriverInfo::compare_versions("11.4", "11.4"), 0);
        assert_eq!(DriverInfo::compare_versions("12.0", "11.8"), 1);
    }

    #[test]
    fn test_driver_max_cuda_version_mapping() {
        assert_eq!(DriverInfo::get_max_cuda_version("525.60.13"), Some("12.0".to_string()));
        assert_eq!(DriverInfo::get_max_cuda_version("520.61.05"), Some("11.8".to_string()));
        assert_eq!(DriverInfo::get_max_cuda_version("470.86"), Some("11.4".to_string()));
        assert_eq!(DriverInfo::get_max_cuda_version("390.48"), None);
    }

    #[test]
    fn test_compiler_info_creation() {
        let compiler = CompilerInfo::new(
            "GCC".to_string(),
            "9.4.0".to_string(),
            true,
            Some("/usr/bin/gcc".to_string()),
        );
        
        assert_eq!(compiler.name, "GCC");
        assert_eq!(compiler.version, "9.4.0");
        assert!(compiler.is_compatible);
        assert_eq!(compiler.path, Some("/usr/bin/gcc".to_string()));
    }

    #[test]
    fn test_gcc_compatibility() {
        assert!(CompilerInfo::is_gcc_compatible("9.4.0"));
        assert!(CompilerInfo::is_gcc_compatible("11.2.0"));
        assert!(!CompilerInfo::is_gcc_compatible("4.8.0"));
        assert!(!CompilerInfo::is_gcc_compatible("13.0.0"));
    }

    #[test]
    fn test_clang_compatibility() {
        assert!(CompilerInfo::is_clang_compatible("12.0.0"));
        assert!(CompilerInfo::is_clang_compatible("15.0.0"));
        assert!(!CompilerInfo::is_clang_compatible("5.0.0"));
        assert!(!CompilerInfo::is_clang_compatible("17.0.0"));
    }

    #[test]
    fn test_gcc_version_parsing() {
        let output = "gcc (Ubuntu 9.4.0-1ubuntu1~20.04.1) 9.4.0\nCopyright (C) 2019 Free Software Foundation, Inc.";
        let version = CompilerInfo::parse_gcc_version(output).unwrap();
        assert_eq!(version, "9.4.0");
    }

    #[test]
    fn test_clang_version_parsing() {
        let output = "clang version 12.0.0-1ubuntu1~20.04.5\nTarget: x86_64-pc-linux-gnu";
        let version = CompilerInfo::parse_clang_version(output).unwrap();
        assert_eq!(version, "12.0.0-1ubuntu1~20.04.5");
    }

    #[test]
    fn test_distro_info_creation() {
        let distro = DistroInfo::new(
            OsType::Linux(LinuxDistro::Ubuntu("20.04".to_string())),
            "Ubuntu".to_string(),
            "20.04".to_string(),
            Some("5.4.0-74-generic".to_string()),
            PackageManager::Apt,
        );
        
        assert_eq!(distro.name, "Ubuntu");
        assert_eq!(distro.version, "20.04");
        assert_eq!(distro.kernel_version, Some("5.4.0-74-generic".to_string()));
        assert!(matches!(distro.package_manager, PackageManager::Apt));
    }

    #[test]
    fn test_os_release_parsing() {
        let content = r#"NAME="Ubuntu"
VERSION="20.04.2 LTS (Focal Fossa)"
ID=ubuntu
ID_LIKE=debian
PRETTY_NAME="Ubuntu 20.04.2 LTS"
VERSION_ID="20.04"
HOME_URL="https://www.ubuntu.com/"
SUPPORT_URL="https://help.ubuntu.com/"
BUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"
PRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"
VERSION_CODENAME=focal
UBUNTU_CODENAME=focal"#;

        let distro = DistroInfo::parse_os_release(content).unwrap();
        assert_eq!(distro.name, "Ubuntu");
        assert!(matches!(distro.os_type, OsType::Linux(LinuxDistro::Ubuntu(_))));
        assert!(matches!(distro.package_manager, PackageManager::Apt));
    }

    #[test]
    fn test_lsb_release_parsing() {
        let content = r#"DISTRIB_ID=Ubuntu
DISTRIB_RELEASE=20.04
DISTRIB_CODENAME=focal
DISTRIB_DESCRIPTION="Ubuntu 20.04.2 LTS""#;

        let distro = DistroInfo::parse_lsb_release(content).unwrap();
        assert_eq!(distro.name, "Ubuntu 20.04.2 LTS");
        assert_eq!(distro.version, "20.04");
    }

    #[test]
    fn test_storage_info_creation() {
        let storage = StorageInfo::new(
            100, // 100 GB available
            500, // 500 GB total
            "/usr/local/cuda".to_string(),
            6,   // 6 GB required
        );
        
        assert_eq!(storage.available_space_gb, 100);
        assert_eq!(storage.total_space_gb, 500);
        assert_eq!(storage.install_path, "/usr/local/cuda");
        assert!(storage.has_sufficient_space);
    }

    #[test]
    fn test_storage_insufficient_space() {
        let storage = StorageInfo::new(
            2,   // 2 GB available
            500, // 500 GB total
            "/usr/local/cuda".to_string(),
            6,   // 6 GB required
        );
        
        assert!(!storage.has_sufficient_space);
        assert!(!storage.check_space_requirement(6));
        assert!(storage.check_space_requirement(1));
    }

    #[test]
    fn test_storage_format_space_info() {
        let storage = StorageInfo::new(
            100,
            500,
            "/usr/local/cuda".to_string(),
            6,
        );
        
        let formatted = storage.format_space_info();
        assert_eq!(formatted, "100.0 GB available / 500.0 GB total");
    }

    #[test]
    fn test_default_cuda_paths() {
        let path = StorageInfo::get_default_cuda_path();
        
        #[cfg(target_os = "linux")]
        assert_eq!(path.to_string_lossy(), "/usr/local/cuda");
        
        #[cfg(target_os = "windows")]
        assert_eq!(path.to_string_lossy(), "C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA");
    }

    #[test]
    fn test_linux_distro_variants() {
        let ubuntu = LinuxDistro::Ubuntu("20.04".to_string());
        let debian = LinuxDistro::Debian("11".to_string());
        let centos = LinuxDistro::CentOS("8".to_string());
        
        assert!(matches!(ubuntu, LinuxDistro::Ubuntu(_)));
        assert!(matches!(debian, LinuxDistro::Debian(_)));
        assert!(matches!(centos, LinuxDistro::CentOS(_)));
    }

    #[test]
    fn test_package_manager_detection() {
        let content = r#"NAME="CentOS Linux"
VERSION="8"
ID="centos"
ID_LIKE="rhel fedora"
VERSION_ID="8"
PLATFORM_ID="platform:el8"
PRETTY_NAME="CentOS Linux 8"
ANSI_COLOR="0;31"
CPE_NAME="cpe:/o:centos:centos:8"
HOME_URL="https://www.centos.org/"
BUG_REPORT_URL="https://bugs.centos.org/"
CENTOS_MANTISBT_PROJECT="CentOS-8"
CENTOS_MANTISBT_PROJECT_VERSION="8"
REDHAT_SUPPORT_PRODUCT="centos"
REDHAT_SUPPORT_PRODUCT_VERSION="8""#;

        let distro = DistroInfo::parse_os_release(content).unwrap();
        assert!(matches!(distro.package_manager, PackageManager::Yum));
        assert!(matches!(distro.os_type, OsType::Linux(LinuxDistro::CentOS(_))));
    }

    #[test]
    fn test_system_checker_basic() {
        let _checker = DefaultSystemChecker;
        // Basic test that the checker can be created
        assert!(true);
    }
}
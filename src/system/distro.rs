use serde::{Deserialize, Serialize};

/// Operating system type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OsType {
    Linux(LinuxDistro),
    Windows(WindowsVersion),
}

/// Linux distribution variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinuxDistro {
    Ubuntu(String),
    Debian(String),
    CentOS(String),
    Fedora(String),
    Arch(String),
    SUSE(String),
    Generic(String),
}

/// Windows version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsVersion {
    pub version: String,
    pub build: String,
}

/// Package manager type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageManager {
    Apt,
    Yum,
    Dnf,
    Pacman,
    Zypper,
    Chocolatey,
    Winget,
    Unknown,
}

/// Distribution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroInfo {
    pub os_type: OsType,
    pub name: String,
    pub version: String,
    pub kernel_version: Option<String>,
    pub package_manager: PackageManager,
}

impl DistroInfo {
    /// Create a new DistroInfo instance
    pub fn new(
        os_type: OsType,
        name: String,
        version: String,
        kernel_version: Option<String>,
        package_manager: PackageManager,
    ) -> Self {
        Self {
            os_type,
            name,
            version,
            kernel_version,
            package_manager,
        }
    }
}

/// Platform-specific handler trait
pub trait PlatformHandler {
    fn get_install_command(&self, package: &str) -> Vec<String>;
    fn get_cuda_install_path(&self) -> std::path::PathBuf;
    fn get_driver_install_command(&self, version: &str) -> Vec<String>;
    fn requires_sudo(&self) -> bool;
}
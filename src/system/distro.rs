use serde::{Deserialize, Serialize};

/// Operating system type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OsType {
    Linux(LinuxDistro),
    Windows(WindowsVersion),
}

/// Linux distribution variants
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl DistroInfo {
    pub fn detect() -> crate::error::CudaMgrResult<Self> {
        #[cfg(target_os = "linux")]
        {
            Self::detect_linux()
        }
        #[cfg(target_os = "windows")]
        {
            Self::detect_windows()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err(crate::error::SystemError::DistroDetection(
                "Unsupported operating system".to_string()
            ).into())
        }
    }

    #[cfg(target_os = "linux")]
    fn detect_linux() -> crate::error::CudaMgrResult<Self> {
        use std::fs;
        
        let kernel_version = Self::detect_kernel_version();
        
        // Try /etc/os-release first
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            let mut distro = Self::parse_os_release(&content)?;
            distro.kernel_version = kernel_version;
            return Ok(distro);
        }
        
        // Fallback to /etc/lsb-release
        if let Ok(content) = fs::read_to_string("/etc/lsb-release") {
            let mut distro = Self::parse_lsb_release(&content)?;
            distro.kernel_version = kernel_version;
            return Ok(distro);
        }
        
        // Last resort: generic Linux
        Ok(Self::new(
            OsType::Linux(LinuxDistro::Generic("unknown".to_string())),
            "Linux".to_string(),
            "unknown".to_string(),
            kernel_version,
            PackageManager::Unknown,
        ))
    }

    #[cfg(target_os = "windows")]
    fn detect_windows() -> crate::error::CudaMgrResult<Self> {
        // Basic Windows detection - can be enhanced with WinAPI calls
        Ok(Self::new(
            OsType::Windows(WindowsVersion {
                version: "10".to_string(),
                build: "unknown".to_string(),
            }),
            "Windows".to_string(),
            "10".to_string(),
            None,
            PackageManager::Winget,
        ))
    }

    pub fn parse_os_release(content: &str) -> crate::error::CudaMgrResult<Self> {
        let mut name = String::new();
        let mut version = String::new();
        let mut id = String::new();
        
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let value = value.trim_matches('"');
                match key {
                    "NAME" => name = value.to_string(),
                    "VERSION" => version = value.to_string(),
                    "ID" => id = value.to_string(),
                    _ => {}
                }
            }
        }
        
        let (distro, pkg_mgr) = match id.as_str() {
            "ubuntu" => (LinuxDistro::Ubuntu(version.clone()), PackageManager::Apt),
            "debian" => (LinuxDistro::Debian(version.clone()), PackageManager::Apt),
            "centos" => (LinuxDistro::CentOS(version.clone()), PackageManager::Yum),
            "fedora" => (LinuxDistro::Fedora(version.clone()), PackageManager::Dnf),
            "arch" => (LinuxDistro::Arch(version.clone()), PackageManager::Pacman),
            "opensuse" | "suse" => (LinuxDistro::SUSE(version.clone()), PackageManager::Zypper),
            _ => (LinuxDistro::Generic(id), PackageManager::Unknown),
        };
        
        Ok(Self::new(
            OsType::Linux(distro),
            name,
            version,
            None,
            pkg_mgr,
        ))
    }

    pub fn parse_lsb_release(content: &str) -> crate::error::CudaMgrResult<Self> {
        let mut name = String::new();
        let mut version = String::new();
        
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "DISTRIB_DESCRIPTION" => name = value.trim_matches('"').to_string(),
                    "DISTRIB_RELEASE" => version = value.to_string(),
                    _ => {}
                }
            }
        }
        
        Ok(Self::new(
            OsType::Linux(LinuxDistro::Generic(name.clone())),
            name,
            version,
            None,
            PackageManager::Unknown,
        ))
    }

    /// Detect kernel version
    #[cfg(target_os = "linux")]
    fn detect_kernel_version() -> Option<String> {
        use std::process::Command;
        
        Command::new("uname")
            .arg("-r")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|version| version.trim().to_string())
            .filter(|version| !version.is_empty())
    }

    #[cfg(target_os = "windows")]
    fn detect_kernel_version() -> Option<String> {
        use std::process::Command;
        
        Command::new("ver")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|version| version.trim().to_string())
            .filter(|version| !version.is_empty())
    }
}
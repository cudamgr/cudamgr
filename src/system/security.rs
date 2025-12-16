use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::error::CudaMgrResult;
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
use crate::error::SystemError;

/// Security and permission information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub secure_boot_enabled: bool,
    pub has_admin_privileges: bool,
    pub can_install_drivers: bool,
    pub uefi_mode: bool,
    pub secure_boot_details: Option<SecureBootInfo>,
    pub path_configuration: PathConfigInfo,
}

/// Detailed Secure Boot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureBootInfo {
    pub enabled: bool,
    pub setup_mode: bool,
    pub vendor_keys: bool,
    pub platform_key_present: bool,
}

/// PATH configuration validation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfigInfo {
    pub cuda_in_path: bool,
    pub conflicting_cuda_paths: Vec<std::path::PathBuf>,
    pub path_entries: Vec<std::path::PathBuf>,
    pub cuda_home_set: bool,
    pub cuda_home_path: Option<std::path::PathBuf>,
}

impl SecurityInfo {
    /// Create a new SecurityInfo instance
    pub fn new(
        secure_boot_enabled: bool,
        has_admin_privileges: bool,
        can_install_drivers: bool,
        uefi_mode: bool,
        secure_boot_details: Option<SecureBootInfo>,
        path_configuration: PathConfigInfo,
    ) -> Self {
        Self {
            secure_boot_enabled,
            has_admin_privileges,
            can_install_drivers,
            uefi_mode,
            secure_boot_details,
            path_configuration,
        }
    }

    /// Detect security information
    pub fn detect() -> CudaMgrResult<Self> {
        let uefi_mode = Self::detect_uefi_mode();
        let secure_boot_details = if uefi_mode {
            Self::detect_secure_boot_details().ok()
        } else {
            None
        };
        let secure_boot_enabled = secure_boot_details
            .as_ref()
            .map(|sb| sb.enabled)
            .unwrap_or(false);
        
        let has_admin_privileges = Self::detect_admin_privileges();
        let can_install_drivers = has_admin_privileges && Self::can_install_drivers_check();
        let path_configuration = Self::detect_path_configuration()?;

        Ok(Self::new(
            secure_boot_enabled,
            has_admin_privileges,
            can_install_drivers,
            uefi_mode,
            secure_boot_details,
            path_configuration,
        ))
    }

    /// Detect detailed Secure Boot information for UEFI systems
    fn detect_secure_boot_details() -> CudaMgrResult<SecureBootInfo> {
        #[cfg(target_os = "linux")]
        {
            let secure_boot_enabled = Self::read_efi_var("SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c")
                .map(|data| data.get(4).copied().unwrap_or(0) == 1)
                .unwrap_or(false);
            
            let setup_mode = Self::read_efi_var("SetupMode-8be4df61-93ca-11d2-aa0d-00e098032b8c")
                .map(|data| data.get(4).copied().unwrap_or(0) == 1)
                .unwrap_or(false);
            
            let vendor_keys = Self::read_efi_var("VendorKeys-8be4df61-93ca-11d2-aa0d-00e098032b8c")
                .map(|data| data.get(4).copied().unwrap_or(0) == 1)
                .unwrap_or(false);
            
            let platform_key_present = Path::new("/sys/firmware/efi/efivars/PK-8be4df61-93ca-11d2-aa0d-00e098032b8c").exists();

            Ok(SecureBootInfo {
                enabled: secure_boot_enabled,
                setup_mode,
                vendor_keys,
                platform_key_present,
            })
        }
        #[cfg(target_os = "windows")]
        {
            use winreg::enums::*;
            use winreg::RegKey;

            // Check Secure Boot state from Registry
            // Key: HKLM\SYSTEM\CurrentControlSet\Control\SecureBoot\State
            // Value: UEFISecureBootEnabled (DWORD)
            // 0 = Disabled, 1 = Enabled

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let sb_key = hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\SecureBoot\\State");
            
            let enabled = if let Ok(key) = sb_key {
                key.get_value::<u32, _>("UEFISecureBootEnabled").unwrap_or(0) == 1
            } else {
                false
            };

            // SetupMode and other details are harder to get without extensive API usage
            // defaulting to safe assumptions
            Ok(SecureBootInfo {
                enabled,
                setup_mode: false,
                vendor_keys: true, // Standard assumption for Windows
                platform_key_present: true,
            })
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err(SystemError::UnsupportedPlatform("Secure Boot detection not supported on this platform".to_string()).into())
        }
    }

    /// Read EFI variable data
    #[cfg(target_os = "linux")]
    fn read_efi_var(var_name: &str) -> Option<Vec<u8>> {
        let path = format!("/sys/firmware/efi/efivars/{}", var_name);
        fs::read(path).ok()
    }

    /// Detect if running with admin privileges
    fn detect_admin_privileges() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }
        #[cfg(target_os = "windows")]
        {
            // Check for elevated privileges using is_elevated crate
            is_elevated::is_elevated()
        }
        #[cfg(not(any(unix, target_os = "windows")))]
        {
            false
        }
    }

    /// Check if the system allows driver installation
    fn can_install_drivers_check() -> bool {
        #[cfg(target_os = "linux")]
        {
            // Check if we can access kernel module directories
            Path::new("/lib/modules").exists() && 
            Path::new("/sys/module").exists()
        }
        #[cfg(target_os = "windows")]
        {
            // On Windows, admin privileges are typically sufficient
            true
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }

    /// Detect if system is running in UEFI mode
    fn detect_uefi_mode() -> bool {
        #[cfg(target_os = "linux")]
        {
            Path::new("/sys/firmware/efi").exists()
        }
        #[cfg(target_os = "windows")]
        {
            // Check FIRMWARE_TYPE environment variable (often present)
            // Or try to access UEFI-only registry keys
            if let Ok(fw_type) = std::env::var("FIRMWARE_TYPE") {
                return fw_type.to_uppercase() == "UEFI";
            }
            
            // Fallback: Check if SecureBoot registry key exists (only on UEFI)
            use winreg::enums::*;
            use winreg::RegKey;
            
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\SecureBoot\\State").is_ok()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }

    /// Detect PATH configuration and CUDA-related environment variables
    fn detect_path_configuration() -> CudaMgrResult<PathConfigInfo> {
        let path_env = std::env::var("PATH").unwrap_or_default();
        let path_entries: Vec<std::path::PathBuf> = path_env
            .split(if cfg!(windows) { ';' } else { ':' })
            .map(std::path::PathBuf::from)
            .collect();

        let cuda_home_path = std::env::var("CUDA_HOME")
            .or_else(|_| std::env::var("CUDA_PATH"))
            .ok()
            .map(std::path::PathBuf::from);
        
        let cuda_home_set = cuda_home_path.is_some();

        // Look for CUDA-related paths in PATH
        let mut cuda_in_path = false;
        let mut conflicting_cuda_paths = Vec::new();

        for path_entry in &path_entries {
            if Self::is_cuda_path(path_entry) {
                cuda_in_path = true;
                
                // Check if this is a different CUDA installation
                if let Some(ref cuda_home) = cuda_home_path {
                    if !path_entry.starts_with(cuda_home) {
                        conflicting_cuda_paths.push(path_entry.clone());
                    }
                } else {
                    // No CUDA_HOME set, but CUDA found in PATH - potential conflict
                    conflicting_cuda_paths.push(path_entry.clone());
                }
            }
        }

        Ok(PathConfigInfo {
            cuda_in_path,
            conflicting_cuda_paths,
            path_entries,
            cuda_home_set,
            cuda_home_path,
        })
    }

    /// Check if a path entry appears to be CUDA-related
    fn is_cuda_path(path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.contains("cuda") || 
        path_str.contains("nvcc") ||
        (path.join("nvcc").exists() || path.join("nvcc.exe").exists())
    }

    /// Check if system allows driver installation
    pub fn allows_driver_installation(&self) -> bool {
        self.has_admin_privileges && self.can_install_drivers
    }

    /// Check if there are PATH configuration issues
    pub fn has_path_conflicts(&self) -> bool {
        !self.path_configuration.conflicting_cuda_paths.is_empty()
    }

    /// Get a summary of security issues
    pub fn get_security_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if !self.has_admin_privileges {
            issues.push("Administrator/root privileges required for CUDA installation".to_string());
        }

        if !self.can_install_drivers {
            issues.push("Cannot install drivers on this system".to_string());
        }

        if self.secure_boot_enabled {
            issues.push("Secure Boot is enabled - may prevent unsigned driver installation".to_string());
        }

        if self.has_path_conflicts() {
            issues.push(format!(
                "Conflicting CUDA paths found in PATH: {}",
                self.path_configuration.conflicting_cuda_paths.len()
            ));
        }

        if !self.path_configuration.cuda_home_set && self.path_configuration.cuda_in_path {
            issues.push("CUDA found in PATH but CUDA_HOME not set".to_string());
        }

        issues
    }
}

impl SecureBootInfo {
    /// Check if Secure Boot configuration allows driver installation
    pub fn allows_driver_installation(&self) -> bool {
        !self.enabled || self.setup_mode
    }

    /// Get a description of the Secure Boot state
    pub fn get_status_description(&self) -> String {
        if !self.enabled {
            "Disabled".to_string()
        } else if self.setup_mode {
            "Enabled (Setup Mode)".to_string()
        } else {
            "Enabled (User Mode)".to_string()
        }
    }
}

impl PathConfigInfo {
    /// Check if PATH configuration is optimal for CUDA
    pub fn is_optimal(&self) -> bool {
        self.cuda_home_set && 
        self.conflicting_cuda_paths.is_empty() &&
        (self.cuda_in_path || !self.cuda_home_set)
    }

    /// Get recommendations for PATH configuration
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !self.cuda_home_set {
            recommendations.push("Set CUDA_HOME environment variable".to_string());
        }

        if !self.conflicting_cuda_paths.is_empty() {
            recommendations.push("Remove conflicting CUDA paths from PATH".to_string());
        }

        if self.cuda_home_set && !self.cuda_in_path {
            recommendations.push("Add CUDA bin directory to PATH".to_string());
        }

        recommendations
    }
}
use serde::{Deserialize, Serialize};
use crate::error::CudaMgrResult;

/// Security and permission information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub secure_boot_enabled: bool,
    pub has_admin_privileges: bool,
    pub can_install_drivers: bool,
    pub uefi_mode: bool,
}

impl SecurityInfo {
    /// Create a new SecurityInfo instance
    pub fn new(
        secure_boot_enabled: bool,
        has_admin_privileges: bool,
        can_install_drivers: bool,
        uefi_mode: bool,
    ) -> Self {
        Self {
            secure_boot_enabled,
            has_admin_privileges,
            can_install_drivers,
            uefi_mode,
        }
    }

    /// Detect security information
    pub fn detect() -> CudaMgrResult<Self> {
        let secure_boot_enabled = Self::detect_secure_boot();
        let has_admin_privileges = Self::detect_admin_privileges();
        let can_install_drivers = has_admin_privileges; // Simplified assumption
        let uefi_mode = Self::detect_uefi_mode();

        Ok(Self::new(
            secure_boot_enabled,
            has_admin_privileges,
            can_install_drivers,
            uefi_mode,
        ))
    }

    /// Detect if Secure Boot is enabled
    fn detect_secure_boot() -> bool {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c")
                .ok()
                .and_then(|content| content.chars().nth(4))
                .map(|c| c == '1')
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "linux"))]
        {
            false // Simplified for other platforms
        }
    }

    /// Detect if running with admin privileges
    fn detect_admin_privileges() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }
        #[cfg(windows)]
        {
            // Simplified check - in real implementation would use Windows APIs
            false
        }
    }

    /// Detect if system is running in UEFI mode
    fn detect_uefi_mode() -> bool {
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/sys/firmware/efi").exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false // Simplified for other platforms
        }
    }

    /// Check if system allows driver installation
    pub fn allows_driver_installation(&self) -> bool {
        self.has_admin_privileges && self.can_install_drivers
    }
}
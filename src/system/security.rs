use serde::{Deserialize, Serialize};

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

    /// Check if system allows driver installation
    pub fn allows_driver_installation(&self) -> bool {
        self.has_admin_privileges && self.can_install_drivers
    }
}
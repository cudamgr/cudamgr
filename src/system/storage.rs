use serde::{Deserialize, Serialize};

/// Storage and disk space information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub available_space_gb: u64,
    pub total_space_gb: u64,
    pub install_path: String,
    pub has_sufficient_space: bool,
}

impl StorageInfo {
    /// Create a new StorageInfo instance
    pub fn new(
        available_space_gb: u64,
        total_space_gb: u64,
        install_path: String,
        required_space_gb: u64,
    ) -> Self {
        Self {
            available_space_gb,
            total_space_gb,
            install_path,
            has_sufficient_space: available_space_gb >= required_space_gb,
        }
    }

    /// Check if there's enough space for installation
    pub fn check_space_requirement(&self, required_gb: u64) -> bool {
        self.available_space_gb >= required_gb
    }
}
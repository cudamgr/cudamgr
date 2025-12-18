use crate::error::CudaMgrResult;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WslVersion {
    Wsl1,
    Wsl2,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslInfo {
    pub is_wsl: bool,
    pub version: WslVersion,
    pub distribution: String,
}

impl WslInfo {
    pub fn detect() -> CudaMgrResult<Self> {
        let mut version = WslVersion::None;
        let mut distribution = String::new();
        let mut is_wsl = false;

        // 1. Check /proc/version for "microsoft" or "WSL"
        if let Ok(content) = fs::read_to_string("/proc/version") {
            let content_lower = content.to_lowercase();
            if content_lower.contains("microsoft") {
                is_wsl = true;
                if content_lower.contains("wsl2") {
                    version = WslVersion::Wsl2;
                } else {
                    version = WslVersion::Wsl1;
                }
            }
        }

        // 2. Check environment variable WSL_DISTRO_NAME
        if let Ok(distro) = env::var("WSL_DISTRO_NAME") {
            is_wsl = true;
            distribution = distro;

            // If we haven't determined version yet, default to WSL2 as it's modern standard
            if version == WslVersion::None {
                version = WslVersion::Wsl2;
            }
        }

        Ok(Self {
            is_wsl,
            version,
            distribution,
        })
    }
}

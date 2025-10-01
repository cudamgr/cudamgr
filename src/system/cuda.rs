use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CudaInstallation {
    pub version: String,
    pub install_path: PathBuf,
    pub toolkit_path: PathBuf,
    pub runtime_version: Option<String>,
    pub driver_version: Option<String>,
    pub install_date: DateTime<Utc>,
    pub size_bytes: u64,
    pub is_active: bool,
    pub components: Vec<CudaComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CudaComponent {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CudaVersion {
    Specific(String),
    Latest,
    LatestLts,
}

impl CudaInstallation {
    pub fn new(version: String, install_path: PathBuf) -> Self {
        let toolkit_path = install_path.join("bin");
        
        Self {
            version,
            install_path,
            toolkit_path,
            runtime_version: None,
            driver_version: None,
            install_date: Utc::now(),
            size_bytes: 0,
            is_active: false,
            components: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.install_path.exists() && 
        self.toolkit_path.exists() &&
        !self.components.iter().filter(|c| c.required).any(|c| !c.path.exists())
    }

    pub fn get_nvcc_path(&self) -> PathBuf {
        self.toolkit_path.join("nvcc")
    }

    pub fn get_lib_path(&self) -> PathBuf {
        self.install_path.join("lib64")
    }
}

impl std::fmt::Display for CudaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CudaVersion::Specific(v) => write!(f, "{}", v),
            CudaVersion::Latest => write!(f, "latest"),
            CudaVersion::LatestLts => write!(f, "latest-lts"),
        }
    }
}
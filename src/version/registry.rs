use crate::error::{CudaMgrResult, VersionError};
use crate::version::VersionInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Version registry for tracking installed CUDA versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRegistry {
    pub versions: Vec<VersionInfo>,
    pub active_version: Option<String>,
    pub registry_path: PathBuf,
}

impl VersionRegistry {
    /// Create a new version registry
    pub fn new(registry_path: PathBuf) -> Self {
        Self {
            versions: Vec::new(),
            active_version: None,
            registry_path,
        }
    }

    /// Load registry from disk
    pub async fn load(&mut self) -> CudaMgrResult<()> {
        // TODO: Implement registry loading
        tracing::info!("Loading version registry from {:?}", self.registry_path);
        Err(VersionError::Registry("Registry loading not yet implemented".to_string()).into())
    }

    /// Save registry to disk
    pub async fn save(&self) -> CudaMgrResult<()> {
        // TODO: Implement registry saving
        tracing::info!("Saving version registry to {:?}", self.registry_path);
        Err(VersionError::Registry("Registry saving not yet implemented".to_string()).into())
    }

    /// Add a new version to the registry
    pub fn add_version(&mut self, version_info: VersionInfo) {
        self.versions.push(version_info);
    }

    /// Remove a version from the registry
    pub fn remove_version(&mut self, version: &str) -> CudaMgrResult<()> {
        let initial_len = self.versions.len();
        self.versions.retain(|v| v.version != version);

        if self.versions.len() == initial_len {
            return Err(VersionError::NotFound(format!(
                "Version {} not found in registry",
                version
            ))
            .into());
        }

        // If we removed the active version, clear it
        if self.active_version.as_ref() == Some(&version.to_string()) {
            self.active_version = None;
        }

        Ok(())
    }

    /// Set the active version
    pub fn set_active_version(&mut self, version: &str) -> CudaMgrResult<()> {
        // Verify the version exists
        if !self.versions.iter().any(|v| v.version == version) {
            return Err(VersionError::NotFound(format!(
                "Version {} not found in registry",
                version
            ))
            .into());
        }

        // Update active flags
        for v in &mut self.versions {
            v.is_active = v.version == version;
        }

        self.active_version = Some(version.to_string());
        Ok(())
    }

    /// Get version information
    pub fn get_version(&self, version: &str) -> Option<&VersionInfo> {
        self.versions.iter().find(|v| v.version == version)
    }
}

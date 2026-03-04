use crate::error::{CudaMgrError, CudaMgrResult, VersionError};
use crate::version::VersionInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File format for persistence (we do not persist registry_path)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionRegistryFile {
    versions: Vec<VersionInfo>,
    active_version: Option<String>,
}

/// Version registry for tracking installed CUDA versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRegistry {
    pub versions: Vec<VersionInfo>,
    pub active_version: Option<String>,
    #[serde(skip)]
    pub registry_path: PathBuf,
}

impl VersionRegistry {
    /// Path to the default registry file (platform-specific cudamgr data dir).
    pub fn default_path() -> PathBuf {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
        base.join("cudamgr").join("registry.json")
    }

    /// Create a new version registry
    pub fn new(registry_path: PathBuf) -> Self {
        Self {
            versions: Vec::new(),
            active_version: None,
            registry_path,
        }
    }

    /// Load registry from disk. Creates an empty registry if the file does not exist.
    pub async fn load(&mut self) -> CudaMgrResult<()> {
        let path = &self.registry_path;
        tracing::debug!("Loading version registry from {:?}", path);

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!("No registry file at {:?}, starting empty", path);
                return Ok(());
            }
            Err(e) => {
                return Err(CudaMgrError::Version(VersionError::Registry(format!(
                    "Failed to read registry: {}",
                    e
                ))));
            }
        };

        let file: VersionRegistryFile = serde_json::from_str(&content).map_err(|e| {
            CudaMgrError::Version(VersionError::Registry(format!("Invalid registry format: {}", e)))
        })?;

        self.versions = file.versions;
        self.active_version = file.active_version;
        Ok(())
    }

    /// Save registry to disk. Creates parent directories if needed.
    pub async fn save(&self) -> CudaMgrResult<()> {
        let path = &self.registry_path;
        tracing::debug!("Saving version registry to {:?}", path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CudaMgrError::Version(VersionError::Registry(format!(
                    "Failed to create registry dir: {}",
                    e
                )))
            })?;
        }

        let file = VersionRegistryFile {
            versions: self.versions.clone(),
            active_version: self.active_version.clone(),
        };
        let content = serde_json::to_string_pretty(&file).map_err(|e| {
            CudaMgrError::Version(VersionError::Registry(format!(
                "Failed to serialize registry: {}",
                e
            )))
        })?;
        std::fs::write(path, content).map_err(|e| {
            CudaMgrError::Version(VersionError::Registry(format!(
                "Failed to write registry: {}",
                e
            )))
        })?;
        Ok(())
    }

    /// Load registry from default path, or create an empty one if the file does not exist.
    pub async fn load_or_create() -> CudaMgrResult<Self> {
        let path = Self::default_path();
        let mut reg = Self::new(path);
        reg.load().await?;
        Ok(reg)
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

    /// Get version information (exact match)
    pub fn get_version(&self, version: &str) -> Option<&VersionInfo> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Find a version by exact match or prefix (e.g. "12.0" matches "12.0.1")
    pub fn find_version(&self, version: &str) -> Option<&VersionInfo> {
        self.versions.iter().find(|v| {
            v.version == version
                || v.version.starts_with(&format!("{}.", version))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_registry_save_and_load() {
        let dir = std::env::temp_dir().join("cudamgr_registry_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("registry.json");

        let mut reg = VersionRegistry::new(path.clone());
        reg.add_version(VersionInfo {
            version: "12.0.3".to_string(),
            install_path: PathBuf::from("/opt/cuda/12.0.3"),
            is_active: true,
            install_date: Utc::now(),
            size_bytes: 1_000_000,
        });
        reg.active_version = Some("12.0.3".to_string());
        reg.save().await.unwrap();

        let mut loaded = VersionRegistry::new(path);
        loaded.load().await.unwrap();
        assert_eq!(loaded.versions.len(), 1);
        assert_eq!(loaded.versions[0].version, "12.0.3");
        assert_eq!(loaded.active_version.as_deref(), Some("12.0.3"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_registry_load_missing_file() {
        let path = std::env::temp_dir().join("cudamgr_nonexistent_registry.json");
        let _ = std::fs::remove_file(&path);
        let mut reg = VersionRegistry::new(path);
        reg.load().await.unwrap();
        assert!(reg.versions.is_empty());
        assert!(reg.active_version.is_none());
    }
}

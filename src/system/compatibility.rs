use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a mapping between GPU architectures and their compute capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuArchitecture {
    pub name: String,
    pub architecture: String,
    pub compute_capability: (u32, u32),
    pub min_driver_version: Option<String>,
}

/// JSON-serializable compatibility registry
///
/// This is the schema for `compatibility.json`. It can be loaded from:
/// 1. Local cache (`~/.cudamgr/compatibility.json`)
/// 2. Remote URL (GitHub-hosted)
/// 3. Built-in defaults (compiled into the binary)
#[derive(Debug, Serialize, Deserialize)]
pub struct CompatibilityRegistry {
    /// Schema version for forward compatibility
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// When this data was last updated (ISO date)
    #[serde(default)]
    pub last_updated: Option<String>,

    /// Source URL for this data
    #[serde(default)]
    pub source: Option<String>,

    /// GPU model -> architecture/compute capability mapping
    pub gpu_architectures: HashMap<String, GpuArchitecture>,

    /// Driver major version -> max CUDA version (ordered newest first)
    pub driver_cuda_map: Vec<(String, String)>,
}

fn default_schema_version() -> u32 {
    1
}

impl Default for CompatibilityRegistry {
    fn default() -> Self {
        Self::default_builtin()
    }
}

impl CompatibilityRegistry {
    /// Create a new registry using the cascade: cache → built-in defaults.
    ///
    /// Remote fetching is NOT done here to avoid blocking at startup.
    /// Use `update_from_remote()` separately (e.g., in `doctor` or background task).
    pub fn new() -> Self {
        // 1. Try loading from local cache
        if let Some(cached) = Self::load_from_cache() {
            tracing::debug!("Loaded compatibility registry from cache");
            return cached;
        }

        // 2. Fall back to built-in defaults
        tracing::debug!("Using built-in compatibility registry defaults");
        Self::default_builtin()
    }

    // ── Loading methods ──────────────────────────────────────────────

    /// Load from a JSON file on disk
    pub fn load_from_file(path: &Path) -> Result<Self, RegistryError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| RegistryError::Io(e.to_string()))?;

        Self::load_from_str(&content)
    }

    /// Load from a JSON string (useful for testing and remote responses)
    pub fn load_from_str(json: &str) -> Result<Self, RegistryError> {
        let registry: Self =
            serde_json::from_str(json).map_err(|e| RegistryError::Parse(e.to_string()))?;

        // Validate schema version
        if registry.schema_version != 1 {
            return Err(RegistryError::UnsupportedSchema(registry.schema_version));
        }

        // Basic sanity check — must have at least some data
        if registry.gpu_architectures.is_empty() || registry.driver_cuda_map.is_empty() {
            return Err(RegistryError::Validation(
                "Registry has empty GPU or driver data".to_string(),
            ));
        }

        Ok(registry)
    }

    /// Load from remote URL (non-blocking, returns Result)
    pub async fn load_from_url(url: &str) -> Result<Self, RegistryError> {
        let response = reqwest::get(url)
            .await
            .map_err(|e| RegistryError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RegistryError::Network(format!(
                "HTTP {} from {}",
                response.status(),
                url
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| RegistryError::Network(e.to_string()))?;

        Self::load_from_str(&body)
    }

    // ── Cache management ─────────────────────────────────────────────

    /// Default remote registry URL (GitHub raw)
    pub fn default_remote_url() -> &'static str {
        "https://raw.githubusercontent.com/cudamgr/cudamgr/main/data/compatibility.json"
    }

    /// Platform-specific cache directory for the registry file
    pub fn cache_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            // %LOCALAPPDATA%\cudamgr\compatibility.json
            if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
                return PathBuf::from(local_app_data)
                    .join("cudamgr")
                    .join("compatibility.json");
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // ~/.local/share/cudamgr/compatibility.json
            if let Some(home) = dirs::home_dir() {
                return home
                    .join(".local")
                    .join("share")
                    .join("cudamgr")
                    .join("compatibility.json");
            }
        }

        // Fallback: ~/.cudamgr/compatibility.json
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cudamgr")
            .join("compatibility.json")
    }

    /// Try to load from the platform cache
    fn load_from_cache() -> Option<Self> {
        let path = Self::cache_path();
        if !path.exists() {
            return None;
        }

        // Check staleness: skip cache if older than 30 days
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                let age = std::time::SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or_default();
                if age > std::time::Duration::from_secs(30 * 24 * 60 * 60) {
                    tracing::info!("Cached compatibility registry is stale (>30 days), skipping");
                    return None;
                }
            }
        }

        match Self::load_from_file(&path) {
            Ok(registry) => Some(registry),
            Err(e) => {
                tracing::warn!("Failed to load cached registry: {}", e);
                None
            }
        }
    }

    /// Save registry to the local cache
    pub fn save_to_cache(&self) -> Result<(), RegistryError> {
        let path = Self::cache_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| RegistryError::Io(e.to_string()))?;
        }

        let content =
            serde_json::to_string_pretty(self).map_err(|e| RegistryError::Parse(e.to_string()))?;

        std::fs::write(&path, content).map_err(|e| RegistryError::Io(e.to_string()))?;

        tracing::info!("Saved compatibility registry to {}", path.display());
        Ok(())
    }

    /// Fetch the latest registry from remote and update local cache.
    /// Returns Ok(true) if updated, Ok(false) if already up-to-date, Err on failure.
    pub async fn update_from_remote(&mut self) -> Result<bool, RegistryError> {
        let url = Self::default_remote_url();
        tracing::info!("Fetching compatibility registry from {}", url);

        let remote = Self::load_from_url(url).await?;

        // Check if the remote data is newer
        let dominated = match (&self.last_updated, &remote.last_updated) {
            (Some(local), Some(remote_date)) => remote_date > local,
            (None, Some(_)) => true,
            _ => true, // If we can't compare, update anyway
        };

        if dominated {
            // Replace self with remote data
            self.schema_version = remote.schema_version;
            self.last_updated = remote.last_updated;
            self.source = remote.source;
            self.gpu_architectures = remote.gpu_architectures;
            self.driver_cuda_map = remote.driver_cuda_map;

            // Save to cache for next startup
            self.save_to_cache()?;

            Ok(true)
        } else {
            tracing::debug!("Remote registry is not newer than local, skipping update");
            Ok(false)
        }
    }

    // ── Lookup methods (unchanged) ───────────────────────────────────

    /// Lookup compute capability for a GPU model
    pub fn get_compute_capability(&self, model: &str) -> Option<(u32, u32)> {
        let model_lower = model.to_lowercase();

        // Exact match
        if let Some(info) = self.gpu_architectures.get(&model_lower) {
            return Some(info.compute_capability);
        }

        // Partial match (e.g., "NVIDIA GeForce RTX 3080" contains "rtx 3080")
        for (key, info) in &self.gpu_architectures {
            if model_lower.contains(key) {
                return Some(info.compute_capability);
            }
        }

        None
    }

    /// Estimate max CUDA version from driver version
    pub fn get_max_cuda_version(&self, driver_version: &str) -> Option<String> {
        // Parse the driver major version for range-based matching
        let driver_major: u32 = driver_version
            .split('.')
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        // Match against the map (ordered newest first, first match wins)
        for (prefix, cuda_ver) in &self.driver_cuda_map {
            let prefix_major: u32 = prefix.trim_end_matches('.').parse().unwrap_or(0);

            if driver_major >= prefix_major {
                return Some(cuda_ver.clone());
            }
        }

        // Fallback for very new drivers beyond our latest known entry
        if let Some((latest_prefix, latest_cuda)) = self.driver_cuda_map.first() {
            let latest_major: u32 = latest_prefix.trim_end_matches('.').parse().unwrap_or(0);
            if driver_major > latest_major {
                return Some(latest_cuda.clone());
            }
        }

        None
    }

    // ── Built-in defaults ────────────────────────────────────────────

    fn default_builtin() -> Self {
        // This is the compile-time fallback. It's the same data as
        // data/compatibility.json but embedded in the binary.
        //
        // To keep this in sync, regenerate from the JSON file when updating.

        let mut gpu_architectures = HashMap::new();

        // Helper macro to reduce boilerplate
        macro_rules! add_gpus {
            ($models:expr, $arch:expr, $cc:expr, $min_driver:expr) => {
                for model in $models {
                    gpu_architectures.insert(
                        model.to_string(),
                        GpuArchitecture {
                            name: model.to_string(),
                            architecture: $arch.to_string(),
                            compute_capability: $cc,
                            min_driver_version: $min_driver.map(|v: &str| v.to_string()),
                        },
                    );
                }
            };
        }

        // Consumer GPUs
        add_gpus!(
            [
                "rtx 5090",
                "rtx 5080",
                "rtx 5070 ti",
                "rtx 5070",
                "rtx 5060 ti",
                "rtx 5060"
            ],
            "Blackwell",
            (12, 0),
            Some("570.00")
        );
        add_gpus!(
            [
                "rtx 4090",
                "rtx 4080 super",
                "rtx 4080",
                "rtx 4070 ti super",
                "rtx 4070 ti",
                "rtx 4070 super",
                "rtx 4070",
                "rtx 4060 ti",
                "rtx 4060"
            ],
            "Ada Lovelace",
            (8, 9),
            Some("520.00")
        );
        add_gpus!(
            [
                "rtx 3090 ti",
                "rtx 3090",
                "rtx 3080 ti",
                "rtx 3080",
                "rtx 3070 ti",
                "rtx 3070",
                "rtx 3060 ti",
                "rtx 3060",
                "rtx 3050"
            ],
            "Ampere",
            (8, 6),
            Some("450.00")
        );
        add_gpus!(
            [
                "rtx 2080 ti",
                "rtx 2080 super",
                "rtx 2080",
                "rtx 2070 super",
                "rtx 2070",
                "rtx 2060 super",
                "rtx 2060",
                "quadro rtx 8000",
                "quadro rtx 6000",
                "quadro rtx 5000",
                "quadro rtx 4000"
            ],
            "Turing",
            (7, 5),
            Some("410.00")
        );
        add_gpus!(
            [
                "gtx 1660 ti",
                "gtx 1660 super",
                "gtx 1660",
                "gtx 1650 super",
                "gtx 1650",
                "gtx 1630"
            ],
            "Turing",
            (7, 5),
            Some("418.00")
        );
        add_gpus!(
            [
                "titan xp",
                "titan x",
                "gtx 1080 ti",
                "gtx 1080",
                "gtx 1070 ti",
                "gtx 1070",
                "gtx 1060",
                "gtx 1050 ti",
                "gtx 1050"
            ],
            "Pascal",
            (6, 1),
            Some("367.00")
        );
        add_gpus!(
            ["gtx 980 ti", "gtx 980", "gtx 970", "gtx 960", "gtx 950"],
            "Maxwell",
            (5, 2),
            Some("340.00")
        );

        // Datacenter / Professional GPUs
        add_gpus!(["b200", "b100"], "Blackwell", (12, 0), None::<&str>);
        add_gpus!(["h100", "h200"], "Hopper", (9, 0), None::<&str>);
        add_gpus!(
            ["l40s", "l40", "l4", "rtx 6000 ada"],
            "Ada Lovelace",
            (8, 9),
            None::<&str>
        );
        add_gpus!(["a100", "a30"], "Ampere", (8, 0), None::<&str>);
        add_gpus!(
            ["a10", "a16", "a2", "rtx a6000", "rtx a5000", "rtx a4000"],
            "Ampere",
            (8, 6),
            None::<&str>
        );
        add_gpus!(["t4", "quadro rtx"], "Turing", (7, 5), None::<&str>);
        add_gpus!(["tesla v100", "titan v"], "Volta", (7, 0), None::<&str>);
        add_gpus!(["tesla p100"], "Pascal", (6, 0), None::<&str>);
        add_gpus!(["tesla p40", "tesla p4"], "Pascal", (6, 1), None::<&str>);
        add_gpus!(["tesla k80"], "Kepler", (3, 7), None::<&str>);
        add_gpus!(["tesla k40"], "Kepler", (3, 5), None::<&str>);

        // Driver -> CUDA mapping (newest first)
        let driver_cuda_map = vec![
            ("590".to_string(), "13.1".to_string()),
            ("580".to_string(), "13.0".to_string()),
            ("575".to_string(), "12.9".to_string()),
            ("570".to_string(), "12.8".to_string()),
            ("560".to_string(), "12.6".to_string()),
            ("555".to_string(), "12.5".to_string()),
            ("550".to_string(), "12.4".to_string()),
            ("545".to_string(), "12.3".to_string()),
            ("535".to_string(), "12.2".to_string()),
            ("530".to_string(), "12.1".to_string()),
            ("525".to_string(), "12.0".to_string()),
            ("520".to_string(), "11.8".to_string()),
            ("515".to_string(), "11.7".to_string()),
            ("510".to_string(), "11.6".to_string()),
            ("495".to_string(), "11.5".to_string()),
            ("470".to_string(), "11.4".to_string()),
            ("465".to_string(), "11.3".to_string()),
            ("460".to_string(), "11.2".to_string()),
            ("455".to_string(), "11.1".to_string()),
            ("450".to_string(), "11.0".to_string()),
            ("440".to_string(), "10.2".to_string()),
            ("418".to_string(), "10.1".to_string()),
            ("410".to_string(), "10.0".to_string()),
            ("396".to_string(), "9.2".to_string()),
            ("390".to_string(), "9.1".to_string()),
            ("384".to_string(), "9.0".to_string()),
        ];

        Self {
            schema_version: 1,
            last_updated: Some("2026-02-21".to_string()),
            source: Some("built-in".to_string()),
            gpu_architectures,
            driver_cuda_map,
        }
    }
}

// ── Error type ───────────────────────────────────────────────────

/// Errors specific to registry loading/updating
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("JSON parse error: {0}")]
    Parse(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Unsupported schema version: {0}")]
    UnsupportedSchema(u32),

    #[error("Validation error: {0}")]
    Validation(String),
}

// ── Global instance ──────────────────────────────────────────────

/// Global registry instance, initialized once at first access.
/// Uses cache → built-in fallback cascade (no network at startup).
pub static REGISTRY: Lazy<CompatibilityRegistry> = Lazy::new(CompatibilityRegistry::new);

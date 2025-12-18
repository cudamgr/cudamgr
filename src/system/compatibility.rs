use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a mapping between GPU architectures and their compute capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuArchitecture {
    pub name: String,
    pub architecture: String,
    pub compute_capability: (u32, u32),
    pub min_driver_version: Option<String>,
}

/// Central registry for compatibility data
pub struct CompatibilityRegistry {
    pub gpu_architectures: HashMap<String, GpuArchitecture>,
    pub driver_cuda_map: Vec<(String, String)>, // (Driver Version Prefix, Max CUDA Version)
}

impl CompatibilityRegistry {
    pub fn new() -> Self {
        // In a future version, this could load from a JSON file
        Self::default_builtin()
    }

    fn default_builtin() -> Self {
        let mut gpu_architectures = HashMap::new();

        // Add GPUs and their capabilities
        // RTX 40 series (Ada Lovelace)
        for model in ["rtx 4090", "rtx 4080", "rtx 4070", "rtx 4060"] {
            gpu_architectures.insert(
                model.to_string(),
                GpuArchitecture {
                    name: model.to_string(),
                    architecture: "Ada Lovelace".to_string(),
                    compute_capability: (8, 9),
                    min_driver_version: Some("520.00".to_string()),
                },
            );
        }

        // RTX 30 series (Ampere)
        for model in ["rtx 3090", "rtx 3080", "rtx 3070", "rtx 3060", "rtx 3050"] {
            gpu_architectures.insert(
                model.to_string(),
                GpuArchitecture {
                    name: model.to_string(),
                    architecture: "Ampere".to_string(),
                    compute_capability: (8, 6),
                    min_driver_version: Some("450.00".to_string()),
                },
            );
        }

        // RTX 20 series (Turing)
        for model in ["rtx 2080", "rtx 2070", "rtx 2060", "quadro rtx"] {
            gpu_architectures.insert(
                model.to_string(),
                GpuArchitecture {
                    name: model.to_string(),
                    architecture: "Turing".to_string(),
                    compute_capability: (7, 5),
                    min_driver_version: Some("410.00".to_string()),
                },
            );
        }

        // GTX 16 series (Turing)
        for model in ["gtx 1660", "gtx 1650", "gtx 1630"] {
            gpu_architectures.insert(
                model.to_string(),
                GpuArchitecture {
                    name: model.to_string(),
                    architecture: "Turing".to_string(),
                    compute_capability: (7, 5),
                    min_driver_version: Some("418.00".to_string()),
                },
            );
        }

        // GTX 10 series (Pascal)
        for model in ["gtx 1080", "gtx 1070", "gtx 1060", "gtx 1050", "titan xp"] {
            gpu_architectures.insert(
                model.to_string(),
                GpuArchitecture {
                    name: model.to_string(),
                    architecture: "Pascal".to_string(),
                    compute_capability: (6, 1),
                    min_driver_version: Some("367.00".to_string()),
                },
            );
        }

        // Tesla/Datacenter keys
        gpu_architectures.insert(
            "tesla v100".to_string(),
            GpuArchitecture {
                name: "Tesla V100".to_string(),
                architecture: "Volta".to_string(),
                compute_capability: (7, 0),
                min_driver_version: None,
            },
        );

        // Driver Version -> Max CUDA Version Mapping
        // Ordered from newest to oldest
        let driver_cuda_map = vec![
            ("570.".to_string(), "12.8".to_string()),
            ("56".to_string(), "12.6".to_string()), // 560-569
            ("55".to_string(), "12.4".to_string()), // 550-559
            ("545.".to_string(), "12.3".to_string()),
            ("535.".to_string(), "12.2".to_string()),
            ("53".to_string(), "12.1".to_string()), // 530-534
            ("525.".to_string(), "12.0".to_string()), // 525.x -> 12.0
            ("527.".to_string(), "12.0".to_string()), // 527.x -> 12.0
            ("52".to_string(), "11.8".to_string()), // 520-524 -> 11.8
            ("51".to_string(), "11.7".to_string()), // 515-519
            ("47".to_string(), "11.4".to_string()), // 470
            ("46".to_string(), "11.2".to_string()),
            ("45".to_string(), "11.0".to_string()),
        ];

        Self {
            gpu_architectures,
            driver_cuda_map,
        }
    }

    /// Lookup compute capability for a GPU model
    pub fn get_compute_capability(&self, model: &str) -> Option<(u32, u32)> {
        let model_lower = model.to_lowercase();

        // Exact match
        if let Some(info) = self.gpu_architectures.get(&model_lower) {
            return Some(info.compute_capability);
        }

        // Partial match
        for (key, info) in &self.gpu_architectures {
            if model_lower.contains(key) {
                return Some(info.compute_capability);
            }
        }

        None
    }

    /// Estimate max CUDA version from driver version
    pub fn get_max_cuda_version(&self, driver_version: &str) -> Option<String> {
        for (prefix, cuda_ver) in &self.driver_cuda_map {
            if driver_version.starts_with(prefix) {
                return Some(cuda_ver.clone());
            }
        }

        // Fallback for very new drivers (assume latest known)
        if let Ok(major) = driver_version
            .split('.')
            .next()
            .unwrap_or("0")
            .parse::<i32>()
        {
            if major > 570 {
                return Some("12.8+".to_string());
            }
        }

        None
    }
}

// Global instance nicely wrapped
pub static REGISTRY: Lazy<CompatibilityRegistry> = Lazy::new(CompatibilityRegistry::new);

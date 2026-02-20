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

impl Default for CompatibilityRegistry {
    fn default() -> Self {
        Self::default_builtin()
    }
}

impl CompatibilityRegistry {
    pub fn new() -> Self {
        // In a future version, this could load from a JSON file
        Self::default_builtin()
    }

    fn default_builtin() -> Self {
        let mut gpu_architectures = HashMap::new();

        // GPU Architecture -> Compute Capability mapping (FALLBACK ONLY)
        //
        // Primary source: nvidia-smi --query-gpu=compute_cap gives exact values.
        // This map is only used when nvidia-smi is unavailable (lspci/wmic detection).
        //
        // Source: https://developer.nvidia.com/cuda-gpus

        // RTX 50 series (Blackwell)
        for model in [
            "rtx 5090", "rtx 5080", "rtx 5070 ti", "rtx 5070", "rtx 5060 ti", "rtx 5060",
        ] {
            gpu_architectures.insert(model.to_string(), GpuArchitecture {
                name: model.to_string(),
                architecture: "Blackwell".to_string(),
                compute_capability: (12, 0),
                min_driver_version: Some("570.00".to_string()),
            });
        }

        // RTX 40 series (Ada Lovelace)
        for model in [
            "rtx 4090", "rtx 4080 super", "rtx 4080", "rtx 4070 ti super", "rtx 4070 ti",
            "rtx 4070 super", "rtx 4070", "rtx 4060 ti", "rtx 4060",
        ] {
            gpu_architectures.insert(model.to_string(), GpuArchitecture {
                name: model.to_string(),
                architecture: "Ada Lovelace".to_string(),
                compute_capability: (8, 9),
                min_driver_version: Some("520.00".to_string()),
            });
        }

        // RTX 30 series (Ampere)
        for model in [
            "rtx 3090 ti", "rtx 3090", "rtx 3080 ti", "rtx 3080",
            "rtx 3070 ti", "rtx 3070", "rtx 3060 ti", "rtx 3060", "rtx 3050",
        ] {
            gpu_architectures.insert(model.to_string(), GpuArchitecture {
                name: model.to_string(),
                architecture: "Ampere".to_string(),
                compute_capability: (8, 6),
                min_driver_version: Some("450.00".to_string()),
            });
        }

        // RTX 20 series (Turing)
        for model in [
            "rtx 2080 ti", "rtx 2080 super", "rtx 2080",
            "rtx 2070 super", "rtx 2070", "rtx 2060 super", "rtx 2060",
            "quadro rtx 8000", "quadro rtx 6000", "quadro rtx 5000", "quadro rtx 4000",
        ] {
            gpu_architectures.insert(model.to_string(), GpuArchitecture {
                name: model.to_string(),
                architecture: "Turing".to_string(),
                compute_capability: (7, 5),
                min_driver_version: Some("410.00".to_string()),
            });
        }

        // GTX 16 series (Turing, no RT cores)
        for model in ["gtx 1660 ti", "gtx 1660 super", "gtx 1660", "gtx 1650 super", "gtx 1650", "gtx 1630"] {
            gpu_architectures.insert(model.to_string(), GpuArchitecture {
                name: model.to_string(),
                architecture: "Turing".to_string(),
                compute_capability: (7, 5),
                min_driver_version: Some("418.00".to_string()),
            });
        }

        // GTX 10 series (Pascal)
        for model in [
            "titan xp", "titan x", "gtx 1080 ti", "gtx 1080", "gtx 1070 ti", "gtx 1070",
            "gtx 1060", "gtx 1050 ti", "gtx 1050",
        ] {
            gpu_architectures.insert(model.to_string(), GpuArchitecture {
                name: model.to_string(),
                architecture: "Pascal".to_string(),
                compute_capability: (6, 1),
                min_driver_version: Some("367.00".to_string()),
            });
        }

        // GTX 9 series (Maxwell)
        for model in ["gtx 980 ti", "gtx 980", "gtx 970", "gtx 960", "gtx 950"] {
            gpu_architectures.insert(model.to_string(), GpuArchitecture {
                name: model.to_string(),
                architecture: "Maxwell".to_string(),
                compute_capability: (5, 2),
                min_driver_version: Some("340.00".to_string()),
            });
        }

        // Datacenter / Professional GPUs
        let datacenter_gpus = vec![
            // Blackwell
            ("b200", "Blackwell", (12, 0), None),
            ("b100", "Blackwell", (12, 0), None),
            // Hopper
            ("h100", "Hopper", (9, 0), None),
            ("h200", "Hopper", (9, 0), None),
            // Ada Lovelace
            ("l40s", "Ada Lovelace", (8, 9), None),
            ("l40", "Ada Lovelace", (8, 9), None),
            ("l4", "Ada Lovelace", (8, 9), None),
            ("rtx 6000 ada", "Ada Lovelace", (8, 9), None),
            // Ampere
            ("a100", "Ampere", (8, 0), None),
            ("a30", "Ampere", (8, 0), None),
            ("a10", "Ampere", (8, 6), None),
            ("a16", "Ampere", (8, 6), None),
            ("a2", "Ampere", (8, 6), None),
            ("rtx a6000", "Ampere", (8, 6), None),
            ("rtx a5000", "Ampere", (8, 6), None),
            ("rtx a4000", "Ampere", (8, 6), None),
            // Turing
            ("t4", "Turing", (7, 5), None),
            ("quadro rtx", "Turing", (7, 5), None),
            // Volta
            ("tesla v100", "Volta", (7, 0), None),
            ("titan v", "Volta", (7, 0), None),
            // Pascal
            ("tesla p100", "Pascal", (6, 0), None),
            ("tesla p40", "Pascal", (6, 1), None),
            ("tesla p4", "Pascal", (6, 1), None),
            // Kepler
            ("tesla k80", "Kepler", (3, 7), None),
            ("tesla k40", "Kepler", (3, 5), None),
        ];

        for (model, arch, cc, min_driver) in datacenter_gpus {
            gpu_architectures.insert(model.to_string(), GpuArchitecture {
                name: model.to_string(),
                architecture: arch.to_string(),
                compute_capability: cc,
                min_driver_version: min_driver.map(|v: &str| v.to_string()),
            });
        }

        // Driver Version -> Max CUDA Version mapping (FALLBACK ONLY)
        //
        // Primary source: nvidia-smi reports "CUDA Version: XX.X" directly.
        // This map is only used when nvidia-smi output cannot be parsed
        // (e.g., modinfo on Linux, Windows registry detection).
        //
        // Source: https://docs.nvidia.com/cuda/cuda-toolkit-release-notes/
        // Ordered from newest to oldest — first match wins in get_max_cuda_version()
        let driver_cuda_map = vec![
            // CUDA 13.x (driver >= 580)
            ("590.".to_string(), "13.1".to_string()),
            ("580.".to_string(), "13.0".to_string()),
            // CUDA 12.x
            ("575.".to_string(), "12.9".to_string()),
            ("570.".to_string(), "12.8".to_string()),
            ("560.".to_string(), "12.6".to_string()),
            ("555.".to_string(), "12.5".to_string()),
            ("550.".to_string(), "12.4".to_string()),
            ("545.".to_string(), "12.3".to_string()),
            ("535.".to_string(), "12.2".to_string()),
            ("530.".to_string(), "12.1".to_string()),
            ("525.".to_string(), "12.0".to_string()),
            // CUDA 11.x
            ("520.".to_string(), "11.8".to_string()),
            ("515.".to_string(), "11.7".to_string()),
            ("510.".to_string(), "11.6".to_string()),
            ("495.".to_string(), "11.5".to_string()),
            ("470.".to_string(), "11.4".to_string()),
            ("465.".to_string(), "11.3".to_string()),
            ("460.".to_string(), "11.2".to_string()),
            ("455.".to_string(), "11.1".to_string()),
            ("450.".to_string(), "11.0".to_string()),
            // CUDA 10.x
            ("440.".to_string(), "10.2".to_string()),
            ("418.".to_string(), "10.1".to_string()),
            ("410.".to_string(), "10.0".to_string()),
            // CUDA 9.x
            ("396.".to_string(), "9.2".to_string()),
            ("390.".to_string(), "9.1".to_string()),
            ("384.".to_string(), "9.0".to_string()),
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
        // Parse the driver major version for range-based matching
        let driver_major: u32 = driver_version
            .split('.')
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        // Try prefix match first (handles exact major.minor matching)
        for (prefix, cuda_ver) in &self.driver_cuda_map {
            // Extract the major from the prefix (e.g., "570." -> 570)
            let prefix_major: u32 = prefix
                .trim_end_matches('.')
                .parse()
                .unwrap_or(0);

            if driver_major >= prefix_major {
                return Some(cuda_ver.clone());
            }
        }

        // Fallback for very new drivers (assume latest known)
        if driver_major > 590 {
            return Some("13.1".to_string());
        }

        None
    }
}

// Global instance nicely wrapped
pub static REGISTRY: Lazy<CompatibilityRegistry> = Lazy::new(CompatibilityRegistry::new);

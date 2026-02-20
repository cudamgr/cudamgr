use crate::error::{CudaMgrResult, SystemError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Information about detected CUDA installations on the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CudaDetectionResult {
    pub installations: Vec<CudaInstallation>,
    pub conflicts: Vec<CudaConflict>,
    pub system_cuda: Option<SystemCudaInfo>,
}

/// Information about system-wide CUDA installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCudaInfo {
    pub nvcc_version: Option<String>,
    pub nvcc_path: Option<PathBuf>,
    pub runtime_version: Option<String>,
    pub driver_version: Option<String>,
}

/// Represents a conflict between CUDA installations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CudaConflict {
    pub conflict_type: ConflictType,
    pub description: String,
    pub affected_installations: Vec<String>,
    pub resolution_suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    MultipleVersionsInPath,
    EnvironmentVariableMismatch,
    SystemPackageConflict,
    SymlinkConflict,
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

    /// Detect CUDA installation from a given path
    pub fn detect_from_path(path: &Path) -> CudaMgrResult<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let bin_path = path.join("bin");
        let nvcc_path = bin_path.join(if cfg!(windows) { "nvcc.exe" } else { "nvcc" });

        if !nvcc_path.exists() {
            return Ok(None);
        }

        // Get version from nvcc
        let version = Self::get_nvcc_version(&nvcc_path)?;
        let mut installation = Self::new(version, path.to_path_buf());

        // Detect components
        installation.components = Self::detect_components(path)?;

        // Calculate size
        installation.size_bytes = Self::calculate_directory_size(path)?;

        // Try to get install date from directory metadata
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(created) = metadata.created() {
                installation.install_date = DateTime::from(created);
            }
        }

        Ok(Some(installation))
    }

    /// Get NVCC version from the executable
    fn get_nvcc_version(nvcc_path: &Path) -> CudaMgrResult<String> {
        let output = Command::new(nvcc_path)
            .arg("--version")
            .output()
            .map_err(|e| SystemError::CommandExecution(format!("Failed to run nvcc: {}", e)))?;

        if !output.status.success() {
            return Err(SystemError::CommandExecution("nvcc --version failed".to_string()).into());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse version from output like "Cuda compilation tools, release 11.8, V11.8.89"
        for line in output_str.lines() {
            if line.contains("release") {
                if let Some(version_part) = line.split("release ").nth(1) {
                    if let Some(version) = version_part.split(',').next() {
                        return Ok(version.trim().to_string());
                    }
                }
            }
        }

        Err(SystemError::ParseError("Could not parse NVCC version".to_string()).into())
    }

    /// Detect CUDA components in the installation
    fn detect_components(install_path: &Path) -> CudaMgrResult<Vec<CudaComponent>> {
        let mut components = Vec::new();

        // Platform-aware component paths
        // On Windows: binaries in bin/, libraries in lib/x64/
        // On Linux: binaries in bin/, libraries in lib64/
        let essential_components: Vec<(&str, String, bool)> = if cfg!(windows) {
            vec![
                ("NVCC Compiler", "bin/nvcc.exe".to_string(), true),
                ("CUDA Runtime", "bin/cudart64_12.dll".to_string(), true),
                ("CUDA Driver API", "lib/x64/cuda.lib".to_string(), false),
                ("cuBLAS", "lib/x64/cublas.lib".to_string(), false),
                ("cuFFT", "lib/x64/cufft.lib".to_string(), false),
                ("cuRAND", "lib/x64/curand.lib".to_string(), false),
                ("cuSPARSE", "lib/x64/cusparse.lib".to_string(), false),
                ("NPP", "lib/x64/nppial.lib".to_string(), false),
            ]
        } else {
            vec![
                ("NVCC Compiler", "bin/nvcc".to_string(), true),
                ("CUDA Runtime", "lib64/libcudart.so".to_string(), true),
                ("CUDA Driver API", "lib64/libcuda.so".to_string(), false),
                ("cuBLAS", "lib64/libcublas.so".to_string(), false),
                ("cuFFT", "lib64/libcufft.so".to_string(), false),
                ("cuRAND", "lib64/libcurand.so".to_string(), false),
                ("cuSPARSE", "lib64/libcusparse.so".to_string(), false),
                ("NPP", "lib64/libnpp.so".to_string(), false),
            ]
        };

        for (name, relative_path, required) in &essential_components {
            let component_path = install_path.join(relative_path);

            // For runtime DLLs on Windows, also check for versioned variants
            // e.g., cudart64_12.dll might be cudart64_11.dll for older versions
            let final_path = if !component_path.exists() && cfg!(windows) && relative_path.contains("cudart64_12") {
                // Try common version suffixes
                let parent = component_path.parent().unwrap_or(install_path);
                let alt_names = ["cudart64_11.dll", "cudart64.dll", "cudart.dll"];
                alt_names
                    .iter()
                    .map(|n| parent.join(n))
                    .find(|p| p.exists())
                    .unwrap_or(component_path)
            } else {
                component_path
            };

            components.push(CudaComponent {
                name: name.to_string(),
                version: "unknown".to_string(),
                path: final_path,
                required: *required,
            });
        }

        Ok(components)
    }

    /// Calculate total size of installation directory
    fn calculate_directory_size(path: &Path) -> CudaMgrResult<u64> {
        fn dir_size(path: &Path) -> std::io::Result<u64> {
            let mut size = 0;
            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        size += dir_size(&path)?;
                    } else {
                        size += entry.metadata()?.len();
                    }
                }
            }
            Ok(size)
        }

        dir_size(path).map_err(|e| SystemError::Io(e).into())
    }

    /// Detect all CUDA installations on the system
    pub fn detect_all_installations() -> CudaMgrResult<CudaDetectionResult> {
        let mut installations = Vec::new();
        let mut conflicts = Vec::new();

        // Common CUDA installation paths
        let common_paths = Self::get_common_cuda_paths();

        for path in common_paths {
            if let Some(installation) = Self::detect_from_path(&path)? {
                installations.push(installation);
            }
        }

        // Detect system CUDA (from PATH)
        let system_cuda = Self::detect_system_cuda()?;

        // Detect conflicts
        conflicts.extend(Self::detect_conflicts(&installations, &system_cuda)?);

        Ok(CudaDetectionResult {
            installations,
            conflicts,
            system_cuda,
        })
    }

    /// Get common CUDA installation paths for the current platform
    fn get_common_cuda_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "linux")]
        {
            // Standard Linux paths
            paths.extend([
                PathBuf::from("/usr/local/cuda"),
                PathBuf::from("/opt/cuda"),
                PathBuf::from("/usr/cuda"),
            ]);

            // Version-specific paths
            for version in [
                "12.3", "12.2", "12.1", "12.0", "11.8", "11.7", "11.6", "11.5", "11.4", "11.3",
                "11.2", "11.1", "11.0",
            ] {
                paths.push(PathBuf::from(format!("/usr/local/cuda-{}", version)));
                paths.push(PathBuf::from(format!("/opt/cuda-{}", version)));
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Standard Windows paths
            if let Ok(program_files) = std::env::var("ProgramFiles") {
                paths.push(
                    PathBuf::from(program_files)
                        .join("NVIDIA GPU Computing Toolkit")
                        .join("CUDA"),
                );
            }

            // Version-specific paths
            for version in [
                "v12.3", "v12.2", "v12.1", "v12.0", "v11.8", "v11.7", "v11.6", "v11.5", "v11.4",
                "v11.3", "v11.2", "v11.1", "v11.0",
            ] {
                if let Ok(program_files) = std::env::var("ProgramFiles") {
                    paths.push(
                        PathBuf::from(program_files)
                            .join("NVIDIA GPU Computing Toolkit")
                            .join("CUDA")
                            .join(version),
                    );
                }
            }
        }

        // Check environment variables
        if let Ok(cuda_home) = std::env::var("CUDA_HOME") {
            paths.push(PathBuf::from(cuda_home));
        }
        if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
            paths.push(PathBuf::from(cuda_path));
        }

        paths
    }

    /// Detect system-wide CUDA installation (from PATH)
    fn detect_system_cuda() -> CudaMgrResult<Option<SystemCudaInfo>> {
        // Try to find nvcc in PATH
        let nvcc_output = Command::new("nvcc").arg("--version").output();

        let (nvcc_version, nvcc_path) = if let Ok(output) = nvcc_output {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = Self::parse_nvcc_version_output(&version_str)?;

                // Try to find nvcc path
                let which_output = Command::new(if cfg!(windows) { "where" } else { "which" })
                    .arg("nvcc")
                    .output();

                let path = if let Ok(which_out) = which_output {
                    if which_out.status.success() {
                        let path_str = String::from_utf8_lossy(&which_out.stdout);
                        Some(PathBuf::from(path_str.trim()))
                    } else {
                        None
                    }
                } else {
                    None
                };

                (Some(version), path)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        if nvcc_version.is_none() && nvcc_path.is_none() {
            return Ok(None);
        }

        Ok(Some(SystemCudaInfo {
            nvcc_version,
            nvcc_path,
            runtime_version: None, // Could be enhanced to detect runtime version
            driver_version: None,  // Could be enhanced to detect driver version
        }))
    }

    /// Parse NVCC version from command output
    fn parse_nvcc_version_output(output: &str) -> CudaMgrResult<String> {
        for line in output.lines() {
            if line.contains("release") {
                if let Some(version_part) = line.split("release ").nth(1) {
                    if let Some(version) = version_part.split(',').next() {
                        return Ok(version.trim().to_string());
                    }
                }
            }
        }
        Err(SystemError::ParseError("Could not parse NVCC version from output".to_string()).into())
    }

    /// Detect conflicts between CUDA installations
    fn detect_conflicts(
        installations: &[CudaInstallation],
        system_cuda: &Option<SystemCudaInfo>,
    ) -> CudaMgrResult<Vec<CudaConflict>> {
        let mut conflicts = Vec::new();

        // Check for multiple versions in PATH
        if installations.len() > 1 && system_cuda.is_some() {
            let versions: Vec<String> = installations.iter().map(|i| i.version.clone()).collect();
            let paths: Vec<String> = installations
                .iter()
                .map(|i| i.install_path.display().to_string())
                .collect();

            let description = format!(
                "Multiple CUDA versions detected on system. Found at:\n    - {}",
                paths.join("\n    - ")
            );

            conflicts.push(CudaConflict {
                conflict_type: ConflictType::MultipleVersionsInPath,
                description,
                affected_installations: versions,
                resolution_suggestion:
                    "Use cudamgr to manage CUDA versions and ensure only one is active".to_string(),
            });
        }

        // Check for environment variable mismatches
        if let Ok(cuda_home) = std::env::var("CUDA_HOME") {
            let cuda_home_path = PathBuf::from(cuda_home);
            let matching_installation = installations
                .iter()
                .find(|inst| inst.install_path == cuda_home_path);

            if matching_installation.is_none() && !installations.is_empty() {
                conflicts.push(CudaConflict {
                    conflict_type: ConflictType::EnvironmentVariableMismatch,
                    description:
                        "CUDA_HOME points to different installation than detected versions"
                            .to_string(),
                    affected_installations: vec!["CUDA_HOME".to_string()],
                    resolution_suggestion: "Update CUDA_HOME to point to desired CUDA installation"
                        .to_string(),
                });
            }
        }

        Ok(conflicts)
    }

    pub fn is_valid(&self) -> bool {
        self.install_path.exists()
            && self.toolkit_path.exists()
            && !self
                .components
                .iter()
                .filter(|c| c.required)
                .any(|c| !c.path.exists())
    }

    pub fn get_nvcc_path(&self) -> PathBuf {
        self.toolkit_path
            .join(if cfg!(windows) { "nvcc.exe" } else { "nvcc" })
    }

    pub fn get_lib_path(&self) -> PathBuf {
        if cfg!(windows) {
            self.install_path.join("lib").join("x64")
        } else {
            self.install_path.join("lib64")
        }
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

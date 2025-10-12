use std::fmt;
use serde::{Deserialize, Serialize};
use crate::error::CudaMgrResult;
use super::SystemInfo;
use super::cuda::{CudaDetectionResult, CudaInstallation};

/// Comprehensive system report for CUDA compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReport {
    pub system_info: SystemInfo,
    pub cuda_detection: CudaDetectionResult,
    pub compatibility_status: CompatibilityStatus,
    pub recommendations: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Overall compatibility status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityStatus {
    Compatible,
    CompatibleWithWarnings,
    Incompatible,
    Unknown,
}

/// System report generator
pub struct SystemReportGenerator;

impl SystemReportGenerator {
    /// Detect system information synchronously
    fn detect_system_info_sync() -> CudaMgrResult<SystemInfo> {
        // Detect GPU information
        let gpu = super::gpu::GpuInfo::detect().unwrap_or(None);
        
        // Detect driver information
        let driver = super::driver::DriverInfo::detect().ok().flatten();
        
        // Detect compiler information
        let compiler = super::compiler::CompilerInfo::detect().ok()
            .and_then(|compilers| compilers.into_iter().find(|c| c.is_compatible));
        
        // Detect distribution information
        let distro = super::distro::DistroInfo::detect()?;
        
        // Detect storage information
        let storage_path = super::storage::StorageInfo::get_default_cuda_path();
        let storage = super::storage::StorageInfo::detect(&storage_path)?;
        
        // Detect security information
        let security = super::security::SecurityInfo::detect()?;
        
        Ok(SystemInfo {
            gpu,
            driver,
            compiler,
            distro,
            storage,
            security,
        })
    }

    /// Generate a comprehensive system report
    pub async fn generate_report() -> CudaMgrResult<SystemReport> {
        // Detect system information synchronously to avoid runtime conflicts
        let system_info = Self::detect_system_info_sync()?;
        let cuda_detection = CudaInstallation::detect_all_installations()?;
        
        let mut recommendations = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Analyze system compatibility
        let compatibility_status = Self::analyze_compatibility(
            &system_info,
            &cuda_detection,
            &mut recommendations,
            &mut warnings,
            &mut errors,
        );

        Ok(SystemReport {
            system_info,
            cuda_detection,
            compatibility_status,
            recommendations,
            warnings,
            errors,
        })
    }

    /// Analyze system compatibility and generate recommendations
    fn analyze_compatibility(
        system_info: &SystemInfo,
        cuda_detection: &CudaDetectionResult,
        recommendations: &mut Vec<String>,
        warnings: &mut Vec<String>,
        errors: &mut Vec<String>,
    ) -> CompatibilityStatus {
        let mut has_errors = false;
        let mut has_warnings = false;

        // Check GPU compatibility
        match &system_info.gpu {
            Some(gpu) => {
                if !gpu.supports_cuda() {
                    errors.push("No CUDA-compatible GPU detected".to_string());
                    has_errors = true;
                } else {
                    if let Some((major, minor)) = gpu.compute_capability {
                        recommendations.push(format!(
                            "GPU {} detected with compute capability {}.{}",
                            gpu.name, major, minor
                        ));
                    } else {
                        recommendations.push(format!("GPU {} detected", gpu.name));
                    }
                }
            }
            None => {
                errors.push("No GPU detected".to_string());
                has_errors = true;
            }
        }

        // Check driver compatibility
        match &system_info.driver {
            Some(driver) => {
                if driver.version.is_empty() {
                    warnings.push("NVIDIA driver version could not be determined".to_string());
                    has_warnings = true;
                } else {
                    recommendations.push(format!("NVIDIA driver {} detected", driver.version));
                }
            }
            None => {
                errors.push("No NVIDIA driver detected".to_string());
                recommendations.push("Install NVIDIA drivers before installing CUDA".to_string());
                has_errors = true;
            }
        }

        // Check compiler availability
        match &system_info.compiler {
            Some(compiler) => {
                if compiler.is_compatible {
                    recommendations.push(format!(
                        "Compatible compiler {} {} detected",
                        compiler.name, compiler.version
                    ));
                } else {
                    warnings.push(format!(
                        "Compiler {} {} may not be compatible with CUDA",
                        compiler.name, compiler.version
                    ));
                    has_warnings = true;
                }
            }
            None => {
                errors.push("No compatible compiler detected".to_string());
                recommendations.push("Install a compatible compiler (GCC on Linux, MSVC on Windows)".to_string());
                has_errors = true;
            }
        }

        // Check storage space
        if !system_info.storage.has_sufficient_space {
            errors.push(format!(
                "Insufficient disk space. Available: {} GB",
                system_info.storage.available_space_gb
            ));
            has_errors = true;
        } else {
            recommendations.push(format!(
                "Sufficient disk space available: {} GB",
                system_info.storage.available_space_gb
            ));
        }

        // Check security settings
        let security_issues = system_info.security.get_security_issues();
        for issue in security_issues {
            if issue.contains("required") || issue.contains("Cannot") {
                errors.push(issue);
                has_errors = true;
            } else {
                warnings.push(issue);
                has_warnings = true;
            }
        }

        // Add security recommendations
        if !system_info.security.has_admin_privileges {
            recommendations.push("Run as administrator/root for CUDA installation".to_string());
        }

        if system_info.security.secure_boot_enabled {
            recommendations.push("Consider disabling Secure Boot if driver installation fails".to_string());
        }

        // Check PATH configuration
        let path_recommendations = system_info.security.path_configuration.get_recommendations();
        recommendations.extend(path_recommendations);

        if system_info.security.has_path_conflicts() {
            warnings.push("Conflicting CUDA paths detected in PATH environment variable".to_string());
            has_warnings = true;
        }

        // Check existing CUDA installations
        if !cuda_detection.installations.is_empty() {
            recommendations.push(format!(
                "{} existing CUDA installation(s) detected",
                cuda_detection.installations.len()
            ));

            for installation in &cuda_detection.installations {
                if !installation.is_valid() {
                    warnings.push(format!(
                        "CUDA {} installation at {} appears to be incomplete",
                        installation.version,
                        installation.install_path.display()
                    ));
                    has_warnings = true;
                }
            }
        }

        // Check for conflicts
        if !cuda_detection.conflicts.is_empty() {
            for conflict in &cuda_detection.conflicts {
                warnings.push(format!("Conflict detected: {}", conflict.description));
                recommendations.push(conflict.resolution_suggestion.clone());
                has_warnings = true;
            }
        }

        // Determine overall compatibility status
        if has_errors {
            CompatibilityStatus::Incompatible
        } else if has_warnings {
            CompatibilityStatus::CompatibleWithWarnings
        } else {
            CompatibilityStatus::Compatible
        }
    }
}

impl fmt::Display for SystemReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== CUDA System Compatibility Report ===")?;
        writeln!(f)?;

        // Overall status
        writeln!(f, "Overall Status: {}", self.compatibility_status)?;
        writeln!(f)?;

        // System Information
        writeln!(f, "=== System Information ===")?;
        writeln!(f, "OS: {} {}", self.system_info.distro.name, self.system_info.distro.version)?;
        
        if let Some(gpu) = &self.system_info.gpu {
            let memory_str = gpu.memory_mb.map(|m| format!("{} MB", m)).unwrap_or_else(|| "Unknown".to_string());
            let compute_str = gpu.compute_capability
                .map(|(major, minor)| format!("Compute {}.{}", major, minor))
                .unwrap_or_else(|| "Unknown".to_string());
            writeln!(f, "GPU: {} ({}, {})", gpu.name, memory_str, compute_str)?;
        } else {
            writeln!(f, "GPU: Not detected")?;
        }

        if let Some(driver) = &self.system_info.driver {
            let cuda_version = driver.max_cuda_version.as_deref().unwrap_or("Unknown");
            writeln!(f, "Driver: NVIDIA {} (Max CUDA {})", driver.version, cuda_version)?;
        } else {
            writeln!(f, "Driver: Not detected")?;
        }

        if let Some(compiler) = &self.system_info.compiler {
            writeln!(f, "Compiler: {} {} (Compatible: {})", 
                compiler.name, compiler.version, compiler.is_compatible)?;
        } else {
            writeln!(f, "Compiler: Not detected")?;
        }

        writeln!(f, "Storage: {} GB available", self.system_info.storage.available_space_gb)?;
        writeln!(f, "Admin Privileges: {}", self.system_info.security.has_admin_privileges)?;
        writeln!(f, "Secure Boot: {}", 
            if self.system_info.security.secure_boot_enabled { "Enabled" } else { "Disabled" })?;
        writeln!(f)?;

        // CUDA Installations
        if !self.cuda_detection.installations.is_empty() {
            writeln!(f, "=== Existing CUDA Installations ===")?;
            for installation in &self.cuda_detection.installations {
                writeln!(f, "  {} at {} ({} GB)", 
                    installation.version,
                    installation.install_path.display(),
                    installation.size_bytes / (1024 * 1024 * 1024))?;
            }
            writeln!(f)?;
        }

        // System CUDA
        if let Some(system_cuda) = &self.cuda_detection.system_cuda {
            writeln!(f, "=== System CUDA (in PATH) ===")?;
            if let Some(version) = &system_cuda.nvcc_version {
                writeln!(f, "  NVCC Version: {}", version)?;
            }
            if let Some(path) = &system_cuda.nvcc_path {
                writeln!(f, "  NVCC Path: {}", path.display())?;
            }
            writeln!(f)?;
        }

        // Conflicts
        if !self.cuda_detection.conflicts.is_empty() {
            writeln!(f, "=== Conflicts Detected ===")?;
            for conflict in &self.cuda_detection.conflicts {
                writeln!(f, "  {}: {}", conflict.conflict_type, conflict.description)?;
                writeln!(f, "    Resolution: {}", conflict.resolution_suggestion)?;
            }
            writeln!(f)?;
        }

        // Errors
        if !self.errors.is_empty() {
            writeln!(f, "=== Errors ===")?;
            for error in &self.errors {
                writeln!(f, "  ❌ {}", error)?;
            }
            writeln!(f)?;
        }

        // Warnings
        if !self.warnings.is_empty() {
            writeln!(f, "=== Warnings ===")?;
            for warning in &self.warnings {
                writeln!(f, "  ⚠️  {}", warning)?;
            }
            writeln!(f)?;
        }

        // Recommendations
        if !self.recommendations.is_empty() {
            writeln!(f, "=== Recommendations ===")?;
            for recommendation in &self.recommendations {
                writeln!(f, "  💡 {}", recommendation)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for CompatibilityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatibilityStatus::Compatible => write!(f, "✅ Compatible"),
            CompatibilityStatus::CompatibleWithWarnings => write!(f, "⚠️  Compatible (with warnings)"),
            CompatibilityStatus::Incompatible => write!(f, "❌ Incompatible"),
            CompatibilityStatus::Unknown => write!(f, "❓ Unknown"),
        }
    }
}

impl fmt::Display for super::cuda::ConflictType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            super::cuda::ConflictType::MultipleVersionsInPath => write!(f, "Multiple Versions in PATH"),
            super::cuda::ConflictType::EnvironmentVariableMismatch => write!(f, "Environment Variable Mismatch"),
            super::cuda::ConflictType::SystemPackageConflict => write!(f, "System Package Conflict"),
            super::cuda::ConflictType::SymlinkConflict => write!(f, "Symlink Conflict"),
        }
    }
}
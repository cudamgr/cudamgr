use crate::cli::output::OutputFormatter;
use crate::error::{CudaMgrError, CudaMgrResult};
use async_trait::async_trait;
use clap::Subcommand;

use crate::install::downloader::PackageDownloader;
use crate::install::redist;
use crate::system::compatibility::{CompatibilityRegistry, REGISTRY};
use crate::system::cuda::CudaInstallation;

#[derive(Subcommand)]
pub enum Command {
    /// Check system compatibility for CUDA installation
    Doctor(DoctorArgs),
    /// Install a specific CUDA version
    Install(InstallArgs),
    /// Switch to a specific CUDA version
    Use(UseArgs),
    /// List installed and available CUDA versions
    List(ListArgs),
    /// Download CUDA toolkit redistributables (one or more versions) in one go
    Download(DownloadArgs),
    /// Uninstall a CUDA version
    Uninstall(UninstallArgs),
    /// View logs and debugging information
    Logs(LogsArgs),
}

#[derive(clap::Args)]
pub struct DoctorArgs {
    /// Show detailed system information
    #[arg(short, long)]
    pub verbose: bool,

    /// Update the compatibility registry from remote before checking
    #[arg(long)]
    pub update_registry: bool,
}

impl DoctorArgs {
    pub fn validate(&self) -> CudaMgrResult<()> {
        // Doctor command has no validation requirements
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct InstallArgs {
    /// CUDA version to install
    pub version: String,
    /// Force installation even if version exists
    #[arg(short, long)]
    pub force: bool,
    /// Skip driver installation
    #[arg(long)]
    pub skip_driver: bool,
}

impl InstallArgs {
    pub fn validate(&self) -> CudaMgrResult<()> {
        if self.version.is_empty() {
            return Err(CudaMgrError::Cli("Version cannot be empty".to_string()));
        }

        // Validate version format (X.Y or X.Y.Z)
        let parts: Vec<&str> = self.version.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(CudaMgrError::Cli(format!(
                "Invalid version format '{}'. Expected format: X.Y or X.Y.Z (e.g., 12.0 or 12.0.1)",
                self.version
            )));
        }
        for part in &parts {
            if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
                return Err(CudaMgrError::Cli(format!(
                    "Invalid version format '{}'. Each segment must be a number",
                    self.version
                )));
            }
        }

        Ok(())
    }
}

#[derive(clap::Args)]
pub struct UseArgs {
    /// CUDA version to switch to
    pub version: String,
    /// Install version if not present
    #[arg(short, long)]
    pub install: bool,
}

impl UseArgs {
    pub fn validate(&self) -> CudaMgrResult<()> {
        if self.version.is_empty() {
            return Err(CudaMgrError::Cli("Version cannot be empty".to_string()));
        }

        // Basic version format validation
        if !self.version.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err(CudaMgrError::Cli(
                "Invalid version format. Use format like '11.8' or '12.0'".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Show available versions for download
    #[arg(short, long)]
    pub available: bool,
    /// Show detailed information
    #[arg(short, long)]
    pub verbose: bool,
}

impl ListArgs {
    pub fn validate(&self) -> CudaMgrResult<()> {
        // List command has no validation requirements
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct DownloadArgs {
    /// CUDA version(s) to download (e.g. 11.8 12.0 12.6)
    #[arg(value_name = "VERSION")]
    pub versions: Vec<String>,

    /// Download all versions from the compatibility registry
    #[arg(long)]
    pub all: bool,

    /// Directory to save downloads (default: cache under cudamgr data dir)
    #[arg(short, long, value_name = "DIR")]
    pub output_dir: Option<std::path::PathBuf>,
}

impl DownloadArgs {
    pub fn validate(&self) -> CudaMgrResult<()> {
        if !self.all && self.versions.is_empty() {
            return Err(CudaMgrError::Cli(
                "Specify at least one VERSION or use --all".to_string(),
            ));
        }
        for v in &self.versions {
            if v.is_empty() || !v.chars().all(|c| c.is_ascii_digit() || c == '.') {
                return Err(CudaMgrError::Cli(format!(
                    "Invalid version '{}'. Use format like 11.8 or 12.0",
                    v
                )));
            }
        }
        Ok(())
    }
}

#[derive(clap::Args)]
pub struct UninstallArgs {
    /// CUDA version to uninstall
    pub version: String,
    /// Skip confirmation prompts
    #[arg(short, long)]
    pub yes: bool,
}

impl UninstallArgs {
    pub fn validate(&self) -> CudaMgrResult<()> {
        if self.version.is_empty() {
            return Err(CudaMgrError::Cli("Version cannot be empty".to_string()));
        }

        // Basic version format validation
        if !self.version.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err(CudaMgrError::Cli(
                "Invalid version format. Use format like '11.8' or '12.0'".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(clap::Args)]
pub struct LogsArgs {
    /// Number of log lines to show
    #[arg(short, long, default_value = "50")]
    pub lines: usize,
    /// Follow log output
    #[arg(short, long)]
    pub follow: bool,
}

impl LogsArgs {
    pub fn validate(&self) -> CudaMgrResult<()> {
        if self.lines == 0 {
            return Err(CudaMgrError::Cli(
                "Number of lines must be greater than 0".to_string(),
            ));
        }

        if self.lines > 10000 {
            return Err(CudaMgrError::Cli(
                "Number of lines cannot exceed 10000".to_string(),
            ));
        }

        Ok(())
    }
}

/// Trait for command handlers that can be executed
#[async_trait]
pub trait CommandHandler {
    async fn execute(&self) -> CudaMgrResult<()>;
    fn validate(&self) -> CudaMgrResult<()> {
        Ok(())
    }
}

/// Router for dispatching commands to their handlers
pub struct CommandRouter;

impl CommandRouter {
    pub async fn route(command: Command) -> CudaMgrResult<()> {
        // Validate command before execution
        command.validate()?;

        // Route to appropriate handler
        match command {
            Command::Doctor(args) => DoctorHandler::new(args).execute().await,
            Command::Install(args) => InstallHandler::new(args).execute().await,
            Command::Use(args) => UseHandler::new(args).execute().await,
            Command::List(args) => ListHandler::new(args).execute().await,
            Command::Download(args) => DownloadHandler::new(args).execute().await,
            Command::Uninstall(args) => UninstallHandler::new(args).execute().await,
            Command::Logs(args) => LogsHandler::new(args).execute().await,
        }
    }
}

impl Command {
    pub async fn execute(self) -> CudaMgrResult<()> {
        CommandRouter::route(self).await
    }

    pub fn validate(&self) -> CudaMgrResult<()> {
        match self {
            Command::Doctor(args) => args.validate(),
            Command::Install(args) => args.validate(),
            Command::Use(args) => args.validate(),
            Command::List(args) => args.validate(),
            Command::Download(args) => args.validate(),
            Command::Uninstall(args) => args.validate(),
            Command::Logs(args) => args.validate(),
        }
    }
}

// Command Handlers
pub struct DoctorHandler {
    args: DoctorArgs,
}

impl DoctorHandler {
    pub fn new(args: DoctorArgs) -> Self {
        Self { args }
    }
}

#[async_trait]
impl CommandHandler for DoctorHandler {
    async fn execute(&self) -> CudaMgrResult<()> {
        use crate::system::compatibility::CompatibilityRegistry;
        use crate::system::SystemReportGenerator;

        tracing::info!("Running system doctor with verbose: {}", self.args.verbose);

        // Update compatibility registry if requested
        if self.args.update_registry {
            OutputFormatter::info("Updating compatibility registry from remote...");
            let mut registry = CompatibilityRegistry::new();
            match registry.update_from_remote().await {
                Ok(true) => {
                    OutputFormatter::success("Registry updated successfully");
                }
                Ok(false) => {
                    OutputFormatter::info("Registry is already up to date");
                }
                Err(e) => {
                    OutputFormatter::warning(&format!(
                        "Failed to update registry (using cached/built-in): {}",
                        e
                    ));
                }
            }
        }

        OutputFormatter::info("Running comprehensive system compatibility check...");

        // Generate comprehensive system report
        let report = SystemReportGenerator::generate_report().await?;

        // Display the report
        if self.args.verbose {
            // In verbose mode, show the full report
            println!("{}", report);
        } else {
            // In normal mode, show a summary
            Self::display_summary(&report);
        }

        // Return success/failure based on compatibility status
        match report.compatibility_status {
            crate::system::CompatibilityStatus::Compatible => {
                OutputFormatter::success("System is compatible with CUDA installation");
                Ok(())
            }
            crate::system::CompatibilityStatus::CompatibleWithWarnings => {
                OutputFormatter::warning("System is compatible but has warnings");
                Ok(())
            }
            crate::system::CompatibilityStatus::Incompatible => {
                OutputFormatter::error("System is not compatible with CUDA installation");
                Err(CudaMgrError::System(
                    crate::error::SystemError::Incompatible(
                        "System compatibility check failed".to_string(),
                    ),
                ))
            }
            crate::system::CompatibilityStatus::PrerequisitesMissing => {
                OutputFormatter::warning(
                    "System is compatible but requires setup (Admin/Compiler)",
                );
                Ok(())
            }
            crate::system::CompatibilityStatus::Unknown => {
                OutputFormatter::warning("System compatibility could not be determined");
                Ok(())
            }
        }
    }
}

impl DoctorHandler {
    /// Display a summary of the system report
    fn display_summary(report: &crate::system::SystemReport) {
        println!("=== CUDA System Compatibility Summary ===\n");

        // Overall status
        println!("Status: {}\n", report.compatibility_status);

        // Key system info
        println!("System Information:");
        println!(
            "  OS: {} {}",
            report.system_info.distro.name, report.system_info.distro.version
        );

        if let Some(gpu) = &report.system_info.gpu {
            if let Some((major, minor)) = gpu.compute_capability {
                println!("  GPU: {} (Compute {}.{})", gpu.name, major, minor);
            } else {
                println!("  GPU: {}", gpu.name);
            }
        } else {
            println!("  GPU: ❌ Not detected");
        }

        if let Some(driver) = &report.system_info.driver {
            println!("  Driver: ✅ NVIDIA {}", driver.version);
        } else {
            println!("  Driver: ❌ Not detected");
        }

        if let Some(compiler) = &report.system_info.compiler {
            println!(
                "  Compiler: {} {} {}",
                if compiler.is_compatible {
                    "✅"
                } else {
                    "⚠️"
                },
                compiler.name,
                compiler.version
            );
        } else {
            println!("  Compiler: ❌ Not detected");
        }

        println!(
            "  Storage: {} GB available",
            report.system_info.storage.available_space_gb
        );
        println!(
            "  Admin: {}",
            if report.system_info.security.has_admin_privileges {
                "✅"
            } else {
                "❌"
            }
        );

        // CUDA installations
        if !report.cuda_detection.installations.is_empty() {
            println!("\nExisting CUDA Installations:");
            for installation in &report.cuda_detection.installations {
                println!(
                    "  {} at {}",
                    installation.version,
                    installation.install_path.display()
                );
            }
        }

        // Show critical errors
        if !report.errors.is_empty() {
            println!("\nCritical Issues:");
            for error in &report.errors {
                println!("  ❌ {}", error);
            }
        }

        // Show warnings
        if !report.warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &report.warnings {
                println!("  ⚠️  {}", warning);
            }
        }

        // Show key recommendations
        if !report.recommendations.is_empty() {
            println!("\nNext Steps:");
            for (i, recommendation) in report.recommendations.iter().take(3).enumerate() {
                println!("  {}. {}", i + 1, recommendation);
            }
            if report.recommendations.len() > 3 {
                println!(
                    "  ... and {} more (use --verbose for full details)",
                    report.recommendations.len() - 3
                );
            }
        }

        println!("\nRun 'cudamgr doctor --verbose' for detailed information.");
    }
}

pub struct InstallHandler {
    args: InstallArgs,
}

impl InstallHandler {
    pub fn new(args: InstallArgs) -> Self {
        Self { args }
    }
}

#[async_trait]
impl CommandHandler for InstallHandler {
    async fn execute(&self) -> CudaMgrResult<()> {
        tracing::info!("Installing CUDA version: {}", self.args.version);
        OutputFormatter::info(&format!("Installing CUDA version {}", self.args.version));

        // TODO: Implement actual install functionality
        OutputFormatter::warning("Install command implementation pending");
        Err(CudaMgrError::Cli(
            "Install command not yet implemented".to_string(),
        ))
    }
}

pub struct UseHandler {
    args: UseArgs,
}

impl UseHandler {
    pub fn new(args: UseArgs) -> Self {
        Self { args }
    }
}

#[async_trait]
impl CommandHandler for UseHandler {
    async fn execute(&self) -> CudaMgrResult<()> {
        tracing::info!("Switching to CUDA version: {}", self.args.version);
        OutputFormatter::info(&format!("Switching to CUDA version {}", self.args.version));

        // TODO: Implement actual use functionality
        OutputFormatter::warning("Use command implementation pending");
        Err(CudaMgrError::Cli(
            "Use command not yet implemented".to_string(),
        ))
    }
}

pub struct ListHandler {
    args: ListArgs,
}

impl ListHandler {
    pub fn new(args: ListArgs) -> Self {
        Self { args }
    }
}

#[async_trait]
impl CommandHandler for ListHandler {
    async fn execute(&self) -> CudaMgrResult<()> {
        tracing::info!("Listing CUDA versions, available: {}", self.args.available);

        if self.args.available {
            Self::list_available(&self.args.verbose);
        } else {
            Self::list_installed(&self.args.verbose)?;
        }

        Ok(())
    }
}

impl ListHandler {
    /// List CUDA versions that can be installed (from compatibility registry).
    fn list_available(verbose: &bool) {
        OutputFormatter::section("Available CUDA versions (installable)");

        if *verbose {
            let with_driver = REGISTRY.available_cuda_versions_with_min_driver();
            if with_driver.is_empty() {
                println!("  No version data in registry.");
                return;
            }
            println!("  {:<12} Min. driver", "Version");
            println!("  {}", "─".repeat(24));
            for (cuda, min_driver) in with_driver {
                println!("  {:<12} {}", cuda, min_driver);
            }
        } else {
            let versions = REGISTRY.available_cuda_versions();
            if versions.is_empty() {
                println!("  No version data in registry.");
                return;
            }
            for v in versions {
                println!("  {}", v);
            }
        }

        println!("\n  Run 'cudamgr install <version>' to install (when implemented).");
    }

    /// List CUDA versions currently installed on the system.
    fn list_installed(verbose: &bool) -> CudaMgrResult<()> {
        let result = CudaInstallation::detect_all_installations()?;

        OutputFormatter::section("Installed CUDA versions");

        if result.installations.is_empty() {
            println!("  No CUDA installations found.");
            if let Some(system) = &result.system_cuda {
                if let Some(v) = &system.nvcc_version {
                    println!("  nvcc in PATH: {} (source not under cudamgr)", v);
                }
            }
            println!("\n  Run 'cudamgr list --available' to see installable versions.");
            return Ok(());
        }

        if *verbose {
            println!("  {:<10} {:<12} Path", "Version", "Size");
            println!("  {}", "─".repeat(50));
            for inst in &result.installations {
                let size_gb = inst.size_bytes / (1024 * 1024 * 1024);
                let size_str = format!("{} GB", size_gb);
                let valid = if inst.is_valid() { "✓" } else { "incomplete" };
                println!(
                    "  {:<10} {:<12} {} {}",
                    inst.version,
                    size_str,
                    inst.install_path.display(),
                    valid
                );
            }
        } else {
            for inst in &result.installations {
                let active = if result
                    .system_cuda
                    .as_ref()
                    .is_some_and(|s| s.nvcc_version.as_deref() == Some(inst.version.as_str()))
                {
                    " (active in PATH)"
                } else {
                    ""
                };
                println!("  {}  {}", inst.version, inst.install_path.display());
                if !active.is_empty() {
                    println!("      ^ active in PATH");
                }
            }
        }

        if let Some(system) = &result.system_cuda {
            if let (Some(nvcc_ver), Some(path)) = (&system.nvcc_version, &system.nvcc_path) {
                let in_list = result.installations.iter().any(|i| i.version == *nvcc_ver);
                if !in_list {
                    println!(
                        "\n  System nvcc in PATH: {} at {}",
                        nvcc_ver,
                        path.display()
                    );
                }
            }
        }

        Ok(())
    }
}

pub struct DownloadHandler {
    args: DownloadArgs,
}

impl DownloadHandler {
    pub fn new(args: DownloadArgs) -> Self {
        Self { args }
    }
}

#[async_trait]
impl CommandHandler for DownloadHandler {
    async fn execute(&self) -> CudaMgrResult<()> {
        let versions: Vec<String> = if self.args.all {
            REGISTRY.available_cuda_versions()
        } else {
            self.args.versions.clone()
        };

        if versions.is_empty() {
            OutputFormatter::warning("No versions to download");
            return Ok(());
        }

        let base_dir = self.args.output_dir.clone().unwrap_or_else(|| {
            CompatibilityRegistry::cache_path()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("downloads")
        });

        std::fs::create_dir_all(&base_dir)
            .map_err(|e| CudaMgrError::Cli(format!("Create output dir: {}", e)))?;

        let downloader = PackageDownloader::new();
        let client = reqwest::Client::new();
        let base_url = PackageDownloader::redist_base_url();

        OutputFormatter::section("Downloading CUDA redistributables");
        println!("  Output directory: {}", base_dir.display());
        println!("  Versions: {}", versions.join(", "));
        println!();

        let mut total_files = 0usize;
        let mut failed = Vec::new();

        for version in &versions {
            let (full_version, paths) =
                match redist::resolve_version_to_redist_paths(version, &client).await {
                    Ok(x) => x,
                    Err(e) => {
                        OutputFormatter::warning(&format!("{}: {}", version, e));
                        failed.push(version.clone());
                        continue;
                    }
                };

            let version_dir = base_dir.join(&full_version);
            std::fs::create_dir_all(&version_dir)
                .map_err(|e| CudaMgrError::Cli(format!("Create version dir: {}", e)))?;

            for rel_path in &paths {
                let url = format!("{}/{}", base_url, rel_path);
                let filename = rel_path.rsplit('/').next().unwrap_or(rel_path);
                let dest = version_dir.join(filename);
                total_files += 1;
                print!("  [{}] {} ... ", full_version, filename);
                std::io::Write::flush(&mut std::io::stdout()).ok();
                match downloader.download(&url, &dest).await {
                    Ok(()) => println!("OK"),
                    Err(e) => {
                        println!("FAILED: {}", e);
                        failed.push(version.clone());
                    }
                }
            }
        }

        println!();
        OutputFormatter::success(&format!(
            "Downloaded {} files for {} version(s)",
            total_files,
            versions.len().saturating_sub(failed.len())
        ));
        if !failed.is_empty() {
            OutputFormatter::warning(&format!("Some versions had errors: {}", failed.join(", ")));
        }

        Ok(())
    }
}

pub struct UninstallHandler {
    args: UninstallArgs,
}

impl UninstallHandler {
    pub fn new(args: UninstallArgs) -> Self {
        Self { args }
    }
}

#[async_trait]
impl CommandHandler for UninstallHandler {
    async fn execute(&self) -> CudaMgrResult<()> {
        tracing::info!("Uninstalling CUDA version: {}", self.args.version);
        OutputFormatter::info(&format!("Uninstalling CUDA version {}", self.args.version));

        // TODO: Implement actual uninstall functionality
        OutputFormatter::warning("Uninstall command implementation pending");
        Err(CudaMgrError::Cli(
            "Uninstall command not yet implemented".to_string(),
        ))
    }
}

pub struct LogsHandler {
    args: LogsArgs,
}

impl LogsHandler {
    pub fn new(args: LogsArgs) -> Self {
        Self { args }
    }
}

#[async_trait]
impl CommandHandler for LogsHandler {
    async fn execute(&self) -> CudaMgrResult<()> {
        tracing::info!("Showing {} log lines", self.args.lines);
        OutputFormatter::info(&format!("Showing {} log lines", self.args.lines));

        // TODO: Implement actual logs functionality
        OutputFormatter::warning("Logs command implementation pending");
        Err(CudaMgrError::Cli(
            "Logs command not yet implemented".to_string(),
        ))
    }
}

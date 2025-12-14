use clap::Subcommand;
use async_trait::async_trait;
use crate::error::{CudaMgrResult, CudaMgrError};
use crate::cli::output::OutputFormatter;

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
        
        // Basic version format validation (e.g., "11.8", "12.0")
        if !self.version.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err(CudaMgrError::Cli("Invalid version format. Use format like '11.8' or '12.0'".to_string()));
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
            return Err(CudaMgrError::Cli("Invalid version format. Use format like '11.8' or '12.0'".to_string()));
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
            return Err(CudaMgrError::Cli("Invalid version format. Use format like '11.8' or '12.0'".to_string()));
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
            return Err(CudaMgrError::Cli("Number of lines must be greater than 0".to_string()));
        }
        
        if self.lines > 10000 {
            return Err(CudaMgrError::Cli("Number of lines cannot exceed 10000".to_string()));
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
        use crate::system::SystemReportGenerator;
        
        tracing::info!("Running system doctor with verbose: {}", self.args.verbose);
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
                Err(CudaMgrError::System(crate::error::SystemError::Incompatible(
                    "System compatibility check failed".to_string()
                )))
            }
            crate::system::CompatibilityStatus::PrerequisitesMissing => {
                OutputFormatter::warning("System is compatible but requires setup (Admin/Compiler)");
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
        println!("  OS: {} {}", report.system_info.distro.name, report.system_info.distro.version);
        
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
            println!("  Compiler: {} {} {}", 
                if compiler.is_compatible { "✅" } else { "⚠️" },
                compiler.name, 
                compiler.version);
        } else {
            println!("  Compiler: ❌ Not detected");
        }
        
        println!("  Storage: {} GB available", report.system_info.storage.available_space_gb);
        println!("  Admin: {}", if report.system_info.security.has_admin_privileges { "✅" } else { "❌" });
        
        // CUDA installations
        if !report.cuda_detection.installations.is_empty() {
            println!("\nExisting CUDA Installations:");
            for installation in &report.cuda_detection.installations {
                println!("  {} at {}", installation.version, installation.install_path.display());
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
                println!("  ... and {} more (use --verbose for full details)", 
                    report.recommendations.len() - 3);
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
        Err(CudaMgrError::Cli("Install command not yet implemented".to_string()))
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
        Err(CudaMgrError::Cli("Use command not yet implemented".to_string()))
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
        OutputFormatter::info("Listing CUDA versions...");
        
        // TODO: Implement actual list functionality
        OutputFormatter::warning("List command implementation pending");
        Err(CudaMgrError::Cli("List command not yet implemented".to_string()))
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
        Err(CudaMgrError::Cli("Uninstall command not yet implemented".to_string()))
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
        Err(CudaMgrError::Cli("Logs command not yet implemented".to_string()))
    }
}
use thiserror::Error;

/// Main error type for CudaMgr operations
#[derive(Debug, Error)]
pub enum CudaMgrError {
    #[error("System check failed: {0}")]
    System(#[from] SystemError),

    #[error("Installation failed: {0}")]
    Install(#[from] InstallError),

    #[error("Version management failed: {0}")]
    Version(#[from] VersionError),

    #[error("Configuration failed: {0}")]
    Config(#[from] ConfigError),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CLI error: {0}")]
    Cli(String),
}

/// System-related errors
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("GPU detection failed: {0}")]
    GpuDetection(String),

    #[error("Driver detection failed: {0}")]
    DriverDetection(String),

    #[error("Compiler detection failed: {0}")]
    CompilerDetection(String),

    #[error("Distribution detection failed: {0}")]
    DistroDetection(String),

    #[error("Storage check failed: {0}")]
    StorageCheck(String),

    #[error("Security check failed: {0}")]
    SecurityCheck(String),

    #[error("Compatibility check failed: {0}")]
    CompatibilityCheck(String),

    #[error("System is incompatible: {0}")]
    Incompatible(String),

    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Installation-related errors
#[derive(Debug, Error)]
pub enum InstallError {
    #[error("Download failed: {0}")]
    Download(String),

    #[error("Installation failed: {0}")]
    Installation(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Cleanup failed: {0}")]
    Cleanup(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),
}

/// Version management errors
#[derive(Debug, Error)]
pub enum VersionError {
    #[error("Version not found: {0}")]
    NotFound(String),

    #[error("Version switching failed: {0}")]
    SwitchFailed(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Version resolution failed: {0}")]
    Resolution(String),
}

/// Configuration management errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment configuration failed: {0}")]
    Environment(String),

    #[error("PATH configuration failed: {0}")]
    Path(String),

    #[error("Symlink operation failed: {0}")]
    Symlink(String),

    #[error("Backup operation failed: {0}")]
    Backup(String),

    #[error("Shell configuration failed: {0}")]
    Shell(String),
}

/// Convenient result type for CudaMgr operations
pub type CudaMgrResult<T> = Result<T, CudaMgrError>;

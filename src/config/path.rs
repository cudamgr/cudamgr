use crate::error::{ConfigError, CudaMgrResult};
use std::path::{Path, PathBuf};

/// PATH manipulation utilities
pub struct PathManager;

impl PathManager {
    /// Create a new PATH manager
    pub fn new() -> Self {
        Self
    }

    /// Add CUDA bin path to user PATH (and remove other cudamgr paths so only one is active).
    /// On Windows: updates HKCU\Environment\Path. On Unix: no-op (caller can print shell instructions).
    pub async fn add_cuda_to_path(&self, cuda_bin_path: &Path) -> CudaMgrResult<()> {
        let path_str = cuda_bin_path
            .to_str()
            .ok_or_else(|| ConfigError::Path("Invalid path encoding".to_string()))?
            .to_string();

        #[cfg(windows)]
        {
            self.add_cuda_to_path_windows(&path_str)?;
        }

        #[cfg(not(windows))]
        {
            let _ = path_str;
            tracing::info!("PATH update on Unix not implemented; user should add to shell config");
        }

        Ok(())
    }

    #[cfg(windows)]
    fn add_cuda_to_path_windows(&self, cuda_bin_path: &str) -> CudaMgrResult<()> {
        use winreg::RegKey;

        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags(
                "Environment",
                winreg::enums::KEY_READ | winreg::enums::KEY_WRITE,
            )
            .map_err(|e| ConfigError::Path(format!("Open Environment key: {}", e)))?;

        let path_value: String = env.get_value("Path").unwrap_or_else(|_| String::new());

        // Remove any existing cudamgr paths so we don't accumulate
        let cudamgr_marker = ".cudamgr";
        let entries: Vec<&str> = path_value
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|s| !s.contains(cudamgr_marker))
            .collect();

        // Prepend new path so it takes precedence
        let mut new_path = cuda_bin_path.to_string();
        if !entries.is_empty() {
            new_path.push(';');
            new_path.push_str(&entries.join(";"));
        }

        env.set_value("Path", &new_path)
            .map_err(|e| ConfigError::Path(format!("Write Path: {}", e)))?;

        // New terminals will see the updated PATH; current terminal unchanged
        Ok(())
    }

    /// Remove CUDA paths from system PATH
    pub async fn remove_cuda_from_path(&self, cuda_bin_path: &Path) -> CudaMgrResult<()> {
        #[cfg(windows)]
        {
            self.remove_cuda_from_path_windows(cuda_bin_path).await?;
        }
        #[cfg(not(windows))]
        {
            let _ = cuda_bin_path;
        }
        Ok(())
    }

    #[cfg(windows)]
    async fn remove_cuda_from_path_windows(&self, cuda_bin_path: &Path) -> CudaMgrResult<()> {
        use winreg::RegKey;

        let path_str = cuda_bin_path.to_string_lossy();
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags(
                "Environment",
                winreg::enums::KEY_READ | winreg::enums::KEY_WRITE,
            )
            .map_err(|e| ConfigError::Path(format!("Open Environment key: {}", e)))?;

        let path_value: String = env.get_value("Path").unwrap_or_default();
        let entries: Vec<String> = path_value
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|s| !s.contains(path_str.as_ref()))
            .map(String::from)
            .collect();
        let new_path = entries.join(";");
        env.set_value("Path", &new_path)
            .map_err(|e| ConfigError::Path(format!("Write Path: {}", e)))?;
        Ok(())
    }

    /// Get current user PATH entries (Windows: from registry; Unix: from env)
    pub fn get_current_path(&self) -> CudaMgrResult<Vec<PathBuf>> {
        #[cfg(windows)]
        {
            let path_value: String = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
                .open_subkey("Environment")
                .and_then(|e| e.get_value("Path"))
                .unwrap_or_default();
            let entries: Vec<PathBuf> = path_value
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .collect();
            Ok(entries)
        }
        #[cfg(not(windows))]
        {
            let path = std::env::var_os("PATH").unwrap_or_default();
            let entries: Vec<PathBuf> = std::env::split_paths(&path).collect();
            Ok(entries)
        }
    }

    /// Check if CUDA path is in user PATH
    pub fn is_cuda_in_path(&self, cuda_bin_path: &PathBuf) -> CudaMgrResult<bool> {
        let paths = self.get_current_path()?;
        let canonical = cuda_bin_path
            .canonicalize()
            .unwrap_or_else(|_| cuda_bin_path.clone());
        for p in &paths {
            if p.canonicalize().ok().as_ref() == Some(&canonical) {
                return Ok(true);
            }
            if p == cuda_bin_path {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Default for PathManager {
    fn default() -> Self {
        Self::new()
    }
}

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::{SystemError, CudaMgrResult};

/// Storage and disk space information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub available_space_gb: u64,
    pub total_space_gb: u64,
    pub install_path: String,
    pub has_sufficient_space: bool,
}

impl StorageInfo {
    /// Create a new StorageInfo instance
    pub fn new(
        available_space_gb: u64,
        total_space_gb: u64,
        install_path: String,
        required_space_gb: u64,
    ) -> Self {
        Self {
            available_space_gb,
            total_space_gb,
            install_path,
            has_sufficient_space: available_space_gb >= required_space_gb,
        }
    }

    /// Detect storage information for a given path
    pub fn detect(install_path: &Path) -> CudaMgrResult<Self> {
        let path_str = install_path.to_string_lossy().to_string();
        
        // Get disk space information
        let (available_bytes, total_bytes) = Self::get_disk_space(install_path)?;
        
        let available_gb = available_bytes / (1024 * 1024 * 1024);
        let total_gb = total_bytes / (1024 * 1024 * 1024);
        
        // CUDA typically requires 3-6 GB depending on version and components
        let required_gb = 6;
        
        Ok(Self::new(
            available_gb,
            total_gb,
            path_str,
            required_gb,
        ))
    }

    /// Get default CUDA installation path for the current platform
    pub fn get_default_cuda_path() -> PathBuf {
        #[cfg(target_os = "linux")]
        {
            PathBuf::from("/usr/local/cuda")
        }
        #[cfg(target_os = "windows")]
        {
            PathBuf::from("C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA")
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            PathBuf::from("/usr/local/cuda")
        }
    }

    /// Get disk space information for a path
    fn get_disk_space(path: &Path) -> CudaMgrResult<(u64, u64)> {
        #[cfg(unix)]
        {
            Self::get_disk_space_unix(path)
        }
        #[cfg(windows)]
        {
            Self::get_disk_space_windows(path)
        }
    }

    /// Get disk space on Unix systems using statvfs
    #[cfg(unix)]
    fn get_disk_space_unix(path: &Path) -> CudaMgrResult<(u64, u64)> {
        use std::ffi::CString;
        use std::mem;
        use std::os::unix::ffi::OsStrExt;

        // Create the path, using parent if the path doesn't exist
        let check_path = if path.exists() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(Path::new("/")).to_path_buf()
        };

        let path_cstring = CString::new(check_path.as_os_str().as_bytes())
            .map_err(|e| SystemError::StorageCheck(format!("Invalid path: {}", e)))?;

        let mut statvfs: libc::statvfs = unsafe { mem::zeroed() };
        
        let result = unsafe { libc::statvfs(path_cstring.as_ptr(), &mut statvfs) };
        
        if result != 0 {
            return Err(SystemError::StorageCheck("Failed to get filesystem statistics".to_string()).into());
        }

        let block_size = statvfs.f_frsize as u64;
        let available_blocks = statvfs.f_bavail as u64;
        let total_blocks = statvfs.f_blocks as u64;

        let available_bytes = available_blocks * block_size;
        let total_bytes = total_blocks * block_size;

        Ok((available_bytes, total_bytes))
    }

    /// Get disk space on Windows systems
    #[cfg(windows)]
    fn get_disk_space_windows(path: &Path) -> CudaMgrResult<(u64, u64)> {
        // Simplified implementation for now - would use Windows APIs in production
        Ok((50 * 1024 * 1024 * 1024, 100 * 1024 * 1024 * 1024)) // Mock 50GB available, 100GB total
    }

    /// Check if there's enough space for installation
    pub fn check_space_requirement(&self, required_gb: u64) -> bool {
        self.available_space_gb >= required_gb
    }

    /// Get human-readable space information
    pub fn format_space_info(&self) -> String {
        format!(
            "{:.1} GB available / {:.1} GB total",
            self.available_space_gb as f64, self.total_space_gb as f64
        )
    }
}
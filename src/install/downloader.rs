use crate::error::{CudaMgrResult, InstallError};
use std::io::Write;
use std::path::Path;

const REDIST_BASE: &str = "https://developer.download.nvidia.com/compute/cuda/redist";

/// Package downloader with progress tracking
pub struct PackageDownloader {
    client: reqwest::Client,
}

impl PackageDownloader {
    /// Create a new package downloader
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("cudamgr/1.0")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    /// Download a package from URL to destination path with progress
    pub async fn download(&self, url: &str, destination: &Path) -> CudaMgrResult<()> {
        let mut response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| InstallError::Download(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(
                InstallError::Download(format!("HTTP {} from {}", response.status(), url)).into(),
            );
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut file = std::fs::File::create(destination)
            .map_err(|e| InstallError::Download(format!("Create file: {}", e)))?;

        let mut downloaded: u64 = 0;
        let mut last_pct = 0u64;

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| InstallError::Download(format!("Stream: {}", e)))?
        {
            file.write_all(&chunk)
                .map_err(|e| InstallError::Download(format!("Write: {}", e)))?;
            downloaded += chunk.len() as u64;
            if total_size > 0 {
                let pct = (downloaded * 100) / total_size;
                if pct >= last_pct + 10 || downloaded == total_size {
                    last_pct = pct;
                    tracing::debug!("Download progress: {}%", pct);
                }
            }
        }

        Ok(())
    }

    /// Base URL for NVIDIA redist packages
    pub fn redist_base_url() -> &'static str {
        REDIST_BASE
    }

    /// Resume a partial download
    pub async fn resume_download(&self, url: &str, destination: &Path) -> CudaMgrResult<()> {
        // Fall back to full download; proper Range support can be added later
        self.download(url, destination).await
    }

    /// Verify download integrity
    pub async fn verify_integrity(
        &self,
        _file_path: &Path,
        _expected_hash: &str,
    ) -> CudaMgrResult<bool> {
        // TODO: Implement SHA256 verification
        Ok(true)
    }
}

impl Default for PackageDownloader {
    fn default() -> Self {
        Self::new()
    }
}

//! Resolve CUDA version to NVIDIA redist download URLs.
//!
//! Uses https://developer.download.nvidia.com/compute/cuda/redist/ manifest (redistrib_X.Y.Z.json)
//! to get direct download URLs for the current platform. "Download in one go" fetches these files.

use crate::error::{CudaMgrResult, InstallError};

const REDIST_INDEX_URL: &str = "https://developer.download.nvidia.com/compute/cuda/redist/";

#[cfg(target_os = "windows")]
const PLATFORM_KEY: &str = "windows-x86_64";

#[cfg(not(target_os = "windows"))]
const PLATFORM_KEY: &str = "linux-x86_64";

/// Top-level keys in redistrib_X.Y.Z.json are component names; values are objects
/// that may contain platform keys like "linux-x86_64" or "windows-x86_64".
fn extract_platform_artifacts(json: &serde_json::Value) -> Vec<String> {
    let mut paths = Vec::new();
    let obj = match json.as_object() {
        Some(o) => o,
        None => return paths,
    };
    for (key, value) in obj {
        if key == "release_date" || key == "release_label" || key == "release_product" {
            continue;
        }
        let platform = value.get(PLATFORM_KEY);
        if let Some(art) = platform.and_then(|p| p.get("relative_path")) {
            if let Some(s) = art.as_str() {
                paths.push(s.to_string());
            }
        }
    }
    paths
}

/// Try to find a full version (X.Y.Z) that has a redist manifest. Tries latest patch first.
fn resolve_version_to_patch(version: &str) -> Vec<String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 3 {
        return vec![version.to_string()];
    }
    if parts.len() != 2 {
        return vec![];
    }
    let (major, minor) = (parts[0], parts[1]);
    let mut candidates = Vec::new();
    for patch in (0..=9).rev() {
        candidates.push(format!("{}.{}.{}", major, minor, patch));
    }
    candidates
}

/// Fetch redistrib_X.Y.Z.json and return relative paths for current platform.
pub async fn get_redist_download_paths(
    full_version: &str,
    client: &reqwest::Client,
) -> CudaMgrResult<Vec<String>> {
    let url = format!("{}redistrib_{}.json", REDIST_INDEX_URL, full_version);
    let body = client
        .get(&url)
        .send()
        .await
        .map_err(|e| InstallError::Download(format!("Fetch manifest: {}", e)))?
        .error_for_status()
        .map_err(|e| InstallError::Download(format!("Manifest HTTP: {}", e)))?
        .text()
        .await
        .map_err(|e| InstallError::Download(format!("Read manifest: {}", e)))?;

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| InstallError::Download(format!("Parse manifest: {}", e)))?;

    Ok(extract_platform_artifacts(&json))
}

/// Resolve a short version (e.g. "12.6") to a full version (e.g. "12.6.3") that has a redist manifest,
/// and return the list of relative paths to download for the current platform.
pub async fn resolve_version_to_redist_paths(
    version: &str,
    client: &reqwest::Client,
) -> CudaMgrResult<(String, Vec<String>)> {
    let candidates = resolve_version_to_patch(version);
    for full in &candidates {
        match get_redist_download_paths(full, client).await {
            Ok(paths) if !paths.is_empty() => return Ok((full.clone(), paths)),
            Ok(_) => continue,
            Err(_) => continue,
        }
    }
    Err(InstallError::Download(format!(
        "No redist manifest found for version {} (tried {})",
        version,
        candidates.join(", ")
    ))
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_version_to_patch() {
        let c = resolve_version_to_patch("12.6");
        assert!(c.contains(&"12.6.3".to_string()));
        assert_eq!(resolve_version_to_patch("12.6.3"), vec!["12.6.3"]);
    }

    #[test]
    fn test_extract_platform_artifacts() {
        let json = serde_json::json!({
            "release_label": "12.6.3",
            "cuda_cudart": {
                "linux-x86_64": { "relative_path": "cuda_cudart/linux-x86_64/foo.tar.xz" },
                "windows-x86_64": { "relative_path": "cuda_cudart/windows-x86_64/bar.zip" }
            }
        });
        let paths = extract_platform_artifacts(&json);
        #[cfg(target_os = "windows")]
        assert!(paths.iter().any(|p| p.contains("windows-x86_64")));
        #[cfg(not(target_os = "windows"))]
        assert!(paths.iter().any(|p| p.contains("linux-x86_64")));
    }
}

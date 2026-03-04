pub mod cleanup;
pub mod downloader;
pub mod installer;
pub mod redist;
pub mod validator;

use crate::config::CudaMgrConfig;
use crate::error::{CudaMgrError, CudaMgrResult, InstallError};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

/// Installation plan containing all necessary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationPlan {
    pub cuda_version: String,
    /// First/legacy single URL (e.g. first artifact)
    pub download_url: String,
    /// All redistributable artifact URLs to download and extract (real install)
    pub download_urls: Vec<String>,
    pub install_path: PathBuf,
    pub required_driver: Option<String>,
    pub dependencies: Vec<Dependency>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub required: bool,
}

/// Installer trait for platform-specific installation
pub trait Installer {
    async fn create_plan(&self, version: &str) -> CudaMgrResult<InstallationPlan>;
    async fn execute_plan(&self, plan: &InstallationPlan) -> CudaMgrResult<()>;
    async fn validate_installation(&self, path: &std::path::Path) -> CudaMgrResult<bool>;
}

/// Default installer implementation
pub struct DefaultInstaller;

impl Installer for DefaultInstaller {
    async fn create_plan(&self, version: &str) -> CudaMgrResult<InstallationPlan> {
        let config = CudaMgrConfig::load()?;
        config.ensure_directories()?;

        let client = reqwest::Client::new();
        let (full_version, paths) =
            redist::resolve_version_to_redist_paths(version, &client).await?;

        let install_path = config.install_dir.join(&full_version);
        let base = redist::REDIST_INDEX_URL;
        let download_urls: Vec<String> = paths
            .iter()
            .map(|p| format!("{}{}", base, p))
            .collect();
        let download_url = download_urls.first().cloned().unwrap_or_default();

        Ok(InstallationPlan {
            cuda_version: full_version,
            download_url,
            download_urls,
            install_path,
            required_driver: None,
            dependencies: vec![],
        })
    }

    async fn execute_plan(&self, plan: &InstallationPlan) -> CudaMgrResult<()> {
        std::fs::create_dir_all(&plan.install_path).map_err(|e| {
            CudaMgrError::Install(InstallError::Installation(format!(
                "Failed to create install directory: {}",
                e
            )))
        })?;

        if plan.download_urls.is_empty() {
            return Err(CudaMgrError::Install(InstallError::Installation(
                "No redist artifacts in plan; cannot install toolkit.".to_string(),
            )));
        }

        let downloader = downloader::PackageDownloader::new();
        let cache_dir = CudaMgrConfig::load()
            .map(|c| c.cache_dir)
            .unwrap_or_else(|_| plan.install_path.join("..").join(".cache").join(&plan.cuda_version));
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            CudaMgrError::Install(InstallError::Download(format!(
                "Failed to create cache dir: {}",
                e
            )))
        })?;

        let total = plan.download_urls.len();
        for (i, url) in plan.download_urls.iter().enumerate() {
            let filename = url
                .rsplit('/')
                .next()
                .unwrap_or("archive")
                .to_string();
            let dest_file = cache_dir.join(&filename);
            eprintln!("  [{}/{}] Downloading {} ...", i + 1, total, filename);
            downloader.download(url, &dest_file).await?;
            eprintln!("  [{}/{}] Extracting {} ...", i + 1, total, filename);
            extract_and_merge(&dest_file, &plan.install_path)?;
            let _ = std::fs::remove_file(&dest_file);
        }

        // If nvcc is still missing (e.g. not in some manifests or wrong layout), install cuda_nvcc explicitly
        if !nvcc_binary_path(&plan.install_path).exists() {
            tracing::info!("nvcc not found after install; fetching cuda_nvcc component...");
            let client = reqwest::Client::new();
            if let Ok(manifest) = redist::get_redist_manifest(&plan.cuda_version, &client).await {
                let nvcc_paths =
                    redist::get_component_paths_from_manifest(&manifest, "cuda_nvcc");
                for rel in nvcc_paths {
                    let url = format!("{}{}", redist::REDIST_INDEX_URL, rel);
                    let filename = url.rsplit('/').next().unwrap_or("nvcc_archive").to_string();
                    let dest_file = cache_dir.join(&filename);
                    downloader.download(&url, &dest_file).await?;
                    extract_and_merge(&dest_file, &plan.install_path)?;
                    let _ = std::fs::remove_file(&dest_file);
                }
            }
            if !nvcc_binary_path(&plan.install_path).exists() {
                tracing::warn!(
                    "nvcc still not found at {} (cuda_nvcc may not be available for this version)",
                    nvcc_binary_path(&plan.install_path).display()
                );
            }
        }

        tracing::info!(
            "CUDA {} installed at {}",
            plan.cuda_version,
            plan.install_path.display()
        );
        Ok(())
    }

    async fn validate_installation(&self, _path: &std::path::Path) -> CudaMgrResult<bool> {
        // TODO: Implement installation validation
        Err(
            InstallError::Validation("Installation validation not yet implemented".to_string())
                .into(),
        )
    }
}

/// Expected path to the nvcc binary for a given install path.
fn nvcc_binary_path(install_path: &Path) -> PathBuf {
    let bin = install_path.join("bin");
    #[cfg(windows)]
    return bin.join("nvcc.exe");
    #[cfg(not(windows))]
    return bin.join("nvcc");
}

/// Extract an archive (.zip or .tar.xz) to a temp dir, then merge contents into target.
/// NVIDIA redist archives have one top-level dir (e.g. *-archive) with bin/, lib/, include/.
fn extract_and_merge(archive_path: &Path, target: &Path) -> CudaMgrResult<()> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let parent = archive_path
        .parent()
        .unwrap_or(Path::new("."));
    let stem = archive_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("extract");
    // For .tar.xz the stem is e.g. "cuda_nvcc-...-archive" (one less extension)
    let extract_dir = parent.join(format!("{}_extract", stem));

    if archive_path.to_string_lossy().ends_with(".zip") {
        extract_zip(archive_path, &extract_dir)?;
    } else if ext == "xz" || archive_path.to_string_lossy().ends_with(".tar.xz") {
        extract_tar_xz(archive_path, &extract_dir)?;
    } else {
        return Err(InstallError::Installation(format!(
            "Unsupported archive format: {}",
            archive_path.display()
        ))
        .into());
    }

    // Merge: find the single top-level dir (e.g. *-archive) and copy its contents into target.
    // NVIDIA zips can be either: (1) one wrapper dir *-archive with bin/lib/include inside,
    // or (2) just bin/ (and/or lib/, include/) at root. We must merge so files end up in target/bin etc.
    let entries: Vec<_> = std::fs::read_dir(&extract_dir)
        .map_err(|e| {
            CudaMgrError::Install(InstallError::Installation(format!(
                "Read extract dir: {}",
                e
            )))
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            CudaMgrError::Install(InstallError::Installation(format!("Read dir entry: {}", e)))
        })?;

    let content_root = if entries.len() == 1 {
        let e = &entries[0];
        let p = e.path();
        if p.is_dir() {
            p
        } else {
            extract_dir.clone()
        }
    } else {
        extract_dir.clone()
    };

    // If the only top-level entry is "bin", "lib", or "include", merge it into target/bin etc.,
    // so we don't put nvcc.exe directly in target/.
    let merge_dest = content_root
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| {
            match s {
                "bin" => target.join("bin"),
                "lib" => target.join("lib"),
                "include" => target.join("include"),
                _ => target.to_path_buf(),
            }
        })
        .unwrap_or_else(|| target.to_path_buf());

    merge_dir_into(&content_root, &merge_dest)?;
    let _ = std::fs::remove_dir_all(&extract_dir);
    Ok(())
}

fn extract_zip(zip_path: &Path, dest: &Path) -> CudaMgrResult<()> {
    std::fs::create_dir_all(dest).map_err(|e| {
        CudaMgrError::Install(InstallError::Installation(format!("Create dir: {}", e)))
    })?;
    let file = std::fs::File::open(zip_path).map_err(|e| {
        CudaMgrError::Install(InstallError::Download(format!("Open zip: {}", e)))
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        CudaMgrError::Install(InstallError::Installation(format!("Invalid zip: {}", e)))
    })?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            CudaMgrError::Install(InstallError::Installation(format!("Zip entry: {}", e)))
        })?;
        // Normalize path: zip may use backslashes on Windows; use forward slash for correct structure
        let name = entry.name().replace('\\', "/");
        let out_path = dest.join(&name);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| {
                CudaMgrError::Install(InstallError::Installation(format!("Mkdir: {}", e)))
            })?;
        } else {
            if let Some(p) = out_path.parent() {
                std::fs::create_dir_all(p).map_err(|e| {
                    CudaMgrError::Install(InstallError::Installation(format!("Mkdir: {}", e)))
                })?;
            }
            let mut out = std::fs::File::create(&out_path).map_err(|e| {
                CudaMgrError::Install(InstallError::Installation(format!("Create file: {}", e)))
            })?;
            io::copy(&mut entry, &mut out).map_err(|e| {
                CudaMgrError::Install(InstallError::Installation(format!("Copy: {}", e)))
            })?;
        }
    }
    Ok(())
}

fn extract_tar_xz(archive_path: &Path, dest: &Path) -> CudaMgrResult<()> {
    std::fs::create_dir_all(dest).map_err(|e| {
        CudaMgrError::Install(InstallError::Installation(format!("Create dir: {}", e)))
    })?;
    let file = std::fs::File::open(archive_path).map_err(|e| {
        CudaMgrError::Install(InstallError::Download(format!("Open tar.xz: {}", e)))
    })?;
    let dec = xz2::read::XzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);
    archive.unpack(dest).map_err(|e| {
        CudaMgrError::Install(InstallError::Installation(format!("Unpack tar: {}", e)))
    })?;
    Ok(())
}

/// Recursively copy contents of src into target (merge: existing dirs are merged).
fn merge_dir_into(src: &Path, target: &Path) -> CudaMgrResult<()> {
    for entry in std::fs::read_dir(src).map_err(|e| {
        CudaMgrError::Install(InstallError::Installation(format!("Read dir: {}", e)))
    })? {
        let entry = entry.map_err(|e| {
            CudaMgrError::Install(InstallError::Installation(format!("Dir entry: {}", e)))
        })?;
        let path = entry.path();
        let name = entry.file_name();
        let dest = target.join(&name);
        if path.is_dir() {
            std::fs::create_dir_all(&dest).map_err(|e| {
                CudaMgrError::Install(InstallError::Installation(format!("Mkdir: {}", e)))
            })?;
            merge_dir_into(&path, &dest)?;
        } else {
            if let Some(p) = dest.parent() {
                std::fs::create_dir_all(p).map_err(|e| {
                    CudaMgrError::Install(InstallError::Installation(format!("Mkdir: {}", e)))
                })?;
            }
            std::fs::copy(&path, &dest).map_err(|e| {
                CudaMgrError::Install(InstallError::Installation(format!("Copy: {}", e)))
            })?;
        }
    }
    Ok(())
}

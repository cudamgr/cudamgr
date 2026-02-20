use crate::error::{CudaMgrResult, SystemError};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Compiler information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerInfo {
    pub name: String,
    pub version: String,
    pub is_compatible: bool,
    pub path: Option<String>,
    pub is_in_path: bool,
}

impl CompilerInfo {
    /// Create a new CompilerInfo instance
    pub fn new(
        name: String,
        version: String,
        is_compatible: bool,
        path: Option<String>,
        is_in_path: bool,
    ) -> Self {
        Self {
            name,
            version,
            is_compatible,
            path,
            is_in_path,
        }
    }

    /// Detect available compilers
    pub fn detect() -> CudaMgrResult<Vec<Self>> {
        let mut compilers = Vec::new();

        // Detect GCC on Linux/macOS
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let Ok(gcc_info) = Self::detect_gcc() {
            compilers.push(gcc_info);
        }

        // Detect Clang on Linux/macOS
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let Ok(clang_info) = Self::detect_clang() {
            compilers.push(clang_info);
        }

        // Detect MSVC on Windows
        #[cfg(target_os = "windows")]
        if let Ok(msvc_info) = Self::detect_msvc() {
            compilers.push(msvc_info);
        }

        Ok(compilers)
    }

    /// Detect GCC compiler
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn detect_gcc() -> CudaMgrResult<Self> {
        let output = Command::new("gcc")
            .arg("--version")
            .output()
            .map_err(|e| SystemError::CompilerDetection(format!("Failed to run gcc: {}", e)))?;

        if !output.status.success() {
            return Err(SystemError::CompilerDetection("gcc command failed".to_string()).into());
        }

        let output_str = String::from_utf8(output.stdout)
            .map_err(|e| SystemError::CompilerDetection(format!("Invalid gcc output: {}", e)))?;

        let version = Self::parse_gcc_version(&output_str)?;
        let is_compatible = Self::is_gcc_compatible(&version);

        // Get gcc path
        let path = Self::get_command_path("gcc");

        Ok(Self::new(
            "GCC".to_string(),
            version,
            is_compatible,
            path,
            true,
        ))
    }

    /// Detect Clang compiler
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn detect_clang() -> CudaMgrResult<Self> {
        let output = Command::new("clang")
            .arg("--version")
            .output()
            .map_err(|e| SystemError::CompilerDetection(format!("Failed to run clang: {}", e)))?;

        if !output.status.success() {
            return Err(SystemError::CompilerDetection("clang command failed".to_string()).into());
        }

        let output_str = String::from_utf8(output.stdout)
            .map_err(|e| SystemError::CompilerDetection(format!("Invalid clang output: {}", e)))?;

        let version = Self::parse_clang_version(&output_str)?;
        let is_compatible = Self::is_clang_compatible(&version);

        // Get clang path
        let path = Self::get_command_path("clang");

        Ok(Self::new(
            "Clang".to_string(),
            version,
            is_compatible,
            path,
            true,
        ))
    }

    /// Detect MSVC compiler on Windows
    #[cfg(target_os = "windows")]
    fn detect_msvc() -> CudaMgrResult<Self> {
        // 1. Try to detect Visual Studio Build Tools via `cl` command first (PATH check)
        let output = Command::new("cl").output();

        if let Ok(out) = output {
            let output_str = String::from_utf8_lossy(&out.stderr);
            if output_str.contains("Microsoft") {
                let version = Self::parse_msvc_version(&output_str)?;
                let is_compatible = Self::is_msvc_compatible(&version);
                let path = Self::get_command_path("cl");

                return Ok(Self::new(
                    "MSVC".to_string(),
                    version,
                    is_compatible,
                    path,
                    true,
                ));
            }
        }

        // 2. If not in PATH, try to find it via Visual Studio installation
        if let Ok(Some(vs_info)) = super::visual_studio::VisualStudioInfo::detect() {
            if vs_info.has_cpp_tools {
                // Construct path to cl.exe
                // Path format: <InstallPath>\VC\Tools\MSVC\<Version>\bin\Hostx64\x64\cl.exe
                let vc_path = vs_info.install_path.join("VC").join("Tools").join("MSVC");

                if vc_path.exists() {
                    // Find the latest version directory
                    if let Ok(entries) = std::fs::read_dir(&vc_path) {
                        let mut versions: Vec<_> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().is_dir())
                            .map(|e| e.file_name().to_string_lossy().to_string())
                            .collect();

                        // Sort to get latest version (compare full semantic version)
                        versions.sort_by(|a, b| {
                            let parts_a: Vec<u32> =
                                a.split('.').filter_map(|s| s.parse().ok()).collect();
                            let parts_b: Vec<u32> =
                                b.split('.').filter_map(|s| s.parse().ok()).collect();
                            let max_len = parts_a.len().max(parts_b.len());
                            for i in 0..max_len {
                                let va = parts_a.get(i).unwrap_or(&0);
                                let vb = parts_b.get(i).unwrap_or(&0);
                                match vb.cmp(va) {
                                    std::cmp::Ordering::Equal => continue,
                                    other => return other, // Descending
                                }
                            }
                            std::cmp::Ordering::Equal
                        });

                        if let Some(latest_ver) = versions.first() {
                            let cl_path = vc_path
                                .join(latest_ver)
                                .join("bin")
                                .join("Hostx64")
                                .join("x64")
                                .join("cl.exe");

                            if cl_path.exists() {
                                // Run this specific cl.exe to get details
                                let output = Command::new(&cl_path).output().map_err(|e| {
                                    SystemError::CompilerDetection(format!(
                                        "Failed to run absolute cl.exe: {}",
                                        e
                                    ))
                                })?;

                                let output_str = String::from_utf8_lossy(&output.stderr);
                                if output_str.contains("Microsoft") {
                                    let version = Self::parse_msvc_version(&output_str)?;
                                    let is_compatible = Self::is_msvc_compatible(&version);

                                    return Ok(Self::new(
                                        "MSVC".to_string(),
                                        version,
                                        is_compatible,
                                        Some(cl_path.to_string_lossy().to_string()),
                                        false,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(SystemError::CompilerDetection("MSVC not found".to_string()).into())
    }

    /// Parse GCC version from output
    pub fn parse_gcc_version(output: &str) -> CudaMgrResult<String> {
        let first_line = output
            .lines()
            .next()
            .ok_or_else(|| SystemError::CompilerDetection("Empty gcc output".to_string()))?;

        // Look for version pattern like "gcc (Ubuntu 9.4.0-1ubuntu1~20.04.1) 9.4.0"
        // We want the clean version number at the end, not the one in parentheses
        let words: Vec<&str> = first_line.split_whitespace().collect();

        // Find the last word that starts with a digit and looks like a version
        let version = words
            .iter()
            .rev()
            .find(|word| {
                word.chars().next().is_some_and(|c| c.is_ascii_digit()) && word.contains('.')
            })
            .ok_or_else(|| {
                SystemError::CompilerDetection("Could not parse gcc version".to_string())
            })?
            .to_string();

        Ok(version)
    }

    /// Parse Clang version from output
    pub fn parse_clang_version(output: &str) -> CudaMgrResult<String> {
        let first_line = output
            .lines()
            .next()
            .ok_or_else(|| SystemError::CompilerDetection("Empty clang output".to_string()))?;

        // Look for version pattern like "clang version 12.0.0"
        if let Some(version_start) = first_line.find("version ") {
            let version_part = &first_line[version_start + 8..];
            let version = version_part
                .split_whitespace()
                .next()
                .ok_or_else(|| {
                    SystemError::CompilerDetection("Could not parse clang version".to_string())
                })?
                .to_string();
            Ok(version)
        } else {
            Err(SystemError::CompilerDetection("Could not find clang version".to_string()).into())
        }
    }

    /// Parse MSVC version from output
    #[cfg(target_os = "windows")]
    fn parse_msvc_version(output: &str) -> CudaMgrResult<String> {
        // Look for version pattern in MSVC output
        for line in output.lines() {
            if line.contains("Version") {
                if let Some(version_start) = line.find("Version ") {
                    let version_part = &line[version_start + 8..];
                    let version = version_part
                        .split_whitespace()
                        .next()
                        .unwrap_or("unknown")
                        .to_string();
                    return Ok(version);
                }
            }
        }
        Ok("unknown".to_string())
    }

    /// Check if GCC version is compatible with CUDA
    pub fn is_gcc_compatible(version: &str) -> bool {
        if let Ok(major) = version.split('.').next().unwrap_or("0").parse::<u32>() {
            // CUDA 12.x supports GCC up to version 12.x
            // CUDA 11.x supports GCC up to version 11.x
            // We'll be conservative and support GCC 5.x through 12.x
            (5..=12).contains(&major)
        } else {
            false
        }
    }

    /// Check if Clang version is compatible with CUDA
    pub fn is_clang_compatible(version: &str) -> bool {
        if let Ok(major) = version.split('.').next().unwrap_or("0").parse::<u32>() {
            // CUDA typically supports Clang 6.0 to 15.x
            (6..=16).contains(&major)
        } else {
            false
        }
    }

    /// Check if MSVC version is compatible with CUDA
    #[cfg(target_os = "windows")]
    fn is_msvc_compatible(version: &str) -> bool {
        if let Ok(major) = version.split('.').next().unwrap_or("0").parse::<u32>() {
            // CUDA typically supports MSVC 2017 (19.1x) and later
            major >= 19
        } else {
            false
        }
    }

    /// Get the full path of a command
    fn get_command_path(command: &str) -> Option<String> {
        let lookup_cmd = if cfg!(windows) { "where" } else { "which" };
        Command::new(lookup_cmd)
            .arg(command)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|path| {
                // On Windows, `where` may return multiple lines; take the first
                path.lines().next().unwrap_or("").trim().to_string()
            })
            .filter(|path| !path.is_empty())
    }
}

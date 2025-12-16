use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use crate::error::CudaMgrResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualStudioInfo {
    pub is_installed: bool,
    pub name: String,
    pub version: String,
    pub install_path: PathBuf,
    pub has_cpp_tools: bool,
}

impl VisualStudioInfo {
    pub fn detect() -> CudaMgrResult<Option<Self>> {
        #[cfg(target_os = "windows")]
        {
            Self::detect_windows()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(None)
        }
    }

    #[cfg(target_os = "windows")]
    fn detect_windows() -> CudaMgrResult<Option<Self>> {
        // Use vswhere.exe which is standard for detecting VS 2017+
        // Usually located at %ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe
        
        let mut vswhere_path = PathBuf::from(std::env::var("ProgramFiles(x86)").unwrap_or(r"C:\Program Files (x86)".to_string()));
        vswhere_path.push("Microsoft Visual Studio");
        vswhere_path.push("Installer");
        vswhere_path.push("vswhere.exe");

        if !vswhere_path.exists() {
            // Try 64-bit Program Files just in case
            vswhere_path = PathBuf::from(std::env::var("ProgramFiles").unwrap_or(r"C:\Program Files".to_string()));
            vswhere_path.push("Microsoft Visual Studio");
            vswhere_path.push("Installer");
            vswhere_path.push("vswhere.exe");
        }

        if vswhere_path.exists() {
            let output = Command::new(vswhere_path)
                .args(&[
                    "-latest", 
                    "-products", "*", 
                    "-requires", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                    "-format", "json"
                ])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let json_str = String::from_utf8_lossy(&out.stdout);
                    // Simple JSON parsing without adding massive struct overhead for now
                    // The output is a JSON array of objects
                    
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json_str) {
                         if let Some(arr) = v.as_array() {
                             if let Some(first) = arr.first() {
                                 let name = first["displayName"].as_str().unwrap_or("Visual Studio").to_string();
                                 let version = first["catalog"]["productDisplayVersion"].as_str().unwrap_or("Unknown").to_string();
                                 let path_str = first["installationPath"].as_str().unwrap_or("");
                                 
                                 return Ok(Some(Self {
                                     is_installed: true,
                                     name,
                                     version,
                                     install_path: PathBuf::from(path_str),
                                     has_cpp_tools: true, // We queried with -requires check
                                 }));
                             }
                         }
                    }
                }
            }
        }
        
        Ok(None)
    }
}

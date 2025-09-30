use serde::{Deserialize, Serialize};

/// Compiler information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerInfo {
    pub name: String,
    pub version: String,
    pub is_compatible: bool,
    pub path: Option<String>,
}

impl CompilerInfo {
    /// Create a new CompilerInfo instance
    pub fn new(
        name: String,
        version: String,
        is_compatible: bool,
        path: Option<String>,
    ) -> Self {
        Self {
            name,
            version,
            is_compatible,
            path,
        }
    }
}
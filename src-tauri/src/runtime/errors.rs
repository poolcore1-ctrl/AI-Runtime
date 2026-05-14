use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureKind {
    TypeScript,
    Rust,
    Dependency,
    Syntax,
    Runtime,
    Hydration,
    BuildTool,
    Network,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureFingerprint {
    pub kind: FailureKind,
    pub code: Option<String>,
    pub message: String,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct ErrorClassifier;

impl ErrorClassifier {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes a set of log lines to produce a structural FailureFingerprint.
    pub fn classify(&self, logs: &[String]) -> FailureFingerprint {
        for line in logs {
            // TypeScript Error Pattern: error TS2339: Property 'x' does not exist on type 'Y'
            if line.contains("error TS") {
                let code = line.split("error ").nth(1)
                    .and_then(|s| s.split(':').next())
                    .map(|s| s.to_string());
                
                return FailureFingerprint {
                    kind: FailureKind::TypeScript,
                    code,
                    message: line.clone(),
                    metadata: std::collections::HashMap::new(),
                };
            }

            // Rust Error Pattern: error[E0308]: mismatched types
            if line.contains("error[E") {
                let code = line.split("error[").nth(1)
                    .and_then(|s| s.split(']').next())
                    .map(|s| s.to_string());

                return FailureFingerprint {
                    kind: FailureKind::Rust,
                    code,
                    message: line.clone(),
                    metadata: std::collections::HashMap::new(),
                };
            }

            // Dependency Error Pattern
            if line.contains("npm ERR!") || line.contains("Module not found") {
                return FailureFingerprint {
                    kind: FailureKind::Dependency,
                    code: None,
                    message: line.clone(),
                    metadata: std::collections::HashMap::new(),
                };
            }
        }

        FailureFingerprint {
            kind: FailureKind::Unknown,
            code: None,
            message: "Unknown build failure".to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

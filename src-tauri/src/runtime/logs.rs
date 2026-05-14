use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildFailure {
    pub file_path: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub error_code: String,
    pub message: String,
    pub raw_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
    Stdout(String),
    Stderr(String),
    BuildError(BuildFailure),
    ExitCode(i32),
    StepStarted(String),
    StepCompleted(String),
    RepairSuggested(String),
}

/// Helper to parse and stream runtime logs
pub struct LogStreamer;

impl LogStreamer {
    pub fn new() -> Self {
        Self
    }
}

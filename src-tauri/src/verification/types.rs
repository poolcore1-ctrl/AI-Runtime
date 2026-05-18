use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use tracing::warn;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationSeverity {
    Informational,
    Cosmetic,
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationFingerprint {
    CompilationError,
    RuntimeCrashLoop,
    ApiSchemaMismatch,
    HydrationFailure,
    VisualLayoutShift,
    StaleDependencyBoot,
    Unknown,
}

impl VerificationFingerprint {
    pub fn classify(logs: &str) -> Self {
        let l_lower = logs.to_lowercase();
        if l_lower.contains("compile") || l_lower.contains("error[e") || l_lower.contains("tsc") {
            VerificationFingerprint::CompilationError
        } else if l_lower.contains("crash") || l_lower.contains("exit") || l_lower.contains("address already in use") {
            VerificationFingerprint::RuntimeCrashLoop
        } else if l_lower.contains("hydration") || l_lower.contains("mismatch") {
            VerificationFingerprint::HydrationFailure
        } else if l_lower.contains("visual") || l_lower.contains("layout") || l_lower.contains("css") {
            VerificationFingerprint::VisualLayoutShift
        } else if l_lower.contains("status 404") || l_lower.contains("schema") || l_lower.contains("bad request") {
            VerificationFingerprint::ApiSchemaMismatch
        } else if l_lower.contains("dependency") || l_lower.contains("module not found") {
            VerificationFingerprint::StaleDependencyBoot
        } else {
            VerificationFingerprint::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactStorage {
    pub screenshots_dir: String,
    pub trace_logs_dir: String,
    pub video_traces: Vec<String>,
}

impl ArtifactStorage {
    pub fn new(base_path: &str) -> Self {
        Self {
            screenshots_dir: format!("{}/screenshots", base_path),
            trace_logs_dir: format!("{}/logs", base_path),
            video_traces: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub node_id: String,
    pub passed: bool,
    pub severity: VerificationSeverity,
    pub fingerprint: VerificationFingerprint,
    pub logs: Vec<String>,
    pub duration_ms: u64,
    pub screenshot_hash: Option<String>,
    pub error_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudgetState {
    pub max_time_ms: u64,
    pub max_screenshots: u32,
    pub elapsed_time_ms: u64,
    pub screenshots_taken: u32,
    pub limit_reached: bool,
}

pub struct VerificationBudgetManager {
    state: Arc<Mutex<TokenBudgetState>>,
}

impl VerificationBudgetManager {
    pub fn new(max_time_ms: u64, max_screenshots: u32) -> Self {
        Self {
            state: Arc::new(Mutex::new(TokenBudgetState {
                max_time_ms,
                max_screenshots,
                elapsed_time_ms: 0,
                screenshots_taken: 0,
                limit_reached: false,
            })),
        }
    }

    pub fn consume_resource(&self, duration: Duration, screenshots: u32) -> Result<()> {
        let mut lock = self.state.lock().unwrap();
        if lock.limit_reached {
            return Err(anyhow!("Verification resource budget exceeded. Verification locked."));
        }

        lock.elapsed_time_ms += duration.as_millis() as u64;
        lock.screenshots_taken += screenshots;

        if lock.elapsed_time_ms >= lock.max_time_ms || lock.screenshots_taken >= lock.max_screenshots {
            lock.limit_reached = true;
            warn!(
                elapsed = lock.elapsed_time_ms,
                max_time = lock.max_time_ms,
                screenshots = lock.screenshots_taken,
                max_screens = lock.max_screenshots,
                "Verification resource limit reached"
            );
            return Err(anyhow!("Verification resource limit reached: time = {}ms, screenshots = {}", lock.elapsed_time_ms, lock.screenshots_taken));
        }

        Ok(())
    }

    pub fn get_state(&self) -> TokenBudgetState {
        self.state.lock().unwrap().clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityTraceReport {
    pub trace_id: String,
    pub timestamp: u64,
    pub workspace_path: String,
    pub results: Vec<VerificationResult>,
    pub consensus_passed: bool,
    pub artifacts: ArtifactStorage,
}

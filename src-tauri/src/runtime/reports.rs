use serde::{Serialize, Deserialize};
use crate::runtime::errors::FailureFingerprint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMetrics {
    pub conflicting_edits: usize,
    pub duplicate_reasoning: usize,
    pub orchestration_latency: u64,
    pub repair_cascades: usize,
    pub strategy_reuse_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairTraceReport {
    pub session_id: String,
    pub initial_failure: FailureFingerprint,
    pub attempts: Vec<RepairAttempt>,
    pub final_outcome: RepairOutcome,
    pub total_duration_ms: u64,
    pub coordination_metrics: Option<CoordinationMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairAttempt {
    pub attempt_number: usize,
    pub retrieved_context_ids: Vec<String>,
    pub proposed_patch: String,
    pub environment_mutations: Vec<EnvironmentMutation>,
    pub strategy_reuse_source: Option<String>,
    pub adaptation_delta: Option<String>,
    pub reuse_confidence: Option<f32>,
    pub verification_passed: bool,
    pub new_failure: Option<FailureFingerprint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentMutation {
    pub package_added: Option<String>,
    pub package_removed: Option<String>,
    pub version_changed: Option<String>,
    pub lockfile_modified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepairOutcome {
    Success,
    MaxAttemptsExceeded,
    RollbackTriggered,
    Diverged,
}

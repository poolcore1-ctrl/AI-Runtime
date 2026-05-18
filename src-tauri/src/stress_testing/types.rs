use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntropyClass {
    Stable,
    Moderate,
    High,
    Extreme,
}

impl EntropyClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Moderate => "Moderate",
            Self::High => "High",
            Self::Extreme => "Extreme",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Moderate" => Self::Moderate,
            "High" => Self::High,
            "Extreme" => Self::Extreme,
            _ => Self::Stable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyScore {
    pub dependency_instability: f32,
    pub runtime_flakiness: f32,
    pub architecture_fragmentation: f32,
    pub verification_noise: f32,
    pub overall_entropy: f32,
    pub class: EntropyClass,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepositoryFailureClass {
    DependencyCollapse,
    RuntimeInstability,
    VerificationNoise,
    StructuralMismatch,
    SemanticRegression,
    ProviderDrift,
    StrategyPoisoning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayFingerprint {
    pub strategy_chain_hash: String,
    pub provider_chain_hash: String,
    pub verification_hash: String,
    pub workspace_snapshot_hash: String,
    pub reasoning_trace_hash: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CognitiveDrift {
    None,
    MinorSemanticVariance,
    StrategyDeviation,
    VerificationPathDeviation,
    BehavioralMismatch,
    CriticalDivergence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMutation {
    pub file_path: String,
    pub mutation_type: String, // "create", "modify", "delete"
    pub diff_hash: String,
    pub timestamp: u64,
    pub originating_agent: String,
}

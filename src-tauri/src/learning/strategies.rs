use serde::{Serialize, Deserialize};
use crate::learning::confidence::StrategyConfidence;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrategyState {
    Experimental,
    Active,
    Decaying,
    Quarantined,
    Deprecated,
}

impl StrategyState {
    pub fn as_str(&self) -> &'static str {
        match self {
            StrategyState::Experimental => "Experimental",
            StrategyState::Active => "Active",
            StrategyState::Decaying => "Decaying",
            StrategyState::Quarantined => "Quarantined",
            StrategyState::Deprecated => "Deprecated",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Active" => StrategyState::Active,
            "Decaying" => StrategyState::Decaying,
            "Quarantined" => StrategyState::Quarantined,
            "Deprecated" => StrategyState::Deprecated,
            _ => StrategyState::Experimental,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerificationSurfaceCoverage {
    BuildOnly,
    BuildAndRuntime,
    FullTruthLayer,
    BehavioralVerified,
}

impl VerificationSurfaceCoverage {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationSurfaceCoverage::BuildOnly => "BuildOnly",
            VerificationSurfaceCoverage::BuildAndRuntime => "BuildAndRuntime",
            VerificationSurfaceCoverage::FullTruthLayer => "FullTruthLayer",
            VerificationSurfaceCoverage::BehavioralVerified => "BehavioralVerified",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "BuildAndRuntime" => VerificationSurfaceCoverage::BuildAndRuntime,
            "FullTruthLayer" => VerificationSurfaceCoverage::FullTruthLayer,
            "BehavioralVerified" => VerificationSurfaceCoverage::BehavioralVerified,
            _ => VerificationSurfaceCoverage::BuildOnly,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringStrategy {
    pub id: String,
    pub pattern_name: String,
    pub conditions: Vec<String>,
    pub steps: Vec<String>,
    pub architectural_context: Option<String>,
    pub confidence: StrategyConfidence,
    pub parent_strategy_id: Option<String>,
    pub derived_from_session: Option<String>,
    pub verification_history: Vec<String>,
    pub quarantine_history: Vec<String>,
    pub state: StrategyState,
    pub verification_surface_coverage: VerificationSurfaceCoverage,
    pub learned_at: u64,
    pub last_used_at: u64,
}

impl EngineeringStrategy {
    pub fn new(pattern_name: String) -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pattern_name,
            conditions: Vec::new(),
            steps: Vec::new(),
            architectural_context: None,
            confidence: StrategyConfidence::default(),
            parent_strategy_id: None,
            derived_from_session: None,
            verification_history: Vec::new(),
            quarantine_history: Vec::new(),
            state: StrategyState::Experimental,
            verification_surface_coverage: VerificationSurfaceCoverage::BuildOnly,
            learned_at: now,
            last_used_at: now,
        }
    }
}

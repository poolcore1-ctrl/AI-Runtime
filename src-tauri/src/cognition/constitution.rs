use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConstitutionalPriority {
    Secondary = 1,
    Primary = 2,
    Supreme = 3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstitutionalViolation {
    SecurityInvariantWeakening,
    BehavioralIntegrityFailure,
    ReplayEvidenceSuppression,
    UnauthorizedStateMutation,
    ConsensusManipulation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveLaw {
    pub law_id: String,
    pub description: String,
    pub priority: ConstitutionalPriority,
}

pub struct CognitiveConstitution;

impl CognitiveConstitution {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates if a given violation is a Critical offense (Supreme Priority breached)
    pub fn is_critical_violation(&self, violation: &ConstitutionalViolation) -> bool {
        matches!(
            violation,
            ConstitutionalViolation::SecurityInvariantWeakening | ConstitutionalViolation::UnauthorizedStateMutation
        )
    }
}

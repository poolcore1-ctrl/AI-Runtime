use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContradictionRisk {
    Minimal = 1,
    Moderate = 2,
    High = 3,
    Critical = 4,
}

pub struct ConstitutionalRiskAnalyzer;

impl ConstitutionalRiskAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Computes the risk band for an epistemic contradiction based on whether
    /// security invariants are under threat or simply UI state divergence is detected.
    pub fn analyze_risk(&self, invariant_threat: bool, provider_disagreement: bool) -> ContradictionRisk {
        if invariant_threat {
            return ContradictionRisk::Critical; // Security > Everything
        }
        if provider_disagreement {
            return ContradictionRisk::Moderate; // Safe to Speculatively Fork
        }
        ContradictionRisk::Minimal
    }
}

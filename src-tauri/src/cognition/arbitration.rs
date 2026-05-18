use serde::{Serialize, Deserialize};
use crate::cognition::constitutional_risk::ContradictionRisk;
use crate::cognition::constitution::{CognitiveConstitution, ConstitutionalViolation};
use crate::cognition::audit::{CognitiveAuditEngine, AuditLineage};
use crate::cognition::uncertainty::UncertaintyGovernor;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArbitrationResolution {
    SpeculativeSandboxFork,
    ExpandedVerificationFork,
    EscalatedConsensus,
    ImmediateConstitutionalHalt,
}

pub struct TruthArbiter {
    constitution: CognitiveConstitution,
    audit_engine: CognitiveAuditEngine,
    _uncertainty_governor: UncertaintyGovernor,
}

impl TruthArbiter {
    pub fn new() -> Self {
        Self {
            constitution: CognitiveConstitution::new(),
            audit_engine: CognitiveAuditEngine::new(),
            _uncertainty_governor: UncertaintyGovernor::new(),
        }
    }

    /// Determines the legal resolution for a cognitive contradiction based on Risk Classification
    pub fn arbitrate(&self, risk: ContradictionRisk, violations: Vec<ConstitutionalViolation>) -> (ArbitrationResolution, Option<AuditLineage>) {
        
        // 1. Check for immediate critical legal violations
        let mut has_critical = false;
        for violation in &violations {
            if self.constitution.is_critical_violation(violation) {
                has_critical = true;
                break;
            }
        }

        if has_critical || risk == ContradictionRisk::Critical {
            let audit = self.audit_engine.log_arbitration("Critical Violation Detected - Mandatory Halt", violations);
            return (ArbitrationResolution::ImmediateConstitutionalHalt, Some(audit));
        }

        // 2. Risk-based hybrid arbitration
        let resolution = match risk {
            ContradictionRisk::Minimal => ArbitrationResolution::SpeculativeSandboxFork,
            ContradictionRisk::Moderate => ArbitrationResolution::ExpandedVerificationFork,
            ContradictionRisk::High => ArbitrationResolution::EscalatedConsensus,
            ContradictionRisk::Critical => ArbitrationResolution::ImmediateConstitutionalHalt, // Handled above, but exhaustive match
        };

        (resolution, None)
    }

    /// Evaluates if provider authority scores should override constitutional legality (Hint: Never)
    pub fn is_legality_supreme(&self, _provider_authority: f64) -> bool {
        // Constitutional Legality > Consensus Confidence > Historical Trust > Provider Authority
        true
    }
}

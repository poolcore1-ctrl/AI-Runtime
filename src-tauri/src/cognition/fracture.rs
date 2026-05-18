use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum FractureSeverity {
    Minor,
    Moderate,
    Severe,
    Existential,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdentityQuarantineMode {
    ObservationOnly,
    EvolutionFrozen,
    ReplayOnly,
    ConstitutionalRecovery,
    FullLockdown,
}

pub struct IdentityQuarantineCoordinator;

impl IdentityQuarantineCoordinator {
    pub fn new() -> Self {
        Self
    }

    /// Determines the appropriate system quarantine mode based on identity fracture severity.
    pub fn evaluate_quarantine_mode(
        &self,
        severity: FractureSeverity,
    ) -> IdentityQuarantineMode {
        match severity {
            FractureSeverity::Minor => IdentityQuarantineMode::ObservationOnly,
            FractureSeverity::Moderate => IdentityQuarantineMode::EvolutionFrozen,
            FractureSeverity::Severe => IdentityQuarantineMode::ReplayOnly,
            FractureSeverity::Existential => IdentityQuarantineMode::FullLockdown,
        }
    }

    /// Evaluates if new motif or trait evolution is permitted under the active quarantine mode.
    pub fn is_evolution_permitted(&self, mode: IdentityQuarantineMode) -> bool {
        match mode {
            IdentityQuarantineMode::ObservationOnly => true,
            _ => false, // Motif mutations are frozen to prevent further corruption
        }
    }

    /// Evaluates if live code repairs and graph execution are permitted under the active quarantine mode.
    pub fn is_live_execution_permitted(&self, mode: IdentityQuarantineMode) -> bool {
        match mode {
            IdentityQuarantineMode::ObservationOnly | IdentityQuarantineMode::EvolutionFrozen => true,
            _ => false, // ReplayOnly & FullLockdown restrict live execution entirely (Pure Replay Verification mode)
        }
    }
}

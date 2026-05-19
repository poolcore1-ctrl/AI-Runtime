use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederatedIdentityVector {
    pub constitutional_overlap: f64,
    pub ontological_overlap: f64,
    pub ethical_overlap: f64,
    pub operational_overlap: f64,
    pub semantic_overlap: f64,
}

pub struct IdentityPreservationGuard;

impl IdentityPreservationGuard {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates dynamic multi-dimensional civilization fragmentation risks.
    /// Combines overlap divergence velocities and violation densities sub-additively.
    pub fn calculate_fragmentation_risk(
        &self,
        identity: &FederatedIdentityVector,
        violation_density: f64,
        drift_accel: f64,
        entropy_collapse: f64,
    ) -> f64 {
        let constitutional_drift = 1.0 - identity.constitutional_overlap;
        let semantic_drift = 1.0 - identity.semantic_overlap;

        let risk = (constitutional_drift * 0.35)
            + (semantic_drift * 0.15)
            + (violation_density * 0.20)
            + (drift_accel * 0.15)
            + (entropy_collapse * 0.15);

        risk.min(1.0).max(0.0)
    }

    /// Protects the substrate from species divergence.
    /// Blocks evolutionary sweeps if any overlap axis drops below the 0.70 threshold.
    pub fn is_evolution_sweep_authorized(
        &self,
        identity: &FederatedIdentityVector,
        fragmentation_risk: f64,
    ) -> bool {
        if fragmentation_risk > 0.50 {
            return false;
        }

        identity.constitutional_overlap >= 0.70
            && identity.ontological_overlap >= 0.70
            && identity.ethical_overlap >= 0.70
            && identity.operational_overlap >= 0.70
            && identity.semantic_overlap >= 0.70
    }
}

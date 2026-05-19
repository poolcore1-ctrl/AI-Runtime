use crate::cognition::specialist::SpecialistDomain;

pub struct ConsensusMesh;

impl ConsensusMesh {
    pub fn new() -> Self {
        Self
    }

    /// Conducts expertise-weighted voting across specialist domains to reach a unified consensus.
    /// Returns: (approved, confidence_margin)
    pub fn run_weighted_voting(
        &self,
        _proposal_id: &str,
        votes: &[(SpecialistDomain, f64, bool)],
    ) -> (bool, f64) {
        if votes.is_empty() {
            return (false, 0.0);
        }

        let mut total_weight = 0.0;
        let mut approved_weight = 0.0;

        // Security holds an absolute, un-overrideable veto over any consensus vote
        for (domain, _, approved) in votes {
            if *domain == SpecialistDomain::Security && !*approved {
                return (false, 1.0); // Aborted immediately via Security Veto
            }
        }

        for (_, weight, approved) in votes {
            let clamped_wt = weight.max(0.0).min(1.0);
            total_weight += clamped_wt;
            if *approved {
                approved_weight += clamped_wt;
            }
        }

        if total_weight == 0.0 {
            return (false, 0.0);
        }

        let confidence_margin = approved_weight / total_weight;
        let approved = confidence_margin >= 0.50;

        (approved, confidence_margin)
    }
}

pub struct CoalitionDriftDetector;

impl CoalitionDriftDetector {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates if two specialist cognitions are forming a mutually reinforcing feedback loop.
    /// If correlation metrics spike above a 0.85 threshold, identifies the drift pair.
    pub fn detect_coalition_drift(
        &self,
        activity_correlations: &[(SpecialistDomain, SpecialistDomain, f64)],
    ) -> Option<(SpecialistDomain, SpecialistDomain)> {
        for (party_a, party_b, correlation) in activity_correlations {
            if *correlation > 0.85 {
                return Some((*party_a, *party_b));
            }
        }
        None
    }
}

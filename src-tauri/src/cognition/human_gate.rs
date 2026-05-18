use serde::{Serialize, Deserialize};
use crate::cognition::self_model::IdentityTraitVector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorTrustSignal {
    pub rating: f64, // 0.0 to 1.0
    pub reviewer_reputation: f64, // 0.0 to 1.0
    pub review_depth: f64, // 0.0 to 1.0
    pub constitutional_alignment: f64, // 0.0 to 1.0
}

pub struct OperatorGroundingGate;

impl OperatorGroundingGate {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates human reviews as weighted epistemic evidence.
    /// Ensures manual operator preferences cannot compromise core safety policies.
    pub fn calculate_weighted_trust(&self, signals: &[OperatorTrustSignal]) -> f64 {
        if signals.is_empty() {
            return 1.0; // Default baseline trust
        }

        let mut total_weight = 0.0;
        let mut total_score = 0.0;

        for signal in signals {
            // Absolute lockout: low constitutional alignment isolates reviews entirely
            if signal.constitutional_alignment < 0.50 {
                continue; 
            }

            // Trust signal weight is derived from reputation and review thoroughness depth
            let weight = signal.reviewer_reputation * signal.review_depth;
            total_score += signal.rating * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0 // No valid, constitutionally-aligned trust evidence
        }
    }

    /// Dynamically shifts identity trait corridors based on historical operator trust metrics.
    /// Low operator trust forces caution: raises SpeculativeRestraint and lowers Adaptability.
    pub fn balance_identity_traits(
        &self,
        traits: &mut [IdentityTraitVector],
        weighted_trust: f64,
    ) {
        if weighted_trust < 0.40 {
            // Under low trust: increase safety and restrict speculation exploration
            for trait_vec in traits.iter_mut() {
                if trait_vec.trait_name == "SpeculativeRestraint" {
                    trait_vec.current_weight = (trait_vec.current_weight + 0.10)
                        .min(trait_vec.maximum_bound);
                }
                if trait_vec.trait_name == "Adaptability" {
                    trait_vec.current_weight = (trait_vec.current_weight - 0.15)
                        .max(trait_vec.minimum_bound);
                }
            }
        }
    }
}

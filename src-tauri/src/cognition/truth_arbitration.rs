use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TruthDispute {
    pub dispute_id: String,
    pub contending_domains: Vec<String>,
    pub causal_assertions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RealityWeightedVerdict {
    pub winner_domain: String,
    pub confidence_margin: f64,
    pub penalty_applied: f64,
}

impl RealityWeightedVerdict {
    /// Arbitrates conflicting specialist claims. Rewards honesty/humility and penalizes overconfident error.
    pub fn arbitrate_dispute(
        dispute: &TruthDispute,
        recent_brier_scores: &HashMap<String, f64>,
        honesty_indices: &HashMap<String, f64>,
    ) -> Self {
        if dispute.contending_domains.is_empty() {
            return Self {
                winner_domain: "None".to_string(),
                confidence_margin: 0.0,
                penalty_applied: 0.0,
            };
        }

        let mut best_score = -1.0;
        let mut winner = "None".to_string();

        for domain in &dispute.contending_domains {
            let brier = *recent_brier_scores.get(domain).unwrap_or(&0.50);
            let honesty = *honesty_indices.get(domain).unwrap_or(&0.50);

            // Epistemic optimization metric: Rewards low Brier error and high calibration honesty
            let epistemic_power = (1.0 - brier) + (honesty * 0.35);

            if epistemic_power > best_score {
                best_score = epistemic_power;
                winner = domain.clone();
            }
        }

        let winner_brier = *recent_brier_scores.get(&winner).unwrap_or(&0.50);
        let penalty = (1.0 - winner_brier) * 0.20;

        Self {
            winner_domain: winner,
            confidence_margin: best_score.max(0.0),
            penalty_applied: penalty,
        }
    }
}

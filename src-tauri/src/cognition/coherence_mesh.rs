use serde::{Serialize, Deserialize};
use crate::cognition::specialist::SpecialistDomain;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPerspective {
    pub specialist: SpecialistDomain,
    pub local_interpretation: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedSemanticFrame {
    pub concept_id: String,
    pub canonical_meaning: String,
    pub perspectives: Vec<SemanticPerspective>,
    pub divergence_score: f64,
}

pub struct SemanticDriftDetector {
    pub decay_rate: f64,
}

impl SemanticDriftDetector {
    pub fn new(decay_rate: f64) -> Self {
        Self { decay_rate }
    }

    /// Evaluates dynamic temporal drift degradation of concept frames.
    /// Prevents old treaty concepts from becoming frozen and semantically stale.
    pub fn calculate_temporal_drift(&self, initial_divergence: f64, delta_time: f64) -> f64 {
        let drift = initial_divergence * (1.0 + (self.decay_rate * delta_time));
        drift.min(1.0).max(0.0)
    }

    /// Measures divergence score directly from perspectives variance.
    pub fn evaluate_concept_divergence(&self, frame: &SharedSemanticFrame) -> f64 {
        if frame.perspectives.is_empty() {
            return 0.0;
        }

        let mut sum_diff = 0.0;
        let count = frame.perspectives.len() as f64;

        for p in &frame.perspectives {
            // Divergence scales if local weights deviate significantly from average frame index
            sum_diff += (1.0 - p.weight).abs();
        }

        (sum_diff / count).min(1.0).max(0.0)
    }
}

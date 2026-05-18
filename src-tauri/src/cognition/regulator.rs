use crate::cognition::homeostasis::CognitivePhysiologyEngine;

pub struct HomeostaticRegulator {
    physiology_engine: CognitivePhysiologyEngine,
}

impl HomeostaticRegulator {
    pub fn new() -> Self {
        Self {
            physiology_engine: CognitivePhysiologyEngine::new(),
        }
    }

    /// Evaluates if the current cognitive stability is high enough to permit maximum speculative depth
    pub fn allowed_speculative_depth(
        &self,
        provider_diversity: f64,
        replay_consistency: f64,
        constitutional_compliance: f64,
        entropy_pressure: f64,
        pathology_density: f64,
        propagated_uncertainty: f64,
    ) -> usize {
        let stability_score = self.physiology_engine.calculate_stability(
            provider_diversity,
            replay_consistency,
            constitutional_compliance,
            entropy_pressure,
            pathology_density,
            propagated_uncertainty,
        );

        if stability_score > 2.0 {
            return 3; // Maximum allowed speculative depth
        } else if stability_score > 0.8 {
            return 1; // Restricted depth
        }

        0 // Speculation halted
    }
}

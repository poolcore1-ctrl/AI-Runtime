use crate::cognition::physiology::{CognitivePhysiologyEngine, CognitiveVitals, CognitiveEnvironment};

pub struct HomeostaticRegulator {
    pub physiology_engine: CognitivePhysiologyEngine,
}

impl HomeostaticRegulator {
    pub fn new() -> Self {
        Self {
            physiology_engine: CognitivePhysiologyEngine::new(),
        }
    }

    /// Evaluates if the current cognitive stability is high enough to permit maximum speculative depth.
    /// Maps parameters to Vitals and Environment, evaluates the stability equation, and outputs allowed depth (0 to 3).
    pub fn allowed_speculative_depth(
        &self,
        provider_diversity: f64,
        replay_consistency: f64,
        constitutional_compliance: f64,
        entropy_pressure: f64,
        pathology_density: f64,
        propagated_uncertainty: f64,
    ) -> usize {
        // Map loose inputs into unified Vitals and Environment
        let vitals = CognitiveVitals {
            entropy_pressure,
            contradiction_density: 1.0 - constitutional_compliance.min(1.0),
            replay_instability: 1.0 - replay_consistency.min(1.0),
            provider_fatigue: pathology_density * 0.5,
            verifier_saturation: propagated_uncertainty * 0.6,
            graph_complexity_load: 0.3,
            memory_fragmentation: 0.2,
        };

        let env = CognitiveEnvironment {
            global_entropy: entropy_pressure * 0.8,
            provider_market_instability: 0.1,
            memory_reliability: provider_diversity.max(0.5).min(1.0),
            replay_confidence: replay_consistency.max(0.5).min(1.0),
        };

        // Determine pathologies count based on pathology density
        let pathology_count = if pathology_density > 0.60 {
            2
        } else if pathology_density > 0.30 {
            1
        } else {
            0
        };

        let stability_score = self.physiology_engine.calculate_stability(&vitals, &env, pathology_count);

        // Map stability score to speculative depth [0..3]
        if stability_score > 0.80 {
            3 // Fully stable: Maximum speculation permitted
        } else if stability_score > 0.65 {
            2 // Elevated stress: Moderate speculation
        } else if stability_score > 0.45 {
            1 // Degraded: Safety-restrained speculation
        } else {
            0 // Critical or Quarantined: Speculation halts entirely
        }
    }
}

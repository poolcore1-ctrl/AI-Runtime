use serde::{Serialize, Deserialize};
use crate::cognition::physiology::{CognitiveVitals, CognitiveEnvironment};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionCausalChain {
    pub triggering_event: String,
    pub constitutional_rules_applied: Vec<String>,
    pub contradictions_detected: Vec<String>,
    pub arbitration_steps: Vec<String>,
    pub final_decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaStabilityIndex {
    pub physiological_stability: f64,
    pub epistemic_stability: f64,
    pub identity_stability: f64,
    pub evolutionary_stability: f64,
    pub composite_score: f64,
}

pub struct IntrospectionEngine;

impl IntrospectionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Formulates a complete, trace-grade explainable reasoning IntrospectionCausalChain.
    pub fn build_introspection_chain(
        &self,
        event: &str,
        rules: &[String],
        contradictions: &[String],
        arbitration: &[String],
        decision: &str,
    ) -> IntrospectionCausalChain {
        IntrospectionCausalChain {
            triggering_event: event.to_string(),
            constitutional_rules_applied: rules.to_vec(),
            contradictions_detected: contradictions.to_vec(),
            arbitration_steps: arbitration.to_vec(),
            final_decision: decision.to_string(),
        }
    }

    /// Evaluates the complete, composite MetaStabilityIndex of the ASOS computational organism.
    pub fn calculate_metastability(
        &self,
        vitals: &CognitiveVitals,
        env: &CognitiveEnvironment,
        identity_stability: f64,
    ) -> MetaStabilityIndex {
        // 1. Physiological stability derived from vitals
        let phys_stab = 1.0 - (vitals.entropy_pressure * 0.3 + vitals.provider_fatigue * 0.4 + vitals.memory_fragmentation * 0.3).min(1.0);

        // 2. Epistemic stability derived from environmental accuracy indicators
        let epistemic_stab = (env.memory_reliability * env.replay_confidence).min(1.0);

        // 3. Evolutionary stability representing motif fitness
        let evo_stab = 1.0 - (vitals.replay_instability * 0.6 + vitals.graph_complexity_load * 0.4).min(1.0);

        // Composite overall score
        let composite = (phys_stab * 0.30) + (epistemic_stab * 0.25) + (identity_stability * 0.25) + (evo_stab * 0.20);

        MetaStabilityIndex {
            physiological_stability: phys_stab,
            epistemic_stability: epistemic_stab,
            identity_stability,
            evolutionary_stability: evo_stab,
            composite_score: composite.max(0.0).min(1.0),
        }
    }
}

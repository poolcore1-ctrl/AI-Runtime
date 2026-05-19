use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CausalEffectClass {
    Stabilizing,
    Neutral,
    Destabilizing,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CausalArchetype {
    RetryStorm,
    Starvation,
    Deadlock,
    MemoryLeak,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CausalTransition {
    pub source_state: String,
    pub triggering_action: String,
    pub target_state: String,
    pub causal_effect_class: CausalEffectClass,
    pub propagation_probability: f64,
    pub temporal_latency_ms: Option<u64>,
    pub affected_invariants: Vec<String>,
    pub downstream_risk_score: f64,
    pub reversibility: f64,
    pub confidence: f64,
    pub causal_uncertainty: f64, // Compounded uncertainty (1.0 - confidence) across multi-hop hops
}

pub struct CausalGraph {
    pub transitions: Vec<CausalTransition>,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self { transitions: Vec::new() }
    }

    /// Evaluates compounded uncertainty along a causal chain pathway.
    /// Uses multiplicative uncertainty retention: U_final = 1.0 - (Product_i (1.0 - U_i))
    pub fn propagate_uncertainty(&self, pathway: &[CausalTransition]) -> f64 {
        if pathway.is_empty() {
            return 0.0;
        }

        let mut compounded_certainty = 1.0;

        for transition in pathway {
            let certainty = 1.0 - transition.causal_uncertainty.max(0.0).min(1.0);
            compounded_certainty *= certainty;
        }

        (1.0 - compounded_certainty).max(0.0).min(1.0)
    }

    /// Compresses a sequence of causal transitions into an archetype pattern
    pub fn compress_to_archetype(&self, pathway: &[CausalTransition]) -> Option<CausalArchetype> {
        let has_mutex = pathway.iter().any(|t| t.source_state.contains("Mutex") || t.target_state.contains("Mutex") || t.triggering_action.contains("lock"));
        let has_retry = pathway.iter().any(|t| t.triggering_action.contains("retry") || t.target_state.contains("storm"));
        let has_leak = pathway.iter().any(|t| t.target_state.contains("allocation") || t.target_state.contains("leak"));
        let has_queue = pathway.iter().any(|t| t.source_state.contains("queue") || t.target_state.contains("starvation"));

        if has_mutex && has_queue {
            Some(CausalArchetype::Deadlock)
        } else if has_retry {
            Some(CausalArchetype::RetryStorm)
        } else if has_leak {
            Some(CausalArchetype::MemoryLeak)
        } else if has_queue {
            Some(CausalArchetype::Starvation)
        } else {
            None
        }
    }
}

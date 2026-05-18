use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveBelief {
    pub belief_id: String,
    pub statement: String,
    pub confidence: f64,
    pub supporting_evidence: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub source_systems: Vec<String>,
    pub temporal_stability: f64,
}

pub struct BeliefDecayPolicy {
    pub temporal_half_life_days: u64,
    pub contradiction_penalty: f64,
    pub replay_confirmation_boost: f64,
}

pub struct BeliefGraphManager {
    decay_policy: BeliefDecayPolicy,
}

impl BeliefGraphManager {
    pub fn new() -> Self {
        Self {
            decay_policy: BeliefDecayPolicy {
                temporal_half_life_days: 14,
                contradiction_penalty: 0.25,
                replay_confirmation_boost: 0.15,
            },
        }
    }

    /// Evaluates if a belief's confidence has dropped too low due to temporal decay or contradiction penalties
    pub fn degrade_confidence(&self, mut belief: CognitiveBelief, has_contradiction: bool) -> CognitiveBelief {
        if has_contradiction {
            belief.confidence -= self.decay_policy.contradiction_penalty;
        }
        
        // Ensure bounds
        if belief.confidence < 0.0 {
            belief.confidence = 0.0;
        }

        belief
    }
}

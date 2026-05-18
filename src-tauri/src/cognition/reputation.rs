use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrustEnvelope {
    pub trust_score: f64,
    pub replay_verified: bool,
    pub anomaly_score: f64,
    pub consensus_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveReputation {
    pub entity_id: String,
    pub trust_score: f64,
    pub semantic_accuracy: f64,
    pub replay_stability: f64,
    pub drift_rate: f64,
}

pub struct ReputationEngine;

impl ReputationEngine {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates whether a memory record is trusted enough to influence predictive routing.
    /// Defends against poisoned memories or adversarial repo executions.
    pub fn is_memory_trusted(&self, envelope: &MemoryTrustEnvelope) -> bool {
        // Must have a baseline trust, not be highly anomalous, and ideally be replay verified
        if envelope.anomaly_score > 0.8 {
            return false; // Highly anomalous
        }
        if envelope.trust_score < 0.5 && !envelope.replay_verified {
            return false; // Untrusted and unverified
        }
        true
    }
}

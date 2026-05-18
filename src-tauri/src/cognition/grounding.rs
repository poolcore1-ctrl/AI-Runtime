use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundingVector {
    pub syntactic_confidence: f64,
    pub behavioral_confidence: f64,
    pub performance_confidence: f64,
    pub replay_consistency: f64,
    pub operator_confidence: f64,
    pub environmental_stability: f64,
    pub adversarial_resilience: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RealityEvidenceIntegrity {
    pub evidence_signature: String,
    pub sandbox_isolation_hash: String,
    pub compiler_binary_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RealityTruthAnchor {
    pub anchor_id: String,
    pub source_node: String,
    pub observed_at: i64,
    pub validity_half_life: f64, // Epoch duration in seconds after which confidence decays by half (50%)
    pub grounding_vector: GroundingVector,
    pub integrity_seal: RealityEvidenceIntegrity,
}

impl GroundingVector {
    /// Computes the overall composite grounding score of the vector
    pub fn composite_score(&self) -> f64 {
        let weighted = (self.syntactic_confidence * 0.20)
            + (self.behavioral_confidence * 0.25)
            + (self.performance_confidence * 0.15)
            + (self.replay_consistency * 0.15)
            + (self.operator_confidence * 0.10)
            + (self.environmental_stability * 0.05)
            + (self.adversarial_resilience * 0.10);

        weighted.max(0.0).min(1.0)
    }
}

impl RealityEvidenceIntegrity {
    /// Asserts evidence authenticity against cryptographic expected checksum signatures
    pub fn verify_integrity(&self, expected_signature: &str) -> bool {
        if self.evidence_signature != expected_signature {
            return false;
        }

        // Must have non-empty isolation hashes
        if self.sandbox_isolation_hash.is_empty() || self.compiler_binary_fingerprint.is_empty() {
            return false;
        }

        true
    }
}

impl RealityTruthAnchor {
    /// Evaluates probabilistic temporal decay over time.
    /// Confidence degrades dynamically according to half-life bounds once time elapsed increases.
    pub fn calculate_decayed_vector(&self, current_time: i64) -> GroundingVector {
        let elapsed = (current_time - self.observed_at).max(0) as f64;
        
        // Compute decay coefficient: 2^(-elapsed / half_life)
        let decay_coeff = if self.validity_half_life > 0.0 {
            (-elapsed / self.validity_half_life).exp2()
        } else {
            1.0
        };

        // Syntactic and adversarial metrics are absolute logic check outputs and don't decay.
        // Dynamic behavioral, operator, and performance indicators decay over time.
        GroundingVector {
            syntactic_confidence: self.grounding_vector.syntactic_confidence,
            behavioral_confidence: self.grounding_vector.behavioral_confidence * decay_coeff,
            performance_confidence: self.grounding_vector.performance_confidence * decay_coeff,
            replay_consistency: self.grounding_vector.replay_consistency * decay_coeff,
            operator_confidence: self.grounding_vector.operator_confidence * decay_coeff,
            environmental_stability: self.grounding_vector.environmental_stability,
            adversarial_resilience: self.grounding_vector.adversarial_resilience,
        }
    }
}

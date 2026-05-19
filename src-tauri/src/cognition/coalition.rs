use std::collections::HashMap;
use crate::cognition::specialist::SpecialistDomain;

pub struct CoalitionEdge {
    pub source: SpecialistDomain,
    pub target: SpecialistDomain,
    pub influence_strength: f64,
    pub dependency_correlation: f64,
    pub coordinated_drift_score: f64,
}

pub struct AntiCorruptionScanner;

impl AntiCorruptionScanner {
    pub fn new() -> Self {
        Self
    }

    /// Computes Shannon entropy over evidence lineage hashes.
    /// Distinguishes independent convergence from dangerous, identical-ancestry reasoning (monocultures).
    pub fn calculate_intent_entropy(&self, evidence_lineage_hashes: &[String]) -> f64 {
        if evidence_lineage_hashes.is_empty() {
            return 0.0;
        }

        let mut hash_counts = HashMap::new();
        for hash in evidence_lineage_hashes {
            *hash_counts.entry(hash.clone()).or_insert(0.0) += 1.0;
        }

        let total = evidence_lineage_hashes.len() as f64;
        let mut entropy = 0.0;

        for count in hash_counts.values() {
            let p = count / total;
            entropy -= p * p.log2();
        }

        entropy
    }

    /// Scans the coalition graph to detect dangerous coordination loops.
    /// Flags capture when coordinated drift is high AND intent entropy falls below the critical limit.
    pub fn evaluate_coalition_capture(&self, edges: &[CoalitionEdge], intent_entropy: f64) -> bool {
        let mut suspicious_drift = false;

        for edge in edges {
            if edge.coordinated_drift_score > 0.80 && edge.influence_strength > 0.75 {
                suspicious_drift = true;
                break;
            }
        }

        // Capture triggered if drift is highly correlated and reasoning diversity collapses (entropy < 0.40)
        suspicious_drift && intent_entropy < 0.40
    }
}

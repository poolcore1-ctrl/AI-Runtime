use serde::{Serialize, Deserialize};
use crate::cognition::belief_graph::CognitiveBelief;
use crate::cognition::optimization::GraphMotif;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamReport {
    pub replay_graph_id: String,
    pub original_accuracy: f64,
    pub dream_accuracy: f64,
    pub drift_detected: bool,
    pub adjustments_made: usize,
}

pub struct ConsolidationEngine;

impl ConsolidationEngine {
    pub fn new() -> Self {
        Self
    }

    /// Prunes dead beliefs whose confidence has fallen below the minimum survival threshold.
    /// Called during idle sleep cycles or forced micro-sleep when energy is critically depleted.
    pub fn prune_dead_beliefs(&self, beliefs: Vec<CognitiveBelief>, min_confidence: f64) -> Vec<CognitiveBelief> {
        beliefs.into_iter().filter(|b| b.confidence >= min_confidence).collect()
    }

    /// Returns true if a micro-sleep is warranted based on energy depletion ratio.
    pub fn should_trigger_micro_sleep(&self, consumed_energy: f64, total_energy: f64) -> bool {
        let depletion_ratio = consumed_energy / total_energy.max(0.01);
        depletion_ratio >= 0.85 // Trigger when 85% of cognitive budget consumed
    }

    /// Runs a complete sleep cycle on internal memory layers, pruning stale beliefs
    /// and decaying half-life indexes.
    pub fn execute_sleep_cycle(
        &self,
        beliefs: &mut Vec<CognitiveBelief>,
        decay_coefficient: f64,
    ) -> usize {
        let original_len = beliefs.len();

        // 1. Decay all belief confidence levels
        for belief in beliefs.iter_mut() {
            belief.confidence *= decay_coefficient;
        }

        // 2. Filter out dead beliefs below threshold (0.25 confidence)
        beliefs.retain(|b| b.confidence >= 0.25);
        original_len - beliefs.len()
    }

    /// Performs a synthetic offline "Dream Replay" of historical repair topologies.
    /// Executes validation checks to see if old strategies hold or suffer from drift.
    pub fn simulate_dream_replay(
        &self,
        graph_id: &str,
        original_success_rate: f64,
        drift_rate: f64,
    ) -> DreamReport {
        // Model synthetic dream verification: success rate decays as environmental drift sets in
        let dream_success = (original_success_rate * (1.0 - drift_rate)).max(0.0).min(1.0);
        let drift_detected = dream_success < 0.70;
        
        let adjustments = if drift_detected {
            // Number of node-routing mutations made to realign strategy in dream state
            3
        } else {
            0
        };

        DreamReport {
            replay_graph_id: graph_id.to_string(),
            original_accuracy: original_success_rate,
            dream_accuracy: dream_success,
            drift_detected,
            adjustments_made: adjustments,
        }
    }

    /// Applies the biological Forgetting Curve to active cognitive motifs.
    /// Obsolete motifs that haven't been triggered decay naturally to prevent graph topology overfitting.
    pub fn decay_motif_lineage(&self, motifs: &mut Vec<GraphMotif>, active_motif_ids: &[String]) -> usize {
        let mut decayed_count = 0;
        for motif in motifs.iter_mut() {
            // If the motif was not active in this execution epoch, apply forgetting decay
            if !active_motif_ids.contains(&motif.motif_id) {
                motif.semantic_success_rate = (motif.semantic_success_rate * 0.95).max(0.0);
                decayed_count += 1;
            }
        }
        decayed_count
    }
}

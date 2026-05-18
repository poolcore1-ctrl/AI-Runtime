use crate::cognition::belief_graph::CognitiveBelief;

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

    /// Returns true if a micro-sleep is warranted based on energy depletion ratio
    pub fn should_trigger_micro_sleep(&self, consumed_energy: f64, total_energy: f64) -> bool {
        let depletion_ratio = consumed_energy / total_energy.max(0.01);
        depletion_ratio >= 0.85 // Trigger when 85% of cognitive budget consumed
    }
}

use crate::cognition::belief_graph::CognitiveBelief;
use crate::cognition::physiology::{CognitiveVitals, CognitiveStabilityState};

pub struct CognitiveRecoveryCoordinator;

impl CognitiveRecoveryCoordinator {
    pub fn new() -> Self {
        Self
    }

    /// Purges highly contradicted or poisoned beliefs from long-term memory.
    /// Returns the number of beliefs removed.
    pub fn purge_poisoned_beliefs(&self, beliefs: &mut Vec<CognitiveBelief>) -> usize {
        let original_len = beliefs.len();

        // Purge any belief that has severe contradiction metrics (e.g. confidence < 0.35)
        beliefs.retain(|b| b.confidence >= 0.35);

        original_len - beliefs.len()
    }

    /// Compacts the memory index layout, simulating database vacuum and index rebuilding.
    /// Resets the memory fragmentation vital factor to baseline.
    pub fn rebuild_memory_index(&self, vitals: &mut CognitiveVitals) -> f64 {
        let original_frag = vitals.memory_fragmentation;
        
        // Rebuilding reduces fragmentation down to a pristine 0.05
        vitals.memory_fragmentation = 0.05;

        original_frag - vitals.memory_fragmentation
    }

    /// Resets physiological stress markers and restores the master stability state to Recovery.
    pub fn restore_constitutional_state(&self, current_state: &mut CognitiveStabilityState) -> bool {
        if *current_state == CognitiveStabilityState::Critical || *current_state == CognitiveStabilityState::Degraded {
            *current_state = CognitiveStabilityState::Recovery;
            true
        } else {
            false
        }
    }
}

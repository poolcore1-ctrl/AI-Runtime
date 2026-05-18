use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use crate::cognition::physiology::{CognitiveVitals, CognitiveStabilityState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitivePhysiology {
    pub entropy_pressure: f64,
    pub memory_saturation: f64,
    pub contradiction_density: f64,
    pub verification_load: f64,
    pub speculative_instability: f64,
    pub provider_fragmentation: f64,
    pub graph_mutation_rate: f64,
    pub constitutional_stress: f64,
}

pub struct HomeostaticRegulationController {
    pub quarantined_motifs: HashSet<String>,
    pub cooled_providers: HashSet<String>,
}

impl HomeostaticRegulationController {
    pub fn new() -> Self {
        Self {
            quarantined_motifs: HashSet::new(),
            cooled_providers: HashSet::new(),
        }
    }

    /// Action 1: Adaptive Consensus Thresholds
    /// If contradiction density spikes, we increase consensus thresholds dynamically
    /// to guarantee safety margins.
    pub fn calculate_consensus_threshold(
        &self,
        base_threshold: f64,
        contradiction_density: f64,
        stability_state: CognitiveStabilityState,
    ) -> f64 {
        let mut threshold = base_threshold;

        // If contradiction density is elevated, add dynamic safety margin
        if contradiction_density > 0.40 {
            let margin = (contradiction_density - 0.40) * 0.35;
            threshold += margin;
        }

        // Stability state modifiers
        threshold += match stability_state {
            CognitiveStabilityState::Stable => 0.0,
            CognitiveStabilityState::ElevatedStress => 0.05,
            CognitiveStabilityState::Degraded => 0.12,
            CognitiveStabilityState::Critical => 0.20,
            CognitiveStabilityState::Recovery => 0.08,
            CognitiveStabilityState::Quarantined => 0.25,
        };

        threshold.min(0.99) // Limit to 99% maximum verification threshold
    }

    /// Action 2: Speculative Branch Suppression
    /// Compresses speculative branch depth under high entropy pressure to preserve tokens and focus reasoning.
    pub fn adjust_speculative_budget(
        &self,
        base_budget: usize,
        entropy_pressure: f64,
        stability_state: CognitiveStabilityState,
    ) -> usize {
        if stability_state == CognitiveStabilityState::Critical {
            return 0; // Absolute shutdown of speculative executions
        }

        let mut allowed = base_budget as f64;

        // Apply progressive penalty based on entropy pressure
        if entropy_pressure > 0.50 {
            let penalty = (entropy_pressure - 0.50) * 1.5; // Steep suppression
            allowed -= allowed * penalty;
        }

        // Apply state limits
        match stability_state {
            CognitiveStabilityState::Stable => {}
            CognitiveStabilityState::ElevatedStress => { allowed = allowed.min(3.0); }
            CognitiveStabilityState::Degraded => { allowed = allowed.min(1.0); }
            CognitiveStabilityState::Recovery => { allowed = allowed.min(2.0); }
            _ => { allowed = 0.0; }
        }

        (allowed.round() as usize).max(0)
    }

    /// Action 3: Provider Cooling
    /// Checks and manages provider cooling periods. If provider fatigue is high,
    /// we temporarily label them cooled down (i.e. suspended or low priority).
    pub fn manage_provider_cooling(&mut self, provider: &str, fatigue_score: f64) -> bool {
        let key = provider.to_lowercase();
        if fatigue_score > 0.75 {
            self.cooled_providers.insert(key.clone());
            true // Suspended (needs cooling)
        } else if fatigue_score < 0.30 {
            self.cooled_providers.remove(&key);
            false // Active (fully cooled)
        } else {
            // Keep current status if in intermediate state
            self.cooled_providers.contains(&key)
        }
    }

    /// Action 4: Replay Escalation
    /// Spiking replay instability triggers mandatory forensic isolated replays
    /// to isolate compiler or target drift.
    pub fn should_escalate_to_forensic_replay(
        &self,
        vitals: &CognitiveVitals,
        stability_state: CognitiveStabilityState,
    ) -> bool {
        if vitals.replay_instability > 0.70 {
            return true;
        }

        match stability_state {
            CognitiveStabilityState::Degraded => vitals.replay_instability > 0.55,
            CognitiveStabilityState::Critical => true,
            _ => false,
        }
    }

    /// Action 5: Motif Quarantine
    /// Suspends evolved motif templates if they are detected in a pathology loop.
    pub fn manage_motif_quarantine(&mut self, motif_id: &str, failure_rate: f64) -> bool {
        if failure_rate > 0.40 {
            self.quarantined_motifs.insert(motif_id.to_string());
            true
        } else {
            self.quarantined_motifs.remove(motif_id);
            false
        }
    }
}

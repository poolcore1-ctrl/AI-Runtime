use serde::{Serialize, Deserialize};
use crate::cognition::physiology::{CognitiveVitals, PhysiologySnapshot};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CognitivePathology {
    RecursiveLooping,
    ConsensusDeadlock,
    VerificationStorm,
    MotifCollapse,
    ProviderDependency,
    BeliefFragmentation,
    ReplayFixation,       // Endlessly replaying historical traces instead of advancing
    EntropyThrashing,     // Graph topology constantly oscillating, preventing convergence
}

pub struct PathologyDetector;

impl PathologyDetector {
    pub fn new() -> Self {
        Self
    }

    /// Scans current vitals and recent longitudinal snapshot trend history
    /// to diagnose cognitive pathologies in ASOS.
    pub fn detect_pathologies(
        &self,
        vitals: &CognitiveVitals,
        history: &[PhysiologySnapshot],
    ) -> Vec<CognitivePathology> {
        let mut pathologies = Vec::new();

        // 1. Recursive Looping: Spiking contradiction and verifier activity
        if vitals.verifier_saturation > 0.70 && vitals.contradiction_density > 0.60 {
            pathologies.push(CognitivePathology::RecursiveLooping);
        }

        // 2. Consensus Deadlock: Spiking contradiction and replay instability
        if vitals.contradiction_density > 0.80 && vitals.replay_instability > 0.75 {
            pathologies.push(CognitivePathology::ConsensusDeadlock);
        }

        // 3. Verification Storm: Verifiers are completely saturated
        if vitals.verifier_saturation > 0.85 {
            pathologies.push(CognitivePathology::VerificationStorm);
        }

        // 4. Motif Collapse: High graph complexity load + extremely high entropy
        if vitals.graph_complexity_load > 0.80 && vitals.entropy_pressure > 0.80 {
            pathologies.push(CognitivePathology::MotifCollapse);
        }

        // 5. Provider Dependency: Extreme provider fatigue
        if vitals.provider_fatigue > 0.85 {
            pathologies.push(CognitivePathology::ProviderDependency);
        }

        // 6. Belief Fragmentation: Memory fragmentation is high
        if vitals.memory_fragmentation > 0.80 {
            pathologies.push(CognitivePathology::BeliefFragmentation);
        }

        // --- Longitudinal / Trend Heuristics ---
        if history.len() >= 3 {
            // 7. Replay Fixation: System is locked in repeated replays
            // Indicated by high, rising replay_instability over consecutive snapshots while in ElevatedStress/Degraded
            let mut rising_replay = true;
            let mut stressed = true;
            for i in 0..history.len() - 1 {
                if history[i].vitals.replay_instability >= history[i + 1].vitals.replay_instability {
                    rising_replay = false;
                }
                if history[i].vitals.replay_instability < 0.4 {
                    rising_replay = false;
                }
                if history[i].stability_state == crate::cognition::physiology::CognitiveStabilityState::Stable {
                    stressed = false;
                }
            }
            if rising_replay && stressed {
                pathologies.push(CognitivePathology::ReplayFixation);
            }

            // 8. Entropy Thrashing: Graph topology and entropy constantly oscillating
            // If entropy pressure swings from very high (>0.7) to very low (<0.3) over consecutive checks
            let mut oscillates = false;
            let first = history[0].vitals.entropy_pressure;
            let second = history[1].vitals.entropy_pressure;
            if history.len() >= 3 {
                let third = history[2].vitals.entropy_pressure;
                if (first > 0.6 && second < 0.4 && third > 0.6) || (first < 0.4 && second > 0.6 && third < 0.4) {
                    oscillates = true;
                }
            }
            if oscillates {
                pathologies.push(CognitivePathology::EntropyThrashing);
            }
        }

        pathologies
    }
}

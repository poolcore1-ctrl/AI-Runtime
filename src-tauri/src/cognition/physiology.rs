use serde::{Serialize, Deserialize};
use crate::cognition::pathology::CognitivePathology;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveVitals {
    pub entropy_pressure: f64,
    pub contradiction_density: f64,
    pub replay_instability: f64,
    pub provider_fatigue: f64,
    pub verifier_saturation: f64,
    pub graph_complexity_load: f64,
    pub memory_fragmentation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEnvironment {
    pub global_entropy: f64,
    pub provider_market_instability: f64,
    pub memory_reliability: f64,
    pub replay_confidence: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CognitiveStabilityState {
    Stable,
    ElevatedStress,
    Degraded,
    Critical,
    Recovery,
    Quarantined,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HomeostaticThresholds {
    pub escalation_threshold: f64, // Trigger level for rising stress (e.g. stability index falls below this)
    pub recovery_threshold: f64,   // Trigger level for returning to safety (must be higher stability than escalation)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologySnapshot {
    pub snapshot_id: String,
    pub vitals: CognitiveVitals,
    pub active_pathologies: Vec<CognitivePathology>,
    pub stability_state: CognitiveStabilityState,
    pub timestamp: i64,
}

pub struct CognitivePhysiologyEngine {
    pub thresholds: HomeostaticThresholds,
}

impl CognitivePhysiologyEngine {
    pub fn new() -> Self {
        Self {
            thresholds: HomeostaticThresholds {
                escalation_threshold: 0.60, // Stability falls below 60% -> Escalate stress
                recovery_threshold: 0.75,   // Stability must rise above 75% -> Recovery/Stable
            },
        }
    }

    /// Evaluates the complete ASOS Stability Equation to compute a live stability score in [0.0, 1.0].
    /// Formula incorporates all vital factors, environment stress, and active pathologies.
    pub fn calculate_stability(
        &self,
        vitals: &CognitiveVitals,
        env: &CognitiveEnvironment,
        pathology_count: usize,
    ) -> f64 {
        // Base stability starts at 1.0
        let mut s_cognition = 1.0;

        // Apply Vital Penalties
        s_cognition *= 1.0 - (vitals.entropy_pressure.min(1.0) * 0.25);
        s_cognition *= 1.0 - (vitals.contradiction_density.min(1.0) * 0.35);
        s_cognition *= 1.0 - (vitals.replay_instability.min(1.0) * 0.20);
        s_cognition *= 1.0 - (vitals.provider_fatigue.min(1.0) * 0.15);
        s_cognition *= 1.0 - (vitals.verifier_saturation.min(1.0) * 0.10);
        s_cognition *= 1.0 - (vitals.memory_fragmentation.min(1.0) * 0.05);

        // Apply Environment Multipliers (scale stability gently based on conditions)
        let env_factor = (1.0 - (1.0 - env.memory_reliability).min(1.0) * 0.20)
            * (1.0 - (1.0 - env.replay_confidence).min(1.0) * 0.20)
            * (1.0 - (env.global_entropy.min(1.0) * 0.15))
            * (1.0 - (env.provider_market_instability.min(1.0) * 0.10));
        s_cognition *= env_factor.max(0.4).min(1.0);

        // Apply Pathology Penalty (Severe drain on cognitive stability)
        if pathology_count > 0 {
            let pathology_drain = 0.15 * (pathology_count as f64);
            s_cognition -= pathology_drain;
        }

        s_cognition.max(0.0).min(1.0)
    }

    /// Transitions stability state using dynamic hysteresis thresholds.
    /// Prevents oscillating between states when stability hovers near boundaries.
    pub fn transition_state(
        &self,
        current_state: CognitiveStabilityState,
        stability_score: f64,
        pathology_count: usize,
    ) -> CognitiveStabilityState {
        if pathology_count >= 4 {
            return CognitiveStabilityState::Quarantined;
        }

        match current_state {
            CognitiveStabilityState::Stable => {
                if stability_score < self.thresholds.escalation_threshold {
                    CognitiveStabilityState::ElevatedStress
                } else {
                    CognitiveStabilityState::Stable
                }
            }
            CognitiveStabilityState::ElevatedStress => {
                if stability_score < 0.40 {
                    CognitiveStabilityState::Degraded
                } else if stability_score >= self.thresholds.recovery_threshold {
                    CognitiveStabilityState::Stable
                } else {
                    CognitiveStabilityState::ElevatedStress
                }
            }
            CognitiveStabilityState::Degraded => {
                if stability_score < 0.20 {
                    CognitiveStabilityState::Critical
                } else if stability_score >= self.thresholds.recovery_threshold {
                    CognitiveStabilityState::Recovery
                } else {
                    CognitiveStabilityState::Degraded
                }
            }
            CognitiveStabilityState::Critical => {
                if stability_score >= 0.45 {
                    CognitiveStabilityState::Recovery
                } else {
                    CognitiveStabilityState::Critical
                }
            }
            CognitiveStabilityState::Recovery => {
                if stability_score >= self.thresholds.recovery_threshold && pathology_count == 0 {
                    CognitiveStabilityState::Stable
                } else if stability_score < self.thresholds.escalation_threshold {
                    CognitiveStabilityState::ElevatedStress
                } else {
                    CognitiveStabilityState::Recovery
                }
            }
            CognitiveStabilityState::Quarantined => {
                if pathology_count == 0 && stability_score >= self.thresholds.recovery_threshold {
                    CognitiveStabilityState::Recovery
                } else {
                    CognitiveStabilityState::Quarantined
                }
            }
        }
    }
}

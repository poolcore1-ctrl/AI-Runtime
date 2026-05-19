use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalStabilityState {
    pub composite_instability: f64,
    pub cognitive_entropy: f64,
    pub is_system_stable: bool,
}

pub struct GlobalCognitiveSignalCompressor;

impl GlobalCognitiveSignalCompressor {
    pub fn new() -> Self {
        Self
    }

    /// Reduces broad multi-regulator signals into a decoupled, simplified global stability state.
    /// This prevents O(n^2) system coupling and resolves regulatory deadlocks.
    pub fn compress_signals(
        &self,
        pressure_instability: f64,
        resonance_risk: f64,
        calibration_error: f64,
    ) -> GlobalStabilityState {
        let composite_instability = (pressure_instability * 0.40)
            + (resonance_risk * 0.40)
            + (calibration_error * 0.20);

        let cognitive_entropy = (pressure_instability * 0.50) + (resonance_risk * 0.50);

        // System is classified stable if composite risk bounds remain below 0.65 threshold
        let is_system_stable = composite_instability < 0.65;

        GlobalStabilityState {
            composite_instability,
            cognitive_entropy,
            is_system_stable,
        }
    }
}

pub struct StabilityArbitrator;

impl StabilityArbitrator {
    pub fn new() -> Self {
        Self
    }

    /// Cleanly resolves competing priority commands from separate regulators.
    /// Prevents deadlocks where recovery and optimization loops fight for speculative cycles.
    pub fn arbitrate_regulators(
        &self,
        fatigue_signal: f64,
        recovery_signal: f64,
        system_entropy: f64,
    ) -> &'static str {
        if system_entropy > 0.80 {
            "Quarantine" // Critical high entropy overrides all cycles
        } else if fatigue_signal > 0.70 && recovery_signal > 0.50 {
            "Sleep" // Rest takes priority over speculatives
        } else if recovery_signal > 0.80 {
            "Consolidate" // Memory index recovery
        } else {
            "Adapt" // Normal evolutionary pipeline
        }
    }
}

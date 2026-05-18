use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDriftVector {
    pub provider_name: String,
    pub reasoning_stability: f64,
    pub constraint_preservation: f64,
    pub replay_determinism: f64,
    pub behavioral_accuracy: f64,
    pub persistence_reliability: f64,
    pub token_efficiency: f64,
    pub latency_consistency: f64,
    pub longitudinal_stability_score: f64, // Aggregate
}

pub struct ProviderDriftObservatory {
    stable_alpha: f64,
    volatile_alpha: f64,
}

impl ProviderDriftObservatory {
    pub fn new() -> Self {
        Self {
            stable_alpha: 0.15,
            volatile_alpha: 0.35,
        }
    }

    /// Applies EWMA (Exponentially Weighted Moving Average) to drift metrics.
    /// S_t = alpha * x_t + (1 - alpha) * S_{t-1}
    pub fn update_drift_vector(
        &self, 
        current_vector: &mut ProviderDriftVector, 
        latest_execution_quality: f64, // e.g. success or failure mapping (0.0 to 1.0)
        is_volatile_model: bool
    ) {
        let alpha = if is_volatile_model { self.volatile_alpha } else { self.stable_alpha };
        
        // Example: updating the aggregate score using EWMA
        current_vector.longitudinal_stability_score = 
            (alpha * latest_execution_quality) + ((1.0 - alpha) * current_vector.longitudinal_stability_score);

        // Sub-vectors can also be updated individually based on telemetry
    }
}

pub struct ForecastDriftIndex {
    pub historical_brier_variance: f64,
    pub calibration_decay_rate: f64,
    pub half_life_epochs: f64,
}

impl ForecastDriftIndex {
    pub fn new(variance: f64, decay: f64, half_life: f64) -> Self {
        Self {
            historical_brier_variance: variance,
            calibration_decay_rate: decay,
            half_life_epochs: half_life,
        }
    }

    /// Evaluates if historical accuracy is decaying systematically over long horizons.
    pub fn calculate_calibration_decay(&self, elapsed_epochs: f64) -> f64 {
        let decay = self.calibration_decay_rate * (elapsed_epochs / self.half_life_epochs).exp2();
        decay.min(1.0).max(0.0)
    }
}

pub struct ConfidenceInflationDetector;

impl ConfidenceInflationDetector {
    pub fn new() -> Self {
        Self
    }

    /// Flags confidence inflation bubbles: high confidence combined with low actual accuracy.
    /// Acts as an immune firewall against runaway optimization cults or delusions.
    pub fn detect_inflation_bubble(&self, average_confidence: f64, actual_accuracy: f64) -> bool {
        // Epistemic Inflation: confidence exceeds accuracy by over 35%, or confidence is high (>0.80) while accuracy is low (<0.50)
        (average_confidence > 0.80 && actual_accuracy < 0.50)
            || (average_confidence - actual_accuracy > 0.35)
    }
}

pub struct LongitudinalDecayTracker;

impl LongitudinalDecayTracker {
    pub fn new() -> Self {
        Self
    }

    /// Causal models decay over temporal epochs.
    /// Confidence(t) = Confidence_0 * 2^(-(elapsed / half_life))
    pub fn calculate_longitudinal_decay(
        &self,
        initial_confidence: f64,
        elapsed_time_seconds: u64,
        half_life_seconds: u64,
    ) -> f64 {
        if half_life_seconds == 0 {
            return 0.0;
        }

        let ratio = elapsed_time_seconds as f64 / half_life_seconds as f64;
        let decayed = initial_confidence * 2.0_f64.powf(-ratio);
        decayed.max(0.0).min(1.0)
    }
}

pub struct ForecastCalibrationCurve {
    // Stores tuple: (predicted_risk_score, actual_outcome_score)
    pub historical_predictions: Vec<(f64, f64)>,
}

impl ForecastCalibrationCurve {
    pub fn new() -> Self {
        Self {
            historical_predictions: Vec::new(),
        }
    }

    pub fn record_prediction(&mut self, predicted_risk: f64, actual_outcome: f64) {
        self.historical_predictions.push((predicted_risk, actual_outcome));
    }

    /// Evaluates forecast accuracy over operational time.
    /// Computes Mean Absolute Error (MAE) between predicted risks and reality occurrences.
    pub fn calculate_calibration_error(&self) -> f64 {
        if self.historical_predictions.is_empty() {
            return 0.0;
        }

        let mut sum_error = 0.0;
        for (predicted, actual) in &self.historical_predictions {
            sum_error += (predicted - actual).abs();
        }

        sum_error / self.historical_predictions.len() as f64
    }
}

pub struct CausalValidationReplay;

impl CausalValidationReplay {
    pub fn new() -> Self {
        Self
    }

    /// Compares predicted latencies against actual live sandbox executions to yield reality delta scores.
    pub fn calculate_forecast_reality_delta(
        &self,
        predicted_latency_ms: u64,
        actual_latency_ms: u64,
    ) -> f64 {
        let max_val = predicted_latency_ms.max(actual_latency_ms) as f64;
        if max_val == 0.0 {
            return 0.0;
        }

        let diff = (predicted_latency_ms as f64 - actual_latency_ms as f64).abs();
        diff / max_val
    }
}

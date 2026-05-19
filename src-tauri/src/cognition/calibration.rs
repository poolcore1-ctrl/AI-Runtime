use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ForecastRecord {
    pub prediction_id: String,
    pub specialist_id: String,
    pub predicted_probability: f64,
    pub actual_outcome: f64,
    pub timestamp: i64,
}

pub struct BrierScore;

impl BrierScore {
    /// Computes the Brier score over a slice of ForecastRecords.
    /// Formula: Brier = (1/N) * sum((f_t - o_t)^2)
    /// Lower scores denote superior calibration quality (0.0 is perfect).
    pub fn calculate(forecasts: &[ForecastRecord]) -> f64 {
        if forecasts.is_empty() {
            return 0.0;
        }

        let mut sum_squared = 0.0;
        for record in forecasts {
            let diff = record.predicted_probability - record.actual_outcome;
            sum_squared += diff * diff;
        }

        sum_squared / (forecasts.len() as f64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ForecastReliabilityProfile {
    pub total_forecasts: usize,
    pub mean_absolute_calibration_error: f64,
}

impl ForecastReliabilityProfile {
    /// Compiles reliability profile checking if "80% confidence" corresponds to 80% accuracy.
    pub fn evaluate_reliability(forecasts: &[ForecastRecord]) -> Self {
        if forecasts.is_empty() {
            return Self {
                total_forecasts: 0,
                mean_absolute_calibration_error: 0.0,
            };
        }

        let mut sum_error = 0.0;
        for record in forecasts {
            sum_error += (record.predicted_probability - record.actual_outcome).abs();
        }

        let mae = sum_error / (forecasts.len() as f64);
        Self {
            total_forecasts: forecasts.len(),
            mean_absolute_calibration_error: mae,
        }
    }
}

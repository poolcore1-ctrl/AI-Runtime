use serde::{Serialize, Deserialize};
use crate::cognition::repository_identity::{RepositoryFailureClass, RepositoryIdentityProfile};
use crate::stress_testing::types::EntropyClass;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureForecast {
    pub predicted_failure_class: RepositoryFailureClass,
    pub confidence: f64,
    pub supporting_patterns: Vec<String>,
}

pub struct CognitiveForecaster;

impl CognitiveForecaster {
    pub fn new() -> Self {
        Self
    }

    /// Generates predictive failure forecasts based on repository identity and current entropy
    pub fn forecast_failures(
        &self, 
        identity: &RepositoryIdentityProfile, 
        current_entropy: EntropyClass
    ) -> Vec<FailureForecast> {
        let mut forecasts = Vec::new();

        // High entropy triggers structural fragility predictions
        if current_entropy == EntropyClass::Extreme && identity.architectural_fragility_score > 0.7 {
            forecasts.push(FailureForecast {
                predicted_failure_class: RepositoryFailureClass::DependencyCollapse,
                confidence: 0.85,
                supporting_patterns: vec!["Extreme entropy detected in highly fragile architecture".to_string()],
            });
        }

        // Detect recurring failures
        if identity.is_chronic_failure(&RepositoryFailureClass::AuthRegression) {
            forecasts.push(FailureForecast {
                predicted_failure_class: RepositoryFailureClass::AuthRegression,
                confidence: 0.90,
                supporting_patterns: vec!["Repository has historical tendency for Auth Regressions".to_string()],
            });
        }

        if identity.is_chronic_failure(&RepositoryFailureClass::HydrationMismatch) && identity.framework_stack.contains(&"React".to_string()) {
            forecasts.push(FailureForecast {
                predicted_failure_class: RepositoryFailureClass::HydrationMismatch,
                confidence: 0.95,
                supporting_patterns: vec!["React workspace with chronic hydration regressions detected".to_string()],
            });
        }

        forecasts
    }
}

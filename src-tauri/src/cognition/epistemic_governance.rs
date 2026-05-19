pub struct EpistemicConstitution {
    pub humility_threshold: f64,
    pub tax_rate: f64,
}

pub struct ConfidenceTaxation;

impl ConfidenceTaxation {
    pub fn new() -> Self {
        Self
    }

    /// Levies a calibration tax on speculative forecasts to prevent reckless over-optimization.
    pub fn calculate_tax(&self, predicted_probability: f64, impact_weight: f64) -> f64 {
        let confidence_excess = (predicted_probability - 0.50).abs();
        // High confidence + high impact = heavily taxed to preserve homeostasis bounds
        let tax = confidence_excess * impact_weight * 0.30;
        tax.max(0.0)
    }
}

pub struct SpeculationBudget {
    pub max_active_staked_capital: f64,
    pub current_staked_capital: f64,
}

impl SpeculationBudget {
    pub fn new(max_active_staked_capital: f64) -> Self {
        Self {
            max_active_staked_capital,
            current_staked_capital: 0.0,
        }
    }

    /// Allocates budget for an active forecast.
    /// Caps total active exposure to protect against bureaucratic simulation exhaustion.
    pub fn request_allocation(&mut self, amount: f64) -> bool {
        if self.current_staked_capital + amount > self.max_active_staked_capital {
            return false;
        }

        self.current_staked_capital += amount;
        true
    }

    pub fn release_allocation(&mut self, amount: f64) {
        self.current_staked_capital = (self.current_staked_capital - amount).max(0.0);
    }
}

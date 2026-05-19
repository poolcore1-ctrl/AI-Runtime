pub struct ResonanceAmplificationMatrix;

impl ResonanceAmplificationMatrix {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates non-linear combined feedback risks.
    /// If change A increases lock contention, and change B increases retry loops,
    /// their combined effect is not merely additive, but is amplified by cross-feedback resonance.
    pub fn calculate_combined_resonance(
        &self,
        base_risk_a: f64,
        base_risk_b: f64,
        cross_feedback_multiplier: f64,
    ) -> f64 {
        let linear_sum = base_risk_a + base_risk_b;
        let resonance_bonus = linear_sum * cross_feedback_multiplier;
        let composite_risk = linear_sum + resonance_bonus;

        composite_risk.min(1.0).max(0.0)
    }
}

pub struct SubsystemInterferenceTracker {
    pub active_interference_factors: Vec<f64>,
}

impl SubsystemInterferenceTracker {
    pub fn new() -> Self {
        Self {
            active_interference_factors: Vec::new(),
        }
    }

    pub fn add_interference(&mut self, factor: f64) {
        self.active_interference_factors.push(factor);
    }

    /// Combines multiple interfering metrics using sub-additive consolidation:
    /// 1.0 - (Product_i (1.0 - factor_i))
    pub fn get_cumulative_interference(&self) -> f64 {
        if self.active_interference_factors.is_empty() {
            return 0.0;
        }

        let mut cumulative_safety = 1.0;
        for factor in &self.active_interference_factors {
            let safety = 1.0 - factor.max(0.0).min(1.0);
            cumulative_safety *= safety;
        }

        (1.0 - cumulative_safety).max(0.0).min(1.0)
    }
}

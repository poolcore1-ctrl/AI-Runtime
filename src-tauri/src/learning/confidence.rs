use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfidence {
    pub success_rate: f32,
    pub stability_score: f32,
    pub verification_reliability: f32,
    pub application_count: usize,
}

impl Default for StrategyConfidence {
    fn default() -> Self {
        Self {
            success_rate: 1.0,
            stability_score: 1.0,
            verification_reliability: 1.0,
            application_count: 1,
        }
    }
}

impl StrategyConfidence {
    pub fn update(&mut self, success: bool, retries: usize) {
        self.application_count += 1;
        
        let success_val = if success { 1.0 } else { 0.0 };
        self.success_rate = (self.success_rate * 0.9) + (success_val * 0.1);

        let stability = if retries == 1 { 1.0 } else { 0.5 };
        self.stability_score = (self.stability_score * 0.9) + (stability * 0.1);
    }
}

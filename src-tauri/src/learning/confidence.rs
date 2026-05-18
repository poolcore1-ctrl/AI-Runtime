use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfidence {
    pub success_rate: f32,
    pub stability_score: f32,
    pub verification_reliability: f32,
    pub application_count: usize,
    pub consecutive_failures: usize,
    pub last_decay_timestamp: u64,
}

impl Default for StrategyConfidence {
    fn default() -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        Self {
            success_rate: 1.0,
            stability_score: 1.0,
            verification_reliability: 1.0,
            application_count: 1,
            consecutive_failures: 0,
            last_decay_timestamp: now,
        }
    }
}

impl StrategyConfidence {
    pub fn update(&mut self, success: bool, retries: usize) {
        self.application_count += 1;
        
        let success_val = if success { 1.0 } else { 0.0 };
        self.success_rate = (self.success_rate * 0.85) + (success_val * 0.15);

        let stability = if retries <= 1 { 1.0 } else { 0.5 };
        self.stability_score = (self.stability_score * 0.85) + (stability * 0.15);

        if success {
            self.consecutive_failures = 0;
            self.verification_reliability = (self.verification_reliability * 0.95) + 0.05;
        } else {
            self.consecutive_failures += 1;
            self.verification_reliability = (self.verification_reliability * 0.80).max(0.0);
        }
    }

    /// Implement dynamic exponential confidence decay:
    /// C_new = C_old * e^(-lambda * t)
    pub fn decay(&mut self, current_time: u64, lambda: f32) {
        if self.last_decay_timestamp == 0 {
            self.last_decay_timestamp = current_time;
            return;
        }

        if current_time > self.last_decay_timestamp {
            // t in days (86400 seconds)
            let dt = (current_time - self.last_decay_timestamp) as f32 / 86400.0;
            if dt > 0.001 {
                let factor = (-lambda * dt).exp();
                self.success_rate = (self.success_rate * factor).max(0.0);
                self.stability_score = (self.stability_score * factor).max(0.0);
                self.verification_reliability = (self.verification_reliability * factor).max(0.0);
                self.last_decay_timestamp = current_time;
            }
        }
    }
}

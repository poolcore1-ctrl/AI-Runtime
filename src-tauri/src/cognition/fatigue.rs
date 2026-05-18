use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderFatigueProfile {
    pub provider_name: String,
    pub fatigue_score: f64,                  // 0.0 (pristine) to 1.0 (unusable)
    pub hallucination_acceleration: f64,     // multiplier that scales up hallucination risk
    pub recovery_rate: f64,                  // fatigue recovery per cooldown unit
    pub sustained_reasoning_limit: usize,    // sequential steps before fatigue spikes exponentially
}

pub struct ProviderFatigueModel {
    pub profiles: HashMap<String, ProviderFatigueProfile>,
}

impl ProviderFatigueModel {
    pub fn new() -> Self {
        let mut model = Self {
            profiles: HashMap::new(),
        };

        // Initialize defaults for primary ASOS providers
        model.profiles.insert("claude".to_string(), ProviderFatigueProfile {
            provider_name: "Claude".to_string(),
            fatigue_score: 0.0,
            hallucination_acceleration: 1.0,
            recovery_rate: 0.08,
            sustained_reasoning_limit: 8,
        });

        model.profiles.insert("gemini".to_string(), ProviderFatigueProfile {
            provider_name: "Gemini".to_string(),
            fatigue_score: 0.0,
            hallucination_acceleration: 1.1,
            recovery_rate: 0.10, // Faster recovery due to vast context cache
            sustained_reasoning_limit: 12,
        });

        model.profiles.insert("deepseek".to_string(), ProviderFatigueProfile {
            provider_name: "DeepSeek".to_string(),
            fatigue_score: 0.0,
            hallucination_acceleration: 1.2,
            recovery_rate: 0.07,
            sustained_reasoning_limit: 6,
        });

        model.profiles.insert("local ollama".to_string(), ProviderFatigueProfile {
            provider_name: "Local Ollama".to_string(),
            fatigue_score: 0.0,
            hallucination_acceleration: 1.8, // Spikes quickly
            recovery_rate: 0.04,             // Cools slowly
            sustained_reasoning_limit: 3,
        });

        model
    }

    /// Increments fatigue score based on dynamic usage, complexity, and steps taken.
    /// If usage exceeds sustained_reasoning_limit, fatigue ramps up exponentially.
    pub fn record_usage(&mut self, provider: &str, steps_taken: usize, complexity_score: f64) {
        let key = provider.to_lowercase();
        if let Some(profile) = self.profiles.get_mut(&key) {
            let limit_exceeded = steps_taken > profile.sustained_reasoning_limit;
            let base_increment = (steps_taken as f64 * 0.04) + (complexity_score * 0.12);
            
            let final_increment = if limit_exceeded {
                let excess = steps_taken - profile.sustained_reasoning_limit;
                base_increment * (1.5 + (excess as f64 * 0.3))
            } else {
                base_increment
            };

            // Fatigue score increases, capped at 1.0
            profile.fatigue_score = (profile.fatigue_score + final_increment).min(1.0);

            // Behavioral collapse modeling: as fatigue rises, hallucination acceleration escalates
            if profile.fatigue_score > 0.70 {
                profile.hallucination_acceleration *= 1.4; // Exponential spike in hallucination rate
            }
        }
    }

    /// Decays the fatigue levels of all registered providers.
    pub fn cool_down(&mut self, cycles: f64) {
        for profile in self.profiles.values_mut() {
            let decay = profile.recovery_rate * cycles;
            profile.fatigue_score = (profile.fatigue_score - decay).max(0.0);
            
            // Gradually restore baseline hallucination acceleration
            if profile.fatigue_score < 0.30 {
                profile.hallucination_acceleration = match profile.provider_name.to_lowercase().as_str() {
                    "claude" => 1.0,
                    "gemini" => 1.1,
                    "deepseek" => 1.2,
                    _ => 1.8,
                };
            }
        }
    }

    /// Gets the fatigue score of a provider, defaults to 0.0 if not found
    pub fn get_fatigue_score(&self, provider: &str) -> f64 {
        self.profiles.get(&provider.to_lowercase())
            .map(|p| p.fatigue_score)
            .unwrap_or(0.0)
    }

    /// Calculates a live hallucination scalar multiplier for strategy execution
    pub fn get_hallucination_multiplier(&self, provider: &str) -> f64 {
        self.profiles.get(&provider.to_lowercase())
            .map(|p| p.hallucination_acceleration * (1.0 + p.fatigue_score * 0.5))
            .unwrap_or(1.0)
    }
}

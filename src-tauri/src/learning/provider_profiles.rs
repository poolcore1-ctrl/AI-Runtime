pub struct ProviderCapabilityProfile {
    pub provider_name: String,
    pub reasoning_depth: f64,
    pub context_window: usize,
    pub json_adherence: f64,
    pub tool_reliability: f64,
    pub hallucination_rate: f64,
    pub semantic_stability: f64,
    pub replay_consistency: f64,
}

impl ProviderCapabilityProfile {
    pub fn get_profile(provider: &str) -> Self {
        match provider.to_lowercase().as_str() {
            "claude" => Self {
                provider_name: "Claude".to_string(),
                reasoning_depth: 0.95,
                context_window: 200000,
                json_adherence: 0.92,
                tool_reliability: 0.96,
                hallucination_rate: 0.05,
                semantic_stability: 0.94,
                replay_consistency: 0.95,
            },
            "gemini" => Self {
                provider_name: "Gemini".to_string(),
                reasoning_depth: 0.90,
                context_window: 1000000,
                json_adherence: 0.98,
                tool_reliability: 0.92,
                hallucination_rate: 0.06,
                semantic_stability: 0.90,
                replay_consistency: 0.92,
            },
            "deepseek" => Self {
                provider_name: "DeepSeek".to_string(),
                reasoning_depth: 0.94,
                context_window: 64000,
                json_adherence: 0.90,
                tool_reliability: 0.88,
                hallucination_rate: 0.04,
                semantic_stability: 0.92,
                replay_consistency: 0.94,
            },
            _ => Self {
                provider_name: "Local Ollama".to_string(),
                reasoning_depth: 0.60,
                context_window: 8192,
                json_adherence: 0.70,
                tool_reliability: 0.65,
                hallucination_rate: 0.20,
                semantic_stability: 0.60,
                replay_consistency: 0.65,
            },
        }
    }
}

use crate::cognition::immune::{CognitivePathology};

pub struct CognitiveDiversityEnforcer;

impl CognitiveDiversityEnforcer {
    pub fn new() -> Self {
        Self
    }

    /// Detects ProviderMonoculture when one provider is handling too large a share of execution
    pub fn detect_provider_monoculture(&self, dominant_provider_share: f64) -> Option<CognitivePathology> {
        if dominant_provider_share >= 0.75 {
            return Some(CognitivePathology::ProviderMonoculture);
        }
        None
    }

    /// Calculates a provider diversity index (range 0.0 to 1.0)
    /// Higher is better — evenly distributed routing produces max diversity
    pub fn calculate_diversity_index(&self, provider_shares: &[f64]) -> f64 {
        if provider_shares.is_empty() {
            return 0.0;
        }
        let n = provider_shares.len() as f64;
        // Gini impurity-inspired diversity: 1 - sum(share^2)
        let concentration: f64 = provider_shares.iter().map(|s| s * s).sum();
        (1.0 - concentration).max(0.0) * (n / (n - 1.0).max(1.0))
    }
}

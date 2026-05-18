use crate::learning::ir::StrategyIR;
use crate::learning::provider_profiles::ProviderCapabilityProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftRisk {
    Minimal,
    Moderate,
    High,
    Critical,
}

pub struct CrossModelDriftPredictor;

impl CrossModelDriftPredictor {
    pub fn predict_stability(&self, ir: &StrategyIR, source: &str, target: &str) -> (f64, DriftRisk) {
        let s_profile = ProviderCapabilityProfile::get_profile(source);
        let t_profile = ProviderCapabilityProfile::get_profile(target);

        // 1. Calculate provider capability ratio (D_model)
        let d_model = if s_profile.reasoning_depth > 0.0 {
            t_profile.reasoning_depth / s_profile.reasoning_depth
        } else {
            1.0
        };

        // 2. Calculate complexity penalty (C_complexity)
        let step_count = ir.normalized_steps.len();
        let c_complexity = if step_count > 3 {
            // 0.04 per step to keep high-quality provider migrations (e.g. Claude→DeepSeek)
            // scoring above the 0.70 Moderate threshold for realistic 5-step strategies
            1.0 - (step_count as f64 * 0.04).min(0.25)
        } else {
            1.0
        };

        // 3. Replay consistency factor (R_replay)
        let r_replay = t_profile.replay_consistency;

        // 4. Hallucination degradation factor (H_hallucination)
        let h_hallucination = 1.0 - t_profile.hallucination_rate;

        // 5. Apply the complete ASOS Transfer Stability formula
        let s_base = 1.0;
        let s_transfer = s_base * d_model * c_complexity * r_replay * h_hallucination;

        // Classify into DriftRisk cases
        let risk = if s_transfer >= 0.85 {
            DriftRisk::Minimal
        } else if s_transfer >= 0.70 {
            DriftRisk::Moderate
        } else if s_transfer >= 0.50 {
            DriftRisk::High
        } else {
            DriftRisk::Critical
        };

        (s_transfer.max(0.0).min(1.0), risk)
    }
}

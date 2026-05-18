use crate::learning::ir::{StrategyIR, ConstraintSeverity};
use crate::learning::drift_predictor::{CrossModelDriftPredictor, DriftRisk};

pub struct CognitiveStabilityRouter;

impl CognitiveStabilityRouter {
    pub fn select_optimal_provider(
        &self,
        ir: &StrategyIR,
        source: &str,
        available_providers: &[&str],
        entropy_score: f64
    ) -> String {
        let predictor = CrossModelDriftPredictor;
        let mut best_provider = source.to_string();
        let mut highest_score = 0.0;

        // Check if strategy has critical constraints
        let has_critical_constraints = ir.constraints.iter().any(|c| c.severity == ConstraintSeverity::Critical);

        for provider in available_providers {
            let (score, risk) = predictor.predict_stability(ir, source, provider);
            
            // Heuristic gating rules:
            // 1. High environmental entropy blocks weak/local providers
            if entropy_score > 0.7 && provider.contains("local") {
                continue;
            }

            // 2. Critical constraints cannot use high or critical risk configurations
            if has_critical_constraints && (risk == DriftRisk::High || risk == DriftRisk::Critical) {
                continue;
            }

            if score > highest_score {
                highest_score = score;
                best_provider = provider.to_string();
            }
        }

        best_provider
    }
}

use serde::{Serialize, Deserialize};
use crate::learning::ir::{StrategyIR, ConstraintType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationValidationResult {
    pub semantically_equivalent: bool,
    pub semantic_preservation_score: f64,
    pub missing_constraints: Vec<String>,
    pub missing_steps: Vec<String>,
    pub safety_checks_failed: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct TranslationValidator;

impl TranslationValidator {
    pub fn validate(&self, ir: &StrategyIR, compiled_prompt: &str) -> TranslationValidationResult {
        let mut missing_constraints = Vec::new();
        let mut missing_steps = Vec::new();
        let mut safety_failed = Vec::new();
        let warnings = Vec::new();

        let prompt_lower = compiled_prompt.to_lowercase();

        // 1. Audit generic steps preservation
        for step in &ir.normalized_steps {
            let target_lower = step.target_file.to_lowercase();
            if !prompt_lower.contains(&target_lower) {
                missing_steps.push(format!("Missing step target file: {}", step.target_file));
            }
        }

        // 2. Audit and check security-critical constraints are explicitly preserved
        for constraint in &ir.constraints {
            let expr_lower = constraint.expression.to_lowercase();
            let expr_terms: Vec<&str> = expr_lower.split_whitespace().collect();
            let mut match_count = 0;
            for term in &expr_terms {
                if term.len() > 3 && prompt_lower.contains(term) {
                    match_count += 1;
                }
            }

            if match_count == 0 {
                let msg = format!("Missing constraint: {:?}", constraint.constraint_type);
                missing_constraints.push(msg.clone());

                // If constraint is security critical, trigger safety failure
                match constraint.constraint_type {
                    ConstraintType::PreserveAuthentication | ConstraintType::PreventSecurityRegression => {
                        safety_failed.push(format!("Critical Safety breach: security constraints stripped! ({:?})", constraint.constraint_type));
                    }
                    _ => {}
                }
            }
        }

        // 3. Compute continuous semantic preservation score
        let total_checks = ir.normalized_steps.len() + ir.constraints.len();
        let missed_checks = missing_steps.len() + missing_constraints.len();
        
        let semantic_preservation_score = if total_checks > 0 {
            1.0 - (missed_checks as f64 / total_checks as f64)
        } else {
            1.0
        };

        let semantically_equivalent = safety_failed.is_empty() && semantic_preservation_score > 0.8;

        TranslationValidationResult {
            semantically_equivalent,
            semantic_preservation_score,
            missing_constraints,
            missing_steps,
            safety_checks_failed: safety_failed,
            warnings,
        }
    }
}

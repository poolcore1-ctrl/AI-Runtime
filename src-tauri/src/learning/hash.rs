use sha2::{Sha256, Digest};
use crate::learning::ir::StrategyIR;

pub fn compute_semantic_hash(ir: &StrategyIR) -> String {
    let mut hasher = Sha256::new();
    
    // 1. Canonicalize and sort targets
    let mut sorted_targets = ir.target_symbols.clone();
    sorted_targets.sort();
    
    // 2. Canonicalize and sort constraints
    let mut sorted_constraints: Vec<String> = ir.constraints.iter().map(|c| {
        let clean_expr = c.expression.split_whitespace().collect::<Vec<&str>>().join(" ");
        format!("{:?}:{:?}:{}", c.constraint_type, c.severity, clean_expr)
    }).collect();
    sorted_constraints.sort();

    // 3. Feed stable, canonicalized components to SHA-256
    hasher.update(ir.objective.split_whitespace().collect::<Vec<&str>>().join(" ").as_bytes());
    
    for target in sorted_targets {
        hasher.update(target.as_bytes());
    }
    for constraint_str in sorted_constraints {
        hasher.update(constraint_str.as_bytes());
    }
    
    // Feed steps in sequence (since step execution order is semantically meaningful)
    for step in &ir.normalized_steps {
        hasher.update(step.step_id.as_bytes());
        hasher.update(format!("{:?}", step.action_type).as_bytes());
        hasher.update(step.target_file.as_bytes());
        hasher.update(step.instructions.split_whitespace().collect::<Vec<&str>>().join(" ").as_bytes());
    }
    
    format!("{:x}", hasher.finalize())
}

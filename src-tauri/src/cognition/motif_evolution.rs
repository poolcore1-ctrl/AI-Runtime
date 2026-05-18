use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotifLineage {
    pub motif_id: String,
    pub parent_motif: Option<String>,
    pub generation: u64,
    pub mutation_reason: String,
    pub success_delta: f64,
}

pub struct MotifEvolutionEngine;

impl MotifEvolutionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates if a newer motif generation supersedes an older one based on historical success delta
    pub fn evaluate_evolution(&self, _lineage: &MotifLineage, historical_parent_success: f64, current_success: f64) -> bool {
        let delta = current_success - historical_parent_success;
        // If the newer generation outperforms the older by at least 2%, promote it
        delta > 0.02
    }
}

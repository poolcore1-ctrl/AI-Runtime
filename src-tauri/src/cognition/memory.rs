use serde::{Serialize, Deserialize};
use crate::stress_testing::types::EntropyClass;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveMemoryRecord {
    pub memory_id: String,
    pub repository_fingerprint: String,
    pub task_category: String,
    pub entropy_class: EntropyClass,
    pub graph_hash: String,
    pub semantic_hash: String,
    pub provider_chain: Vec<String>,
    pub verification_outcome: String,
    pub behavioral_drift_score: f64,
    pub execution_cost_usd: f64,
    pub timestamp: i64,
}

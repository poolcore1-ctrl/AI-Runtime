use serde::{Serialize, Deserialize};
use crate::learning::confidence::StrategyConfidence;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringStrategy {
    pub id: String,
    pub pattern_name: String,
    pub conditions: Vec<String>,
    pub steps: Vec<String>,
    pub architectural_context: Option<String>,
    pub confidence: StrategyConfidence,
}

impl EngineeringStrategy {
    pub fn new(pattern_name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            pattern_name,
            conditions: Vec::new(),
            steps: Vec::new(),
            architectural_context: None,
            confidence: StrategyConfidence::default(),
        }
    }
}

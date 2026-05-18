pub mod abstraction;
pub mod strategies;
pub mod fingerprints;
pub mod confidence;
pub mod retrieval;
pub mod evolution;

use std::sync::Arc;
use crate::learning::retrieval::StrategyStore;
use crate::learning::abstraction::AbstractionEngine;
use crate::learning::evolution::KnowledgeEvolver;

pub struct LearningEngine {
    pub store: Arc<StrategyStore>,
    pub abstraction: Arc<AbstractionEngine>,
    pub evolver: Arc<KnowledgeEvolver>,
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            store: Arc::new(StrategyStore::new()),
            abstraction: Arc::new(AbstractionEngine::new()),
            evolver: Arc::new(KnowledgeEvolver::new()),
        }
    }
}

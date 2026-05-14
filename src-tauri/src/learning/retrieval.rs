use crate::learning::strategies::EngineeringStrategy;
use dashmap::DashMap;
use std::sync::Arc;

pub struct StrategyStore {
    /// In-memory storage of learned strategies indexed by pattern name.
    pub strategies: Arc<DashMap<String, Vec<EngineeringStrategy>>>,
}

impl StrategyStore {
    pub fn new() -> Self {
        Self {
            strategies: Arc::new(DashMap::new()),
        }
    }

    /// Saves a newly learned strategy to the store.
    pub fn save(&self, strategy: EngineeringStrategy) {
        self.strategies.entry(strategy.pattern_name.clone())
            .or_insert_with(Vec::new)
            .push(strategy);
    }

    /// Retrieves all strategies matching a specific pattern name.
    pub fn find_by_pattern(&self, pattern_name: &str) -> Vec<EngineeringStrategy> {
        self.strategies.get(pattern_name)
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Finds the highest confidence strategy for a given pattern.
    pub fn get_best_strategy(&self, pattern_name: &str) -> Option<EngineeringStrategy> {
        self.find_by_pattern(pattern_name)
            .into_iter()
            .max_by(|a, b| a.confidence.success_rate.partial_cmp(&b.confidence.success_rate).unwrap())
    }
}

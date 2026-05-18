use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::stress_testing::types::EntropyClass;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RepositoryFailureClass {
    HydrationMismatch,
    TypeWeakening,
    DependencyCollapse,
    AuthRegression,
    PersistenceCorruption,
    ApiContractBreach,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryIdentityProfile {
    pub repository_fingerprint: String,
    pub dominant_languages: Vec<String>,
    pub framework_stack: Vec<String>,
    pub entropy_baseline: EntropyClass,
    pub recurring_failure_classes: Vec<RepositoryFailureClass>,
    pub avg_verification_cost: f64,
    pub provider_affinity_scores: HashMap<String, f64>,
    pub architectural_fragility_score: f64,
    pub behavioral_instability_score: f64,
    pub persistence_reliability_score: f64,
}

impl RepositoryIdentityProfile {
    pub fn new(fingerprint: &str) -> Self {
        Self {
            repository_fingerprint: fingerprint.to_string(),
            dominant_languages: vec![],
            framework_stack: vec![],
            entropy_baseline: EntropyClass::Stable,
            recurring_failure_classes: vec![],
            avg_verification_cost: 0.0,
            provider_affinity_scores: HashMap::new(),
            architectural_fragility_score: 0.0,
            behavioral_instability_score: 0.0,
            persistence_reliability_score: 1.0,
        }
    }

    /// Evaluates if a given failure class is historically chronic in this repo
    pub fn is_chronic_failure(&self, failure_class: &RepositoryFailureClass) -> bool {
        self.recurring_failure_classes.contains(failure_class)
    }
}

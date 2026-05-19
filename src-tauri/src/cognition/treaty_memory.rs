use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::cognition::specialist::SpecialistDomain;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreatyInteractionRecord {
    pub participants: Vec<SpecialistDomain>,
    pub action: String,
    pub treaty_compliance_score: f64,
    pub long_term_outcome: f64,
    pub timestamp: i64,
}

pub struct ReputationEngine {
    pub trust_capital: f64,
    pub contextual_trust: HashMap<String, f64>,
}

impl ReputationEngine {
    pub fn new(initial_capital: f64) -> Self {
        Self {
            trust_capital: initial_capital,
            contextual_trust: HashMap::new(),
        }
    }

    /// Records a dynamic diplomatic interaction, adjusting both global trust capital
    /// and contextual capability trust maps based on compliance.
    pub fn record_interaction(&mut self, record: &TreatyInteractionRecord, context: &str) {
        // Adjust global capital
        let compliance_delta = record.treaty_compliance_score - 0.50;
        self.trust_capital = (self.trust_capital + (compliance_delta * 0.10)).min(1.0).max(0.0);

        // Adjust contextual trust
        let current_context_val = self.contextual_trust.entry(context.to_string()).or_insert(self.trust_capital);
        let context_delta = (record.treaty_compliance_score * 0.70 + record.long_term_outcome * 0.30) - 0.50;
        *current_context_val = (*current_context_val + (context_delta * 0.15)).min(1.0).max(0.0);
    }

    pub fn get_contextual_trust(&self, context: &str) -> f64 {
        *self.contextual_trust.get(context).unwrap_or(&self.trust_capital)
    }
}

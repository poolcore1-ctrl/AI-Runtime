use serde::{Serialize, Deserialize};
use crate::stress_testing::types::EntropyClass;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyTimelineEvent {
    pub timeline_id: String,
    pub repository_fingerprint: String,
    pub entropy_class: EntropyClass,
    pub timestamp: i64,
}

pub struct TemporalAnalyticsEngine;

impl TemporalAnalyticsEngine {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates if a repository's entropy is escalating over time (e.g. Stable -> Moderate -> Extreme)
    pub fn detect_entropy_escalation(&self, history: &[EntropyTimelineEvent]) -> bool {
        if history.len() < 2 {
            return false;
        }
        
        let oldest = &history[0];
        let newest = &history[history.len() - 1];

        // Basic escalation detection: Extreme > Moderate > Stable
        if oldest.entropy_class == EntropyClass::Stable && newest.entropy_class == EntropyClass::Extreme {
            return true;
        }
        
        false
    }
}

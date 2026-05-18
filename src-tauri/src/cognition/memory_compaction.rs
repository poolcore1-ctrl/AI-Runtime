use serde::{Serialize, Deserialize};
use crate::cognition::memory::CognitiveMemoryRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongitudinalSummary {
    pub summary_id: String,
    pub total_runs: u64,
    pub avg_success_rate: f64,
    pub avg_cost: f64,
    pub dominant_failure_class: String,
}

pub struct MemoryCompactionEngine;

impl MemoryCompactionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Compresses a set of high-granularity hot memory records into a single Cold memory statistical summary.
    pub fn compact_hot_memory(&self, hot_records: &[CognitiveMemoryRecord], summary_id: &str) -> Option<LongitudinalSummary> {
        if hot_records.is_empty() {
            return None;
        }

        let total_runs = hot_records.len() as u64;
        let mut total_cost = 0.0;
        let mut success_count = 0;
        
        for record in hot_records {
            total_cost += record.execution_cost_usd;
            if record.verification_outcome == "Success" {
                success_count += 1;
            }
        }

        let avg_cost = total_cost / total_runs as f64;
        let avg_success_rate = success_count as f64 / total_runs as f64;

        Some(LongitudinalSummary {
            summary_id: summary_id.to_string(),
            total_runs,
            avg_success_rate,
            avg_cost,
            dominant_failure_class: "Unknown".to_string(), // In full system, this calculates the mode of failure classes
        })
    }
}

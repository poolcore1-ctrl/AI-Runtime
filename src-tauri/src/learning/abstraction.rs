use crate::runtime::reports::{RepairTraceReport, RepairOutcome};
use crate::learning::strategies::EngineeringStrategy;
use crate::learning::fingerprints::StrategyFingerprint;
use crate::runtime::errors::FailureKind;
use anyhow::{Result, anyhow};

pub struct AbstractionEngine;

impl AbstractionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Transforms a verified repair trace into a generalized strategy.
    pub fn abstract_trace(&self, report: &RepairTraceReport) -> Result<EngineeringStrategy> {
        // 1. Safety Gate: Only learn from successful outcomes
        if report.final_outcome != RepairOutcome::Success {
            return Err(anyhow!("Cannot abstract an unsuccessful repair trace"));
        }

        // 2. Identify the Fingerprint based on the initial failure
        let fingerprint = match report.initial_failure.kind {
            FailureKind::TypeScript => StrategyFingerprint::TypeScriptStructuralExtension,
            FailureKind::Dependency => StrategyFingerprint::MissingDependencyResolution,
            _ => return Err(anyhow!("Unsupported failure kind for abstraction")),
        };

        // 3. Construct the strategy
        let mut strategy = EngineeringStrategy::new(fingerprint.as_str().to_string());
        
        // 4. Distill steps from attempts
        for attempt in &report.attempts {
            strategy.steps.push(attempt.proposed_patch.clone());
        }

        // 5. Capture architectural context (simplified)
        strategy.architectural_context = Some(format!(
            "Applies to {} symbols across {} files",
            report.attempts.last().map(|a| a.retrieved_context_ids.len()).unwrap_or(0),
            report.initial_failure.metadata.get("affected_files_count").unwrap_or(&"0".to_string())
        ));

        Ok(strategy)
    }
}

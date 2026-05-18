use crate::runtime::reports::{RepairTraceReport, RepairOutcome};
use crate::learning::strategies::{EngineeringStrategy, StrategyState, VerificationSurfaceCoverage};
use crate::learning::fingerprints::StrategyFingerprint;
use crate::learning::anti_poisoning::{AntiPoisoningGuard, RepairIntegrity};
use crate::runtime::errors::FailureKind;
use anyhow::{Result, anyhow};
use tracing::{info, warn, instrument};

pub struct AbstractionEngine;

impl AbstractionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Transforms a verified repair trace into a generalized strategy under rigorous memory safety criteria.
    #[instrument(skip(self, report))]
    pub fn abstract_trace(&self, report: &RepairTraceReport) -> Result<EngineeringStrategy> {
        info!("Running abstraction pipeline with anti-poisoning safeguards");

        // 1. Safety Gate: Only learn from successful outcomes
        if report.final_outcome != RepairOutcome::Success {
            return Err(anyhow!("Memory Safety: Cannot abstract an unsuccessful repair trace"));
        }

        // 2. Safe Learning Criteria: Prevent learning if rollbacks / repeats occurred
        if report.attempts.len() > 4 {
            return Err(anyhow!("Memory Safety: Rejecting strategy due to high instability / rollback loops during repair"));
        }

        // 3. Heuristic Anti-Poisoning Gate
        let guard = AntiPoisoningGuard::new();
        for (idx, attempt) in report.attempts.iter().enumerate() {
            let integrity = guard.inspect_patch(&attempt.proposed_patch);
            match integrity {
                RepairIntegrity::Suppressive => {
                    warn!(index = idx, "Memory Safety: Suppressive repair attempt detected. Structural bypass prohibited!");
                    return Err(anyhow!("Memory Safety: Suppressive repair attempt detected. Integrity audit failed."));
                }
                RepairIntegrity::Suspicious => {
                    warn!(index = idx, "Memory Safety: Suspicious repair attempt detected. Structural bypass prohibited!");
                    return Err(anyhow!("Memory Safety: Suspicious repair attempt detected. Integrity audit failed."));
                }
                RepairIntegrity::Structural => {
                    info!(index = idx, "Attempt passed anti-poisoning integrity checks.");
                }
            }
        }

        // 4. Identify the Fingerprint based on the initial failure
        let fingerprint = match report.initial_failure.kind {
            FailureKind::TypeScript => StrategyFingerprint::TypeScriptStructuralExtension,
            FailureKind::Dependency => StrategyFingerprint::MissingDependencyResolution,
            _ => StrategyFingerprint::RuntimeStabilityFix,
        };

        // 5. Construct the strategy as Experimental
        let mut strategy = EngineeringStrategy::new(fingerprint.as_str().to_string());
        strategy.state = StrategyState::Experimental;

        // 6. Map Verification Surface Coverage based on Truth Layer traces
        let mut has_e2e = false;
        let mut has_runtime = false;
        
        if let Some(meta) = report.initial_failure.metadata.get("verification_steps") {
            let steps = meta.to_lowercase();
            if steps.contains("playwright") || steps.contains("e2e") {
                has_e2e = true;
            }
            if steps.contains("boot") || steps.contains("runtime") {
                has_runtime = true;
            }
        }

        strategy.verification_surface_coverage = if has_e2e {
            VerificationSurfaceCoverage::FullRealityVerified
        } else if has_runtime {
            VerificationSurfaceCoverage::RuntimeVerified
        } else {
            VerificationSurfaceCoverage::BuildOnly
        };

        // 7. Distill steps from attempts
        for attempt in &report.attempts {
            strategy.steps.push(attempt.proposed_patch.clone());
        }

        // 8. Capture architectural context
        strategy.architectural_context = Some(format!(
            "Applies to {} symbols across {} files. Verification coverage: {:?}",
            report.attempts.last().map(|a| a.retrieved_context_ids.len()).unwrap_or(0),
            report.initial_failure.metadata.get("affected_files_count").unwrap_or(&"0".to_string()),
            strategy.verification_surface_coverage
        ));

        info!(
            id = %strategy.id, 
            pattern = %strategy.pattern_name, 
            coverage = ?strategy.verification_surface_coverage, 
            "Successfully generalized repair trace into Experimental strategy."
        );
        Ok(strategy)
    }
}

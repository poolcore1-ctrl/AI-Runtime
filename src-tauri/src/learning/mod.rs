pub mod abstraction;
pub mod strategies;
pub mod fingerprints;
pub mod confidence;
pub mod retrieval;
pub mod evolution;
pub mod anti_poisoning;

use std::sync::Arc;
use crate::storage::SharedStorage;
use crate::learning::retrieval::StrategyStore;
use crate::learning::abstraction::AbstractionEngine;
use crate::learning::evolution::KnowledgeEvolver;

pub struct LearningEngine {
    pub store: Arc<StrategyStore>,
    pub abstraction: Arc<AbstractionEngine>,
    pub evolver: Arc<KnowledgeEvolver>,
}

impl LearningEngine {
    pub fn new(storage: SharedStorage) -> Self {
        Self {
            store: Arc::new(StrategyStore::new(storage)),
            abstraction: Arc::new(AbstractionEngine::new()),
            evolver: Arc::new(KnowledgeEvolver::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use crate::learning::strategies::{EngineeringStrategy, StrategyState, VerificationSurfaceCoverage};
    use crate::learning::anti_poisoning::{AntiPoisoningGuard, RepairIntegrity};
    use crate::runtime::reports::{RepairTraceReport, RepairOutcome, RepairAttempt};
    use crate::runtime::errors::{FailureFingerprint, FailureKind};
    use std::collections::HashMap;

    #[test]
    fn test_anti_poisoning_guard_heuristics() {
        let guard = AntiPoisoningGuard::new();

        // 1. Structural, clean diff
        let clean_patch = "
Index: src/main.rs
===================================================================
--- src/main.rs
+++ src/main.rs
@@ -10,3 +10,3 @@
-let x = 10;
+let x = 20;
";
        assert_eq!(guard.inspect_patch(clean_patch), RepairIntegrity::Structural);

        // 2. Suppressive: Deleting tests
        let test_delete_patch = "
--- src/test.rs
+++ src/test.rs
@@ -10,3 +10,3 @@
-describe('TaskTracker', () => {
-  test('it persists tasks', () => {
-    expect(tracker.size()).toBe(1);
";
        assert_eq!(guard.inspect_patch(test_delete_patch), RepairIntegrity::Suppressive);

        // 3. Suppressive: Auth bypass
        let auth_bypass_patch = "
--- src/auth.rs
+++ src/auth.rs
@@ -5,3 +5,3 @@
-if !is_authorized {
-  return Err(Unauthorized);
+if true || !is_authorized {
+  return Ok(Authorized);
";
        assert_eq!(guard.inspect_patch(auth_bypass_patch), RepairIntegrity::Suppressive);

        // 4. Suspicious: type weakening and commenting logic
        let suspicious_patch = "
--- src/product.ts
+++ src/product.ts
@@ -5,3 +5,3 @@
-function check(product: Product) {
+function check(product: any) {
+  // product.validate();
";
        assert_eq!(guard.inspect_patch(suspicious_patch), RepairIntegrity::Suspicious);
    }

    #[test]
    fn test_safe_learning_abstraction_gate() {
        let abstraction = AbstractionEngine::new();

        // 1. Setup a clean successful trace report
        let clean_report = RepairTraceReport {
            session_id: "session-abc".to_string(),
            initial_failure: FailureFingerprint {
                kind: FailureKind::TypeScript,
                code: None,
                message: "TS compile issue".to_string(),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("verification_steps".to_string(), "tsc build, boot testing".to_string());
                    m
                },
            },
            attempts: vec![RepairAttempt {
                attempt_number: 1,
                retrieved_context_ids: vec!["lib_y".to_string()],
                proposed_patch: "--- src/lib.rs\n+++ src/lib.rs\n+let y = 10;".to_string(),
                environment_mutations: Vec::new(),
                strategy_reuse_source: None,
                adaptation_delta: None,
                reuse_confidence: None,
                verification_passed: true,
                new_failure: None,
            }],
            final_outcome: RepairOutcome::Success,
            total_duration_ms: 100,
            coordination_metrics: None,
        };

        let strategy_res = abstraction.abstract_trace(&clean_report);
        assert!(strategy_res.is_ok());
        let strategy = strategy_res.unwrap();
        assert_eq!(strategy.state, StrategyState::Experimental);
        assert_eq!(strategy.verification_surface_coverage, VerificationSurfaceCoverage::BuildAndRuntime);

        // 2. Setup a suppressive trace report
        let mut suppressive_report = clean_report.clone();
        suppressive_report.attempts = vec![RepairAttempt {
            attempt_number: 1,
            retrieved_context_ids: vec!["auth_bypass".to_string()],
            proposed_patch: "--- src/auth.rs\n+++ src/auth.rs\n+if true || bypass_auth {}".to_string(),
            environment_mutations: Vec::new(),
            strategy_reuse_source: None,
            adaptation_delta: None,
            reuse_confidence: None,
            verification_passed: true,
            new_failure: None,
        }];

        let suppressive_res = abstraction.abstract_trace(&suppressive_report);
        assert!(suppressive_res.is_err());
        assert!(suppressive_res.unwrap_err().to_string().contains("Memory Safety"));
    }

    #[test]
    fn test_strategy_decay_quarantine_and_promotion_flow() {
        let storage = Arc::new(Storage::new(":memory:").unwrap());
        let store = StrategyStore::new(storage);

        // 1. Initial State: Save newly created Experimental strategy
        let mut strategy = EngineeringStrategy::new("typescript_structural_extension".to_string());
        strategy.state = StrategyState::Experimental;
        strategy.confidence.success_rate = 1.0;
        strategy.confidence.consecutive_failures = 0;
        strategy.confidence.application_count = 1;
        assert_eq!(strategy.state, StrategyState::Experimental);

        store.save(&strategy).unwrap();

        // 2. Verify dynamic exponential decay
        let retrieved = store.find_by_pattern("typescript_structural_extension").unwrap();
        assert_eq!(retrieved.len(), 1);
        let mut strategy_retrieved = retrieved[0].clone();
        
        // Decay with elapsed days since last_decay
        let current_time = strategy_retrieved.confidence.last_decay_timestamp + 10 * 86400; // 10 days later
        strategy_retrieved.confidence.decay(current_time, 0.05); // lambda = 0.05
        assert!(strategy_retrieved.confidence.success_rate < 0.70); // Decayed from 1.0 down to ~0.60
        
        // Save back
        store.save(&strategy_retrieved).unwrap();

        // 3. Verify Active State promotion after 3 successful reuses with success >= 90%
        let mut prom_strategy = EngineeringStrategy::new("typescript_structural_extension".to_string());
        prom_strategy.state = StrategyState::Experimental;
        prom_strategy.confidence.application_count = 3;
        prom_strategy.confidence.success_rate = 0.95;
        store.save(&prom_strategy).unwrap();

        // Querying will trigger promotion evaluation hook in retrieval.rs
        let retrieved_prom = store.find_by_pattern("typescript_structural_extension").unwrap();
        let promoted = retrieved_prom.iter().find(|s| s.id == prom_strategy.id).unwrap();
        assert_eq!(promoted.state, StrategyState::Active);

        // 4. Verify Quarantine isolation trigger on failure count
        let mut failing_strategy = EngineeringStrategy::new("typescript_structural_extension".to_string());
        failing_strategy.state = StrategyState::Active;
        failing_strategy.confidence.consecutive_failures = 4;
        failing_strategy.confidence.success_rate = 0.35;
        store.save(&failing_strategy).unwrap();

        // Finding active strategies will trigger quarantine decay and exclude the quarantined strategy
        let retrieved_active = store.find_by_pattern("typescript_structural_extension").unwrap();
        
        // Assert that the quarantined strategy is filtered out of active query results
        let active_found = retrieved_active.iter().any(|s| s.id == failing_strategy.id);
        assert!(!active_found);
    }
}

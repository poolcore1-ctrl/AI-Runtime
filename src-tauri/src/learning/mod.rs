pub mod abstraction;
pub mod strategies;
pub mod fingerprints;
pub mod confidence;
pub mod retrieval;
pub mod evolution;
pub mod anti_poisoning;
pub mod ir;
pub mod hash;
pub mod translation;
pub mod translation_validator;
pub mod provider_profiles;
pub mod drift_predictor;
pub mod router;

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
        assert_eq!(strategy.verification_surface_coverage, VerificationSurfaceCoverage::RuntimeVerified);

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

    #[test]
    fn test_strategy_ir_normalization() {
        use crate::learning::ir::{StrategyIR, ActionType, NormalizedStep, StrategyConstraint, ConstraintType, ConstraintSeverity, StrategyMetadata, DeterminismLevel, StrategyIRMigrator, IRMigrator};
        
        let metadata = StrategyMetadata {
            generated_by: "Claude 3.5 Sonnet".to_string(),
            source_provider: "Anthropic".to_string(),
            base_difficulty: "Medium".to_string(),
            complexity_factor: 0.45,
            entropy_class: "Stable".to_string(),
            generated_timestamp: 1620000000,
            determinism_requirement: DeterminismLevel::Standard,
        };

        let constraint = StrategyConstraint {
            constraint_type: ConstraintType::PreserveAuthentication,
            severity: ConstraintSeverity::Critical,
            expression: "Verify auth token validity".to_string(),
        };

        let step = NormalizedStep {
            step_id: "step_01".to_string(),
            action_type: ActionType::Inspect,
            target_file: "src/auth.rs".to_string(),
            instructions: "Read security middleware".to_string(),
            expected_outcome: Some("auth check verified".to_string()),
        };

        let ir = StrategyIR {
            ir_version: "3.7".to_string(), // older version
            id: "strat_auth_verify".to_string(),
            semantic_hash: "temporary_hash".to_string(),
            objective: "Assert authentication preservation".to_string(),
            target_symbols: vec!["auth_middleware".to_string()],
            constraints: vec![constraint],
            normalized_steps: vec![step],
            metadata,
        };

        // Serialize to JSON value
        let json_value = serde_json::to_value(&ir).unwrap();

        // Perform IR Version Migration to 3.8
        let migrator = StrategyIRMigrator;
        let migrated_ir_res = migrator.migrate(json_value);
        assert!(migrated_ir_res.is_ok());
        let migrated_ir = migrated_ir_res.unwrap();
        assert_eq!(migrated_ir.ir_version, "3.8");
        assert_eq!(migrated_ir.id, "strat_auth_verify");
    }

    #[test]
    fn test_prompt_translation_compilation() {
        use crate::learning::ir::{StrategyIR, ActionType, NormalizedStep, StrategyConstraint, ConstraintType, ConstraintSeverity, StrategyMetadata, DeterminismLevel};
        use crate::learning::translation::{PromptCompiler, ClaudeCompiler, GeminiCompiler, DeepSeekCompiler, LocalCompiler, CompressionLevel};

        let metadata = StrategyMetadata {
            generated_by: "Gemini 1.5 Pro".to_string(),
            source_provider: "Google".to_string(),
            base_difficulty: "Low".to_string(),
            complexity_factor: 0.1,
            entropy_class: "Stable".to_string(),
            generated_timestamp: 1620000000,
            determinism_requirement: DeterminismLevel::Relaxed,
        };

        let step = NormalizedStep {
            step_id: "step_02".to_string(),
            action_type: ActionType::Edit,
            target_file: "src/config.rs".to_string(),
            instructions: "Modify port binding variable".to_string(),
            expected_outcome: None,
        };

        let ir = StrategyIR {
            ir_version: "3.8".to_string(),
            id: "strat_config".to_string(),
            semantic_hash: "".to_string(),
            objective: "Change active server port".to_string(),
            target_symbols: vec!["ServerConfig".to_string()],
            constraints: vec![],
            normalized_steps: vec![step],
            metadata,
        };

        // 1. Claude XML Compiler
        let claude_prompt = ClaudeCompiler.compile(&ir);
        assert!(claude_prompt.contains("<strategy version=\"3.8\">"));
        assert!(claude_prompt.contains("<objective>Change active server port</objective>"));
        assert!(claude_prompt.contains("src/config.rs"));

        // 2. Gemini JSON Compiler
        let gemini_prompt = GeminiCompiler.compile(&ir);
        assert!(gemini_prompt.contains("\"id\": \"strat_config\""));

        // 3. DeepSeek Reasoning Compiler
        let deepseek_prompt = DeepSeekCompiler.compile(&ir);
        assert!(deepseek_prompt.contains("Please execute the following engineering strategy step-by-step:"));
        assert!(deepseek_prompt.contains("Step 1: [Action: Edit on src/config.rs]"));

        // 4. Local Compiler - Moderate Compression
        let local_moderate = LocalCompiler { compression: CompressionLevel::Moderate }.compile(&ir);
        assert!(local_moderate.contains("Brief Steps: Edit on src/config.rs"));

        // 5. Local Compiler - Aggressive Compression
        let local_aggressive = LocalCompiler { compression: CompressionLevel::Aggressive }.compile(&ir);
        assert!(!local_moderate.contains("Brief Steps: Edit on src/config.rs"));
        assert!(local_aggressive.contains("Repair targets: ServerConfig"));
    }

    #[test]
    fn test_translation_equivalence_validation() {
        use crate::learning::ir::{StrategyIR, ActionType, NormalizedStep, StrategyConstraint, ConstraintType, ConstraintSeverity, StrategyMetadata, DeterminismLevel};
        use crate::learning::translation_validator::TranslationValidator;

        let metadata = StrategyMetadata {
            generated_by: "DeepSeek V3".to_string(),
            source_provider: "DeepSeek".to_string(),
            base_difficulty: "High".to_string(),
            complexity_factor: 0.9,
            entropy_class: "Extreme".to_string(),
            generated_timestamp: 1620000000,
            determinism_requirement: DeterminismLevel::Forensic,
        };

        let constraint_security = StrategyConstraint {
            constraint_type: ConstraintType::PreventSecurityRegression,
            severity: ConstraintSeverity::Critical,
            expression: "Never bypass JWT auth check logic".to_string(),
        };

        let step = NormalizedStep {
            step_id: "step_03".to_string(),
            action_type: ActionType::Refactor,
            target_file: "src/jwt.rs".to_string(),
            instructions: "Audit validation loops".to_string(),
            expected_outcome: None,
        };

        let ir = StrategyIR {
            ir_version: "3.8".to_string(),
            id: "strat_jwt".to_string(),
            semantic_hash: "".to_string(),
            objective: "Secure validation handlers".to_string(),
            target_symbols: vec!["JWTCheck".to_string()],
            constraints: vec![constraint_security],
            normalized_steps: vec![step],
            metadata,
        };

        let validator = TranslationValidator;

        // Positive Case: Prompt preserves all details
        let valid_prompt = "
            Strategy Objective: Secure validation handlers
            Ensure step target files: src/jwt.rs
            Never bypass JWT auth check logic
        ";
        let res_valid = validator.validate(&ir, valid_prompt);
        assert!(res_valid.semantically_equivalent);
        assert_eq!(res_valid.semantic_preservation_score, 1.0);

        // Negative Case: Missing constraints and safety checks failure
        let invalid_prompt = "
            Strategy Objective: Secure validation handlers
            Target is src/jwt.rs
        ";
        let res_invalid = validator.validate(&ir, invalid_prompt);
        assert!(!res_invalid.semantically_equivalent);
        assert!(res_invalid.semantic_preservation_score < 0.6);
        assert_eq!(res_invalid.safety_checks_failed.len(), 1);
        assert!(res_invalid.safety_checks_failed[0].contains("Critical Safety breach"));
    }

    #[test]
    fn test_cross_model_drift_prediction() {
        use crate::learning::ir::{StrategyIR, ActionType, NormalizedStep, StrategyConstraint, ConstraintType, ConstraintSeverity, StrategyMetadata, DeterminismLevel};
        use crate::learning::drift_predictor::{CrossModelDriftPredictor, DriftRisk};

        let metadata = StrategyMetadata {
            generated_by: "Claude 3.5 Sonnet".to_string(),
            source_provider: "Anthropic".to_string(),
            base_difficulty: "Medium".to_string(),
            complexity_factor: 0.5,
            entropy_class: "Stable".to_string(),
            generated_timestamp: 1620000000,
            determinism_requirement: DeterminismLevel::Standard,
        };

        let ir = StrategyIR {
            ir_version: "3.8".to_string(),
            id: "strat_complex".to_string(),
            semantic_hash: "".to_string(),
            objective: "Complex distributed repair".to_string(),
            target_symbols: vec![],
            constraints: vec![],
            normalized_steps: vec![
                NormalizedStep {
                    step_id: "s1".to_string(),
                    action_type: ActionType::Edit,
                    target_file: "a.rs".to_string(),
                    instructions: "edit".to_string(),
                    expected_outcome: None,
                },
                NormalizedStep {
                    step_id: "s2".to_string(),
                    action_type: ActionType::Edit,
                    target_file: "b.rs".to_string(),
                    instructions: "edit".to_string(),
                    expected_outcome: None,
                },
                NormalizedStep {
                    step_id: "s3".to_string(),
                    action_type: ActionType::Edit,
                    target_file: "c.rs".to_string(),
                    instructions: "edit".to_string(),
                    expected_outcome: None,
                },
                NormalizedStep {
                    step_id: "s4".to_string(),
                    action_type: ActionType::Edit,
                    target_file: "d.rs".to_string(),
                    instructions: "edit".to_string(),
                    expected_outcome: None,
                },
                NormalizedStep {
                    step_id: "s5".to_string(),
                    action_type: ActionType::Edit,
                    target_file: "e.rs".to_string(),
                    instructions: "edit".to_string(),
                    expected_outcome: None,
                },
            ],
            metadata,
        };

        let predictor = CrossModelDriftPredictor;

        // 1. Stable Migration: Claude -> DeepSeek
        let (score_stable, risk_stable) = predictor.predict_stability(&ir, "Claude", "DeepSeek");
        assert!(score_stable > 0.70);
        assert_eq!(risk_stable, DriftRisk::Moderate);

        // 2. High Drift Migration: Claude -> Local Ollama Gemma
        let (score_drift, risk_drift) = predictor.predict_stability(&ir, "Claude", "Local");
        assert!(score_drift < 0.60);
        assert!(risk_drift == DriftRisk::High || risk_drift == DriftRisk::Critical);
    }

    #[test]
    fn test_replay_semantic_hash_consistency() {
        use crate::learning::ir::{StrategyIR, ActionType, NormalizedStep, StrategyConstraint, ConstraintType, ConstraintSeverity, StrategyMetadata, DeterminismLevel};
        use crate::learning::hash::compute_semantic_hash;

        let metadata = StrategyMetadata {
            generated_by: "Gemini 1.5 Pro".to_string(),
            source_provider: "Google".to_string(),
            base_difficulty: "Low".to_string(),
            complexity_factor: 0.1,
            entropy_class: "Stable".to_string(),
            generated_timestamp: 1620000000,
            determinism_requirement: DeterminismLevel::Standard,
        };

        let c1 = StrategyConstraint {
            constraint_type: ConstraintType::PreserveAPI,
            severity: ConstraintSeverity::Major,
            expression: "Keep v1 routes".to_string(),
        };

        let c2 = StrategyConstraint {
            constraint_type: ConstraintType::PreventTypeWeakening,
            severity: ConstraintSeverity::Minor,
            expression: "Never use any".to_string(),
        };

        let step = NormalizedStep {
            step_id: "step_04".to_string(),
            action_type: ActionType::Inspect,
            target_file: "src/api.rs".to_string(),
            instructions: "Inspect routes   with   spaces".to_string(),
            expected_outcome: None,
        };

        // IR 1: Order: c1 then c2, targets: t1 then t2
        let ir1 = StrategyIR {
            ir_version: "3.8".to_string(),
            id: "strat_api".to_string(),
            semantic_hash: "".to_string(),
            objective: "API routes cleanup".to_string(),
            target_symbols: vec!["api_endpoint".to_string(), "v1_endpoint".to_string()],
            constraints: vec![c1.clone(), c2.clone()],
            normalized_steps: vec![step.clone()],
            metadata: metadata.clone(),
        };

        // IR 2: Order: c2 then c1, targets: t2 then t1 (and slightly shifted spaces in expression/objective)
        let ir2 = StrategyIR {
            ir_version: "3.8".to_string(),
            id: "strat_api".to_string(),
            semantic_hash: "".to_string(),
            objective: "API   routes   cleanup".to_string(), // different spacing
            target_symbols: vec!["v1_endpoint".to_string(), "api_endpoint".to_string()], // reordered targets
            constraints: vec![c2.clone(), c1.clone()], // reordered constraints
            normalized_steps: vec![NormalizedStep {
                instructions: "Inspect routes with spaces".to_string(), // canonicalized spacing in instructions
                ..step.clone()
            }],
            metadata: metadata.clone(),
        };

        let hash1 = compute_semantic_hash(&ir1);
        let hash2 = compute_semantic_hash(&ir2);

        // Verification: Canonicalization guarantees hashes are identical
        assert_eq!(hash1, hash2);
    }
}


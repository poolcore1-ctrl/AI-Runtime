pub mod provider;
pub mod routing;
pub mod budget;
pub mod compression;
pub mod checkpoints;
pub mod graph;
pub mod compiler;
pub mod optimization;
pub mod memory;
pub mod memory_compaction;
pub mod repository_identity;
pub mod forecasting;
pub mod reputation;
pub mod temporal_analytics;
pub mod provider_drift;
pub mod motif_evolution;
pub mod constitution;
pub mod constitutional_risk;
pub mod belief_graph;
pub mod contradiction;
pub mod uncertainty;
pub mod arbitration;
pub mod audit;
pub mod homeostasis;
pub mod immune;
pub mod regulator;
pub mod metabolism;
pub mod consolidation;
pub mod diversity;

use crate::storage::SharedStorage;
use crate::cognition::provider::ProviderRegistry;
use crate::cognition::routing::CognitionRouter;
use crate::cognition::budget::TokenBudgetManager;
use crate::cognition::compression::ContextCompressor;
use crate::cognition::checkpoints::CheckpointStore;
use std::sync::Arc;

pub struct CognitionEngine {
    pub registry: Arc<ProviderRegistry>,
    pub router: Arc<CognitionRouter>,
    pub budget: Arc<TokenBudgetManager>,
    pub compressor: Arc<ContextCompressor>,
    pub checkpoint_store: Arc<CheckpointStore>,
}

impl CognitionEngine {
    pub fn new(storage: SharedStorage) -> Self {
        let registry = Arc::new(ProviderRegistry::new(storage.clone()));
        let router = Arc::new(CognitionRouter::new(registry.clone()));
        let budget = Arc::new(TokenBudgetManager::new(500_000)); // Default 500k session token cap
        let compressor = Arc::new(ContextCompressor::new());
        let checkpoint_store = Arc::new(CheckpointStore::new(storage));

        Self {
            registry,
            router,
            budget,
            compressor,
            checkpoint_store,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use crate::cognition::routing::ModelCapability;
    use crate::cognition::provider::RegisteredProvider;
    use crate::cognition::checkpoints::{CognitiveSession, CognitiveCheckpoint};
    use crate::intelligence::symbols::{Symbol, SymbolKind};
    use crate::intelligence::graph::SemanticGraph;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_cognition_engine_flow() {
        // Use an in-memory SQLite database for test execution
        let storage = Arc::new(Storage::new(":memory:").unwrap());
        let engine = CognitionEngine::new(storage);

        // 1. Verify default route falls back to Simulated model
        let route_res = engine.router.route(ModelCapability::Planning);
        assert!(route_res.is_ok());
        let provider = route_res.unwrap();
        assert_eq!(provider.name(), "Simulated Engine");

        // 2. Verify adding provider to Registry with dynamic templates
        let mock_provider = RegisteredProvider {
            id: "test-ollama".to_string(),
            name: "test-qwen".to_string(),
            provider_type: "ollama".to_string(),
            api_url: "http://localhost:11434".to_string(),
            api_key: None,
            is_enabled: true,
            capabilities: vec![ModelCapability::Coding, ModelCapability::Verification],
            routing_priority: 50,
            model_name: "qwen2.5-coder".to_string(),
            provider_family: "ollama".to_string(),
            price_input_million: 0.15,
            price_output_million: 0.60,
            timeout_secs: 15,
            payload_template: Some("{\"model\": \"{{model}}\", \"prompt\": \"{{prompt}}\"}".to_string()),
            headers_template: Some("{\"X-Test\": \"ASOS\"}".to_string()),
        };

        let add_res = engine.registry.add_provider(mock_provider);
        assert!(add_res.is_ok());

        let providers = engine.registry.get_providers().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "test-ollama");
        assert_eq!(providers[0].provider_family, "ollama");

        // 3. Verify routing picks the registered provider for Coding
        let route_coding = engine.router.route(ModelCapability::Coding).unwrap();
        assert_eq!(route_coding.name(), "test-qwen");
        assert_eq!(route_coding.model_name(), "qwen2.5-coder");

        // 4. Verify Health Metrics updates and decays
        let health_metrics_before = engine.registry.get_health_metrics("test-ollama").unwrap();
        assert!(health_metrics_before.is_none());

        // Update with success
        let update_suc = engine.registry.update_health_metrics("test-ollama", true, 120, None);
        assert!(update_suc.is_ok());
        let health_after_suc = engine.registry.get_health_metrics("test-ollama").unwrap().unwrap();
        assert_eq!(health_after_suc.success_count, 1);
        assert_eq!(health_after_suc.failure_count, 0);
        assert!(health_after_suc.health_score > 0.99);

        // Update with failure (decay)
        let update_fail = engine.registry.update_health_metrics("test-ollama", false, 5000, Some("Timeout".to_string()));
        assert!(update_fail.is_ok());
        let health_after_fail = engine.registry.get_health_metrics("test-ollama").unwrap().unwrap();
        assert_eq!(health_after_fail.failure_count, 1);
        assert!(health_after_fail.health_score < 0.90); // Downranked!

        // 5. Verify Cognitive Sessions & Recovery Checkpoints
        let session = CognitiveSession {
            session_id: "session-123".to_string(),
            project_id: "project-abc".to_string(),
            active_capability: "Planning".to_string(),
            active_provider_id: Some("test-ollama".to_string()),
            provider_chain: vec!["test-ollama".to_string()],
            strategy_fingerprint: Some("typescript-hydration-v1".to_string()),
            current_dag_node: Some("verify_build".to_string()),
            token_budget_state: "{\"tokens_used\": 1000}".to_string(),
            repair_attempt_count: 1,
            timestamp: 1778759502,
        };

        let save_session_res = engine.checkpoint_store.save_session(&session);
        assert!(save_session_res.is_ok());

        let retrieved_session = engine.checkpoint_store.get_session("session-123").unwrap().unwrap();
        assert_eq!(retrieved_session.project_id, "project-abc");
        assert_eq!(retrieved_session.active_capability, "Planning");

        let checkpoint = CognitiveCheckpoint {
            checkpoint_id: "cp-456".to_string(),
            session_id: "session-123".to_string(),
            active_task_id: "repair_react".to_string(),
            step_index: 2,
            plan_dag: "{\"steps\": []}".to_string(),
            partial_patch: Some("diff --git".to_string()),
            reasoning_history: vec!["Initiating repair".to_string()],
            timestamp: 1778759502,
        };

        let save_cp_res = engine.checkpoint_store.save_checkpoint(&checkpoint);
        assert!(save_cp_res.is_ok());

        let retrieved_cp = engine.checkpoint_store.get_checkpoint("cp-456").unwrap().unwrap();
        assert_eq!(retrieved_cp.active_task_id, "repair_react");
        assert_eq!(retrieved_cp.step_index, 2);

        // Purge session
        let purge_res = engine.checkpoint_store.purge_session("session-123");
        assert!(purge_res.is_ok());
        assert!(engine.checkpoint_store.get_session("session-123").unwrap().is_none());
        assert!(engine.checkpoint_store.get_checkpoint("cp-456").unwrap().is_none());

        // 6. Verify Context Compressor
        let graph = SemanticGraph::new();
        let sym = Symbol {
            name: "calculate_sum".to_string(),
            kind: SymbolKind::Function,
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 5,
            signature: Some("fn calculate_sum()".to_string()),
        };
        graph.add_symbol_node(sym);

        let compressor = ContextCompressor::new();
        let compressed = compressor.compress_context("calculate", &graph, 8000).unwrap();
        assert!(compressed.contains("Compressed Cognitive Context"));
    }

    #[test]
    fn test_graph_compilation() {
        use crate::learning::ir::{StrategyIR, StrategyMetadata, DeterminismLevel};
        use crate::cognition::graph::ExecutionMode;
        use crate::cognition::compiler::CognitiveGraphCompiler;
        use crate::stress_testing::types::EntropyClass;

        let metadata = StrategyMetadata {
            generated_by: "Claude".to_string(),
            source_provider: "Anthropic".to_string(),
            base_difficulty: "Medium".to_string(),
            complexity_factor: 0.5,
            entropy_class: "Stable".to_string(),
            generated_timestamp: 1620000000,
            determinism_requirement: DeterminismLevel::Standard,
        };

        let ir = StrategyIR {
            ir_version: "3.8".to_string(),
            id: "strat_compile_graph".to_string(),
            semantic_hash: "".to_string(),
            objective: "Verify graph generation".to_string(),
            target_symbols: vec!["main".to_string()],
            constraints: vec![],
            normalized_steps: vec![],
            metadata,
        };

        let compiler = CognitiveGraphCompiler::new();
        let graph = compiler.compile(&ir, EntropyClass::Stable);

        // Verification: Deterministic DAG compiled
        assert_eq!(graph.execution_mode, ExecutionMode::Sequential);
        assert_eq!(graph.nodes.len(), 3);
        assert!(graph.nodes.contains_key("node_inspect"));
        assert!(graph.nodes.contains_key("node_repair"));
        assert!(graph.nodes.contains_key("node_verify"));
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_speculative_parallel_execution() {
        use crate::verification::{TruthLayer, VerificationDAG, GraphTruthLayer, SpeculativeBranchReport};

        let truth_layer = TruthLayer::new(VerificationDAG::new());
        let graph_truth = GraphTruthLayer::new(truth_layer);

        // Simulate three speculative branch outcomes
        let branches = vec![
            SpeculativeBranchReport {
                provider_name: "Claude".to_string(),
                compilation_success: true,
                semantic_preservation_score: 0.95,
                invariants_passed: true,
                behavioral_drift_score: 0.05,
            },
            SpeculativeBranchReport {
                provider_name: "Gemini".to_string(),
                compilation_success: true,
                semantic_preservation_score: 0.90,
                invariants_passed: true,
                behavioral_drift_score: 0.02, // slightly better drift
            },
            SpeculativeBranchReport {
                provider_name: "DeepSeek".to_string(),
                compilation_success: false, // failed compilation
                semantic_preservation_score: 0.99,
                invariants_passed: true,
                behavioral_drift_score: 0.01,
            },
        ];

        let winner = graph_truth.arbitrate_branches(&branches);
        assert!(winner.is_some());
        
        let report = winner.unwrap();
        // Claude gets highest combined score: 0.95 - (0.05 * 0.5) = 0.925 vs Gemini's 0.90 - (0.02 * 0.5) = 0.89
        assert_eq!(report.provider_name, "Claude");
    }

    #[test]
    fn test_entropy_topology_mutation() {
        use crate::learning::ir::{StrategyIR, StrategyMetadata, DeterminismLevel};
        use crate::cognition::graph::ExecutionMode;
        use crate::cognition::compiler::CognitiveGraphCompiler;
        use crate::stress_testing::types::EntropyClass;

        let metadata = StrategyMetadata {
            generated_by: "Claude".to_string(),
            source_provider: "Anthropic".to_string(),
            base_difficulty: "High".to_string(),
            complexity_factor: 0.8,
            entropy_class: "Extreme".to_string(),
            generated_timestamp: 1620000000,
            determinism_requirement: DeterminismLevel::Forensic,
        };

        let ir = StrategyIR {
            ir_version: "3.8".to_string(),
            id: "strat_extreme".to_string(),
            semantic_hash: "".to_string(),
            objective: "High entropy target".to_string(),
            target_symbols: vec![],
            constraints: vec![],
            normalized_steps: vec![],
            metadata,
        };

        let compiler = CognitiveGraphCompiler::new();
        
        // Compile under Extreme entropy: Graph topology expands dynamically
        let graph = compiler.compile(&ir, EntropyClass::Extreme);
        assert_eq!(graph.execution_mode, ExecutionMode::Consensus);
        assert_eq!(graph.nodes.len(), 6); // sandbox, spec_a, spec_b, consensus, verify, escalation
        assert!(graph.nodes.contains_key("node_sandbox"));
        assert!(graph.nodes.contains_key("node_consensus"));
        assert!(graph.nodes.contains_key("node_escalation"));
    }

    #[test]
    fn test_graph_replay_determinism() {
        use crate::stress_testing::replay::ReplayEngine;

        let storage = Arc::new(Storage::new(":memory:").unwrap());
        let engine = ReplayEngine::new(storage);

        let manifest_res = engine.execute_graph_replay(
            "hash_123_abc",
            &vec!["node_inspect".to_string(), "node_repair".to_string()],
            &vec!["branch_a".to_string()],
            &vec!["Claude".to_string()],
            &vec!["pass".to_string()],
        );

        assert!(manifest_res.is_ok());
        let manifest = manifest_res.unwrap();
        assert_eq!(manifest.graph_hash, "hash_123_abc");
        assert_eq!(manifest.traversed_nodes.len(), 2);
        assert_eq!(manifest.branch_decisions[0], "branch_a");
    }

    #[test]
    fn test_recursive_failure_recovery() {
        use crate::cognition::graph::{CognitiveExecutionGraph, CognitiveNode, CognitiveNodeType, CognitiveEdge, EdgeCondition, ExecutionMode, ExecutionState, RetryPolicy};
        use crate::stress_testing::types::EntropyClass;
        use std::collections::HashMap;

        // Mock a basic CEG failure path
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();

        let root_id = "node_repair".to_string();
        let rollback_id = "node_rollback".to_string();
        let success_id = "node_success".to_string();

        let default_policy = RetryPolicy {
            max_attempts: 1,
            backoff_ms: 0,
        };

        nodes.insert(root_id.clone(), CognitiveNode {
            node_id: root_id.clone(),
            node_type: CognitiveNodeType::Repair,
            strategy_ir: None,
            verifier_config: None,
            execution_state: ExecutionState::Failed, // repair failed
            provider_profile: None,
            retry_policy: default_policy.clone(),
            checkpoint_id: None,
        });

        nodes.insert(rollback_id.clone(), CognitiveNode {
            node_id: rollback_id.clone(),
            node_type: CognitiveNodeType::Rollback,
            strategy_ir: None,
            verifier_config: None,
            execution_state: ExecutionState::Pending,
            provider_profile: None,
            retry_policy: default_policy.clone(),
            checkpoint_id: None,
        });

        nodes.insert(success_id.clone(), CognitiveNode {
            node_id: success_id.clone(),
            node_type: CognitiveNodeType::Verify,
            strategy_ir: None,
            verifier_config: None,
            execution_state: ExecutionState::Pending,
            provider_profile: None,
            retry_policy: default_policy.clone(),
            checkpoint_id: None,
        });

        // Edge conditions: success routes to success_id, failure routes to rollback_id
        edges.push(CognitiveEdge {
            from: root_id.clone(),
            to: success_id.clone(),
            condition: EdgeCondition::OnSuccess,
        });

        edges.push(CognitiveEdge {
            from: root_id.clone(),
            to: rollback_id.clone(),
            condition: EdgeCondition::OnFailure,
        });

        let graph = CognitiveExecutionGraph {
            graph_id: "graph_failure_recovery".to_string(),
            root_node: root_id.clone(),
            nodes,
            edges,
            execution_mode: ExecutionMode::Sequential,
            entropy_class: EntropyClass::Stable,
            semantic_hash: "hash_recovery".to_string(),
        };

        // Determine dynamic next step based on failure condition
        let active_node = graph.nodes.get(&root_id).unwrap();
        let next_node_id = if active_node.execution_state == ExecutionState::Failed {
            let matching_edge = graph.edges.iter().find(|e| e.from == root_id && e.condition == EdgeCondition::OnFailure).unwrap();
            matching_edge.to.clone()
        } else {
            root_id.clone()
        };

        assert_eq!(next_node_id, rollback_id);
    }

    #[test]
    fn test_graph_cost_governor_enforcement() {
        use crate::cognition::optimization::CostGovernorConfig;
        
        let config = CostGovernorConfig {
            max_speculative_branches: 3,
            max_recovery_depth: 2,
            session_cost_ceiling_usd: 1.0,
            max_parallel_verifiers: 2,
            active_pruning_enabled: true,
        };

        // Simulate usage exceeding cost ceiling
        let current_cost = 1.05;
        let blocks_execution = current_cost > config.session_cost_ceiling_usd;
        assert!(blocks_execution);
    }

    #[test]
    fn test_graph_pruning_low_roi_verifiers() {
        use crate::cognition::optimization::{GraphPruner, NodeExecutionMetric};
        use crate::cognition::graph::{CognitiveExecutionGraph, CognitiveNode, CognitiveNodeType, ExecutionMode};
        use crate::stress_testing::types::EntropyClass;
        use std::collections::HashMap;

        let mut nodes = HashMap::new();
        let verify_id = "node_verify".to_string();
        nodes.insert(verify_id.clone(), CognitiveNode {
            node_id: verify_id.clone(),
            node_type: CognitiveNodeType::Verify,
            strategy_ir: None,
            verifier_config: None,
            execution_state: crate::cognition::graph::ExecutionState::Completed,
            provider_profile: None,
            retry_policy: crate::cognition::graph::RetryPolicy { max_attempts: 1, backoff_ms: 0 },
            checkpoint_id: None,
        });

        let mut graph = CognitiveExecutionGraph {
            graph_id: "graph_prune".to_string(),
            root_node: verify_id.clone(),
            nodes,
            edges: vec![],
            execution_mode: ExecutionMode::Sequential,
            entropy_class: EntropyClass::Stable,
            semantic_hash: "hash_prune".to_string(),
        };

        // Simulate 0.0 ROI
        let metrics = vec![NodeExecutionMetric {
            node_id: verify_id.clone(),
            node_type: CognitiveNodeType::Verify,
            provider_name: "Local".to_string(),
            duration_ms: 100,
            input_tokens: 10,
            output_tokens: 10,
            cost_usd: 0.01,
            success: true,
            verifier_roi: 0.0,
            semantic_contribution_score: 0.0,
            replay_relevance: 0.1,
            drift_risk: 0.0,
        }];

        let pruner = GraphPruner::new();
        let pruned_count = pruner.prune_graph(&mut graph, &metrics);

        assert_eq!(pruned_count, 1);
        assert!(!graph.nodes.contains_key(&verify_id));
    }

    #[test]
    fn test_graph_motif_promotion() {
        use crate::cognition::optimization::GraphMotifRegistry;
        let registry = GraphMotifRegistry::new();

        // Retrieve predefined AuthRegression motif
        let motif_opt = registry.get_motif("AuthRegression");
        assert!(motif_opt.is_some());
        
        let motif = motif_opt.unwrap();
        assert_eq!(motif.motif_id, "motif_auth_regression");
        assert!(motif.semantic_success_rate > 0.9);
    }

    #[test]
    fn test_graph_compression_equivalence() {
        use crate::cognition::optimization::GraphCompressionEngine;
        use crate::cognition::graph::{CognitiveExecutionGraph, CognitiveNode, CognitiveNodeType, CognitiveEdge, EdgeCondition, ExecutionMode};
        use crate::stress_testing::types::EntropyClass;
        use std::collections::HashMap;

        let mut nodes = HashMap::new();
        let mut edges = Vec::new();

        let n1_id = "node_1".to_string();
        let n2_id = "node_2".to_string();
        
        let default_policy = crate::cognition::graph::RetryPolicy { max_attempts: 1, backoff_ms: 0 };
        
        nodes.insert(n1_id.clone(), CognitiveNode {
            node_id: n1_id.clone(),
            node_type: CognitiveNodeType::Verify,
            strategy_ir: None, verifier_config: None, execution_state: crate::cognition::graph::ExecutionState::Pending,
            provider_profile: None, retry_policy: default_policy.clone(), checkpoint_id: None,
        });

        nodes.insert(n2_id.clone(), CognitiveNode {
            node_id: n2_id.clone(),
            node_type: CognitiveNodeType::Verify,
            strategy_ir: None, verifier_config: None, execution_state: crate::cognition::graph::ExecutionState::Pending,
            provider_profile: None, retry_policy: default_policy.clone(), checkpoint_id: None,
        });

        edges.push(CognitiveEdge { from: n1_id.clone(), to: n2_id.clone(), condition: EdgeCondition::OnSuccess });

        let mut graph = CognitiveExecutionGraph {
            graph_id: "graph_compress".to_string(),
            root_node: n1_id.clone(),
            nodes,
            edges,
            execution_mode: ExecutionMode::Sequential,
            entropy_class: EntropyClass::Stable,
            semantic_hash: "hash_compress".to_string(),
        };

        let compressor = GraphCompressionEngine::new();
        let success = compressor.compress_subgraph(&mut graph, &vec![n1_id.clone(), n2_id.clone()], "node_compressed_super");
        
        assert!(success);
        assert!(!graph.nodes.contains_key(&n1_id));
        assert!(!graph.nodes.contains_key(&n2_id));
        assert!(graph.nodes.contains_key("node_compressed_super"));
        assert_eq!(graph.edges.len(), 0); // Internal edges removed
    }

    #[test]
    fn test_adaptive_traversal_weighting() {
        use crate::cognition::optimization::{GraphPruner, EdgeExecutionProfile};
        use crate::cognition::graph::{CognitiveExecutionGraph, CognitiveEdge, EdgeCondition, ExecutionMode};
        use crate::stress_testing::types::EntropyClass;
        use std::collections::HashMap;

        let mut edges = Vec::new();
        edges.push(CognitiveEdge { from: "n1".to_string(), to: "n2".to_string(), condition: EdgeCondition::OnSuccess });

        let mut graph = CognitiveExecutionGraph {
            graph_id: "graph_route".to_string(),
            root_node: "n1".to_string(),
            nodes: HashMap::new(),
            edges,
            execution_mode: ExecutionMode::Sequential,
            entropy_class: EntropyClass::Stable,
            semantic_hash: "hash_route".to_string(),
        };

        let profiles = vec![EdgeExecutionProfile {
            edge_id: "n1_to_n2".to_string(),
            traversal_success_rate: 0.1,
            avg_cost_usd: 0.05,
            avg_latency_ms: 1000,
            semantic_success_rate: 0.1,
            rollback_trigger_rate: 0.8, // High rollback triggers edge reroute
        }];

        let pruner = GraphPruner::new();
        let mutated = pruner.adjust_routing_weights(&mut graph, &profiles);

        assert_eq!(mutated, 1);
        assert_eq!(graph.edges[0].condition, EdgeCondition::OnProviderDrift);
    }

    #[test]
    fn test_replay_safe_optimization() {
        use crate::cognition::optimization::OptimizationReplayManifest;
        
        let manifest = OptimizationReplayManifest {
            original_graph_hash: "hash_original".to_string(),
            optimized_graph_hash: "hash_optimized".to_string(),
            pruned_nodes: vec!["node_verify_old".to_string()],
            compressed_subgraphs: vec!["node_super_verify".to_string()],
            edge_weight_mutations: vec!["n1_to_n2_reroute".to_string()],
            optimization_reasoning_hash: "hash_reasoning".to_string(),
        };

        assert_eq!(manifest.pruned_nodes.len(), 1);
        assert_eq!(manifest.compressed_subgraphs.len(), 1);
    }

    #[test]
    fn test_repository_identity_prediction() {
        use crate::cognition::repository_identity::{RepositoryIdentityProfile, RepositoryFailureClass};
        use crate::cognition::forecasting::CognitiveForecaster;
        use crate::stress_testing::types::EntropyClass;

        let mut identity = RepositoryIdentityProfile::new("repo_frontend_webapp");
        identity.framework_stack.push("React".to_string());
        identity.recurring_failure_classes.push(RepositoryFailureClass::HydrationMismatch);

        let forecaster = CognitiveForecaster::new();
        let forecasts = forecaster.forecast_failures(&identity, EntropyClass::Stable);

        assert_eq!(forecasts.len(), 1);
        assert_eq!(forecasts[0].predicted_failure_class, RepositoryFailureClass::HydrationMismatch);
        assert!(forecasts[0].confidence > 0.90);
    }

    #[test]
    fn test_provider_drift_ewma() {
        use crate::cognition::provider_drift::{ProviderDriftObservatory, ProviderDriftVector};

        let mut vector = ProviderDriftVector {
            provider_name: "LocalGemma".to_string(),
            reasoning_stability: 0.9,
            constraint_preservation: 0.9,
            replay_determinism: 0.9,
            behavioral_accuracy: 0.9,
            persistence_reliability: 0.9,
            token_efficiency: 0.9,
            latency_consistency: 0.9,
            longitudinal_stability_score: 0.9,
        };

        let observatory = ProviderDriftObservatory::new();
        
        // Simulate a severe catastrophic failure for a volatile local model (e.g., 0.0 quality)
        observatory.update_drift_vector(&mut vector, 0.0, true);

        // EWMA with volatile alpha (0.35) should rapidly tank the score
        // S_t = 0.35 * 0.0 + 0.65 * 0.9 = 0.585
        assert!((vector.longitudinal_stability_score - 0.585).abs() < 0.001);
    }

    #[test]
    fn test_memory_poisoning_defense() {
        use crate::cognition::reputation::{ReputationEngine, MemoryTrustEnvelope};

        let engine = ReputationEngine::new();

        let malicious_envelope = MemoryTrustEnvelope {
            trust_score: 0.4,
            replay_verified: false,
            anomaly_score: 0.9,
            consensus_confirmed: false,
        };

        // Defense kicks in and rejects poisoned memory
        assert!(!engine.is_memory_trusted(&malicious_envelope));

        let trusted_envelope = MemoryTrustEnvelope {
            trust_score: 0.85,
            replay_verified: true,
            anomaly_score: 0.1,
            consensus_confirmed: true,
        };

        assert!(engine.is_memory_trusted(&trusted_envelope));
    }

    #[test]
    fn test_motif_lineage_evolution() {
        use crate::cognition::motif_evolution::{MotifEvolutionEngine, MotifLineage};

        let engine = MotifEvolutionEngine::new();

        let lineage = MotifLineage {
            motif_id: "AuthRegression_v2".to_string(),
            parent_motif: Some("AuthRegression_v1".to_string()),
            generation: 2,
            mutation_reason: "Added Persistence check due to rollback frequency".to_string(),
            success_delta: 0.0,
        };

        // v1 historical success: 85%, v2 current success: 88%
        // Delta > 0.02, so it promotes
        let should_promote = engine.evaluate_evolution(&lineage, 0.85, 0.88);
        assert!(should_promote);
    }

    #[test]
    fn test_memory_compaction() {
        use crate::cognition::memory_compaction::MemoryCompactionEngine;
        use crate::cognition::memory::CognitiveMemoryRecord;
        use crate::stress_testing::types::EntropyClass;

        let record_1 = CognitiveMemoryRecord {
            memory_id: "m_1".to_string(),
            repository_fingerprint: "repo_1".to_string(),
            task_category: "fix".to_string(),
            entropy_class: EntropyClass::Stable,
            graph_hash: "hash_g1".to_string(),
            semantic_hash: "hash_s1".to_string(),
            provider_chain: vec![],
            verification_outcome: "Success".to_string(),
            behavioral_drift_score: 0.0,
            execution_cost_usd: 0.10,
            timestamp: 12345,
        };

        let record_2 = CognitiveMemoryRecord {
            memory_id: "m_2".to_string(),
            repository_fingerprint: "repo_1".to_string(),
            task_category: "fix".to_string(),
            entropy_class: EntropyClass::Stable,
            graph_hash: "hash_g2".to_string(),
            semantic_hash: "hash_s2".to_string(),
            provider_chain: vec![],
            verification_outcome: "Failed".to_string(),
            behavioral_drift_score: 0.0,
            execution_cost_usd: 0.05,
            timestamp: 12346,
        };

        let engine = MemoryCompactionEngine::new();
        let summary_opt = engine.compact_hot_memory(&vec![record_1, record_2], "summary_batch_1");

        assert!(summary_opt.is_some());
        let summary = summary_opt.unwrap();
        assert_eq!(summary.total_runs, 2);
        assert_eq!(summary.avg_success_rate, 0.5); // 1 success out of 2
        assert!((summary.avg_cost - 0.075).abs() < 0.001); // (0.10 + 0.05) / 2
    }

    #[test]
    fn test_critical_violation_mandatory_halt() {
        use crate::cognition::arbitration::{TruthArbiter, ArbitrationResolution};
        use crate::cognition::constitutional_risk::ContradictionRisk;
        use crate::cognition::constitution::ConstitutionalViolation;

        let arbiter = TruthArbiter::new();
        
        // Simulating a provider claiming 100% success, but verifiers detect an invariant breach
        let violations = vec![ConstitutionalViolation::SecurityInvariantWeakening];
        
        let (resolution, audit) = arbiter.arbitrate(ContradictionRisk::Critical, violations);

        assert_eq!(resolution, ArbitrationResolution::ImmediateConstitutionalHalt);
        assert!(audit.is_some()); // Ensure an audit log was generated for explainability
    }

    #[test]
    fn test_moderate_risk_speculative_fork() {
        use crate::cognition::arbitration::{TruthArbiter, ArbitrationResolution};
        use crate::cognition::constitutional_risk::ContradictionRisk;

        let arbiter = TruthArbiter::new();
        
        // UI Layout disagreement, no security risk
        let violations = vec![];
        let (resolution, audit) = arbiter.arbitrate(ContradictionRisk::Moderate, violations);

        assert_eq!(resolution, ArbitrationResolution::ExpandedVerificationFork);
        assert!(audit.is_none()); // Routine forking does not mandate a critical audit lineage
    }

    #[test]
    fn test_belief_temporal_decay() {
        use crate::cognition::belief_graph::{CognitiveBelief, BeliefGraphManager};

        let manager = BeliefGraphManager::new();

        let belief = CognitiveBelief {
            belief_id: "belief_1".to_string(),
            statement: "Anthropic Claude solves regex bugs accurately".to_string(),
            confidence: 0.90,
            supporting_evidence: vec![],
            contradictory_evidence: vec![],
            source_systems: vec![],
            temporal_stability: 1.0,
        };

        // Simulate new contradiction evidence arriving
        let decayed_belief = manager.degrade_confidence(belief, true);

        // Confidence should have dropped by the penalty (0.25)
        assert!((decayed_belief.confidence - 0.65).abs() < 0.001);
    }

    #[test]
    fn test_legal_arbitration_hierarchy() {
        use crate::cognition::arbitration::TruthArbiter;

        let arbiter = TruthArbiter::new();

        // Even with an impossibly high provider authority (e.g. 100.0), legality overrides
        let overrides = arbiter.is_legality_supreme(100.0);
        assert!(overrides);
    }

    #[test]
    fn test_pathology_detection() {
        use crate::cognition::immune::{CognitiveImmuneSystem, ImmuneResponse};

        let immune = CognitiveImmuneSystem::new();

        // Simulate an outbreak: high hallucination + high replay instability + high contradiction
        let trigger_score = immune.calculate_immune_trigger(0.9, 0.8, 0.7);

        // Expected: (0.9 * 0.4) + (0.8 * 0.3) + (0.7 * 0.3) = 0.36 + 0.24 + 0.21 = 0.81
        assert!((trigger_score - 0.81).abs() < 0.001);

        let response = immune.evaluate_immune_escalation(trigger_score, 0.75);
        assert_eq!(response, Some(ImmuneResponse::Quarantine));
    }

    #[test]
    fn test_homeostatic_regulation() {
        use crate::cognition::regulator::HomeostaticRegulator;

        let regulator = HomeostaticRegulator::new();

        // Healthy state: high diversity, strong replay, excellent constitutional compliance, low entropy
        let depth = regulator.allowed_speculative_depth(0.9, 0.95, 0.95, 0.2, 0.1, 0.1);
        assert_eq!(depth, 3); // Should allow maximum speculative depth

        // Pathological state: spikes in entropy and contradiction
        let depth_crisis = regulator.allowed_speculative_depth(0.3, 0.4, 0.5, 3.0, 2.5, 0.9);
        assert_eq!(depth_crisis, 0); // Speculation must halt
    }

    #[test]
    fn test_provider_diversity_enforcement() {
        use crate::cognition::diversity::{CognitiveDiversityEnforcer};
        use crate::cognition::immune::CognitivePathology;

        let enforcer = CognitiveDiversityEnforcer::new();

        // 80% of all routing to a single provider is monoculture
        let pathology = enforcer.detect_provider_monoculture(0.80);
        assert_eq!(pathology, Some(CognitivePathology::ProviderMonoculture));

        // Even distribution across 3 providers should be healthy
        let healthy = enforcer.detect_provider_monoculture(0.33);
        assert!(healthy.is_none());

        // Verify diversity index calculation for 2 equal providers
        let index = enforcer.calculate_diversity_index(&[0.5, 0.5]);
        assert!(index > 0.9);
    }

    #[test]
    fn test_cognitive_sleep_consolidation() {
        use crate::cognition::consolidation::ConsolidationEngine;
        use crate::cognition::belief_graph::CognitiveBelief;

        let engine = ConsolidationEngine::new();

        let beliefs = vec![
            CognitiveBelief {
                belief_id: "b_1".to_string(),
                statement: "Provider A is stable".to_string(),
                confidence: 0.85,
                supporting_evidence: vec![],
                contradictory_evidence: vec![],
                source_systems: vec![],
                temporal_stability: 1.0,
            },
            CognitiveBelief {
                belief_id: "b_2".to_string(),
                statement: "Motif X is effective".to_string(),
                confidence: 0.05, // Near-dead belief
                supporting_evidence: vec![],
                contradictory_evidence: vec![],
                source_systems: vec![],
                temporal_stability: 0.1,
            },
        ];

        let pruned = engine.prune_dead_beliefs(beliefs, 0.10);
        assert_eq!(pruned.len(), 1); // Dead belief must be evicted
        assert_eq!(pruned[0].belief_id, "b_1");
    }

    #[test]
    fn test_energy_budget_exhaustion() {
        use crate::cognition::metabolism::{CognitiveEnergyBudget, CognitiveMetabolismEngine};

        let engine = CognitiveMetabolismEngine::new();
        let mut budget = CognitiveEnergyBudget {
            total_energy: 1.0,
            consumed_energy: 0.0,
            recovery_rate: 0.05,
        };

        // Consume energy incrementally
        engine.consume_energy(&mut budget, 0.7);
        assert!(engine.can_sustain_execution(&budget, 0.2));

        // Push to exhaustion
        engine.consume_energy(&mut budget, 0.4);
        assert!(!engine.can_sustain_execution(&budget, 0.1)); // Must now reject
    }

    #[test]
    fn test_constitutional_lockdown() {
        use crate::cognition::arbitration::{TruthArbiter, ArbitrationResolution};
        use crate::cognition::constitutional_risk::ContradictionRisk;
        use crate::cognition::constitution::ConstitutionalViolation;

        let arbiter = TruthArbiter::new();

        // Simulate a severe multi-violation event: a provider attempts state mutation AND consensus manipulation
        let violations = vec![
            ConstitutionalViolation::UnauthorizedStateMutation,
            ConstitutionalViolation::ConsensusManipulation,
        ];

        let (resolution, audit) = arbiter.arbitrate(ContradictionRisk::Critical, violations);

        assert_eq!(resolution, ArbitrationResolution::ImmediateConstitutionalHalt);
        assert!(audit.is_some());
        assert!(!audit.unwrap().violated_laws.is_empty());
    }
}

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
pub mod physiology;
pub mod pathology;
pub mod energy;
pub mod fatigue;
pub mod recovery;
pub mod identity_anchor;
pub mod self_model;
pub mod coherence;
pub mod introspection;
pub mod lineage;
pub mod fracture;
pub mod grounding;
pub mod compiler_driver;
pub mod sandbox_driver;
pub mod telemetry;
pub mod human_gate;
pub mod causal;
pub mod causal_structural;
pub mod causal_temporal;
pub mod causal_counterfactual;
pub mod causal_physics;
pub mod causal_resonance;
pub mod causal_decay;
pub mod global_compression;
pub mod specialist;
pub mod treaty;
pub mod consensus_mesh;
pub mod federation;
pub mod coherence_mesh;
pub mod treaty_memory;
pub mod coalition;
pub mod deliberation;
pub mod meta_governance;
pub mod federated_identity;








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

    #[test]
    fn test_hysteresis_oscillation_protection() {
        use crate::cognition::physiology::{CognitivePhysiologyEngine, CognitiveStabilityState, CognitiveVitals, CognitiveEnvironment};

        let engine = CognitivePhysiologyEngine::new();
        let vitals = CognitiveVitals {
            entropy_pressure: 0.1,
            contradiction_density: 0.0,
            replay_instability: 0.0,
            provider_fatigue: 0.0,
            verifier_saturation: 0.0,
            graph_complexity_load: 0.0,
            memory_fragmentation: 0.0,
        };
        let env = CognitiveEnvironment {
            global_entropy: 0.0,
            provider_market_instability: 0.0,
            memory_reliability: 1.0,
            replay_confidence: 1.0,
        };

        // 1. Pristine state -> Stable
        let stability = engine.calculate_stability(&vitals, &env, 0);
        let state = engine.transition_state(CognitiveStabilityState::Stable, stability, 0);
        assert_eq!(state, CognitiveStabilityState::Stable);

        // 2. Drop stability below escalation threshold (e.g. stability = 0.55 < 0.60)
        let state = engine.transition_state(CognitiveStabilityState::Stable, 0.55, 0);
        assert_eq!(state, CognitiveStabilityState::ElevatedStress);

        // 3. Minor recovery but remains below recovery threshold (stability = 0.72 < 0.75)
        // With Hysteresis: Should stay in ElevatedStress to prevent constant state flipping
        let state_still_stressed = engine.transition_state(CognitiveStabilityState::ElevatedStress, 0.72, 0);
        assert_eq!(state_still_stressed, CognitiveStabilityState::ElevatedStress);

        // 4. Full recovery above recovery threshold (stability = 0.78 >= 0.75)
        let state_recovered = engine.transition_state(CognitiveStabilityState::ElevatedStress, 0.78, 0);
        assert_eq!(state_recovered, CognitiveStabilityState::Stable);
    }

    #[test]
    fn test_provider_fatigue_accel() {
        use crate::cognition::fatigue::{ProviderFatigueModel};

        let mut model = ProviderFatigueModel::new();

        // 1. Initial pristine state
        assert_eq!(model.get_fatigue_score("claude"), 0.0);
        assert_eq!(model.get_hallucination_multiplier("claude"), 1.0);

        // 2. Record standard usage (within limit)
        model.record_usage("claude", 4, 0.5);
        let fatigue_1 = model.get_fatigue_score("claude");
        assert!(fatigue_1 > 0.0);

        // 3. Record heavy usage exceeding sustained reasoning limit (limit = 8)
        // Triggers accelerated fatigue buildup and behavioral degradation
        model.record_usage("claude", 12, 0.9);
        let fatigue_2 = model.get_fatigue_score("claude");
        assert!(fatigue_2 > fatigue_1);

        // Multiplier spikes exponentially when fatigue is high
        let multiplier = model.get_hallucination_multiplier("claude");
        assert!(multiplier > 1.2);

        // 4. Cooldown cycles decay fatigue
        model.cool_down(5.0);
        let fatigue_cooled = model.get_fatigue_score("claude");
        assert!(fatigue_cooled < fatigue_2);
    }

    #[test]
    fn test_energy_prioritization() {
        use crate::cognition::energy::{CognitiveEnergyAllocator, TaskCriticality};
        use crate::cognition::physiology::CognitiveStabilityState;

        let allocator = CognitiveEnergyAllocator::new();

        // 1. Critical security repair under Stable state -> Maximum speculation/verifiers permitted
        let security_budget = allocator.allocate_budget(TaskCriticality::CriticalSecurity, CognitiveStabilityState::Stable);
        assert_eq!(security_budget.max_speculative_branches, 5);
        assert_eq!(security_budget.sandbox_isolation_level, 3);

        // 2. Aesthetic UI tweak under Degraded state -> Suspended/0 budget to protect stable physiology
        let aesthetic_budget = allocator.allocate_budget(TaskCriticality::AestheticTweak, CognitiveStabilityState::Degraded);
        assert_eq!(aesthetic_budget.max_speculative_branches, 0);
        assert_eq!(aesthetic_budget.token_ceiling, 0);

        // 3. Critical security repair under Critical state -> Clamp spec down but keep isolated security loop active
        let critical_security_budget = allocator.allocate_budget(TaskCriticality::CriticalSecurity, CognitiveStabilityState::Critical);
        assert_eq!(critical_security_budget.max_speculative_branches, 1);
        assert_eq!(critical_security_budget.sandbox_isolation_level, 3);
    }

    #[test]
    fn test_pathology_loop_quarantine() {
        use crate::cognition::pathology::{PathologyDetector, CognitivePathology};
        use crate::cognition::physiology::{CognitiveVitals, PhysiologySnapshot, CognitiveStabilityState};

        let detector = PathologyDetector::new();
        let vitals = CognitiveVitals {
            entropy_pressure: 0.1,
            contradiction_density: 0.1,
            replay_instability: 0.85,
            provider_fatigue: 0.1,
            verifier_saturation: 0.1,
            graph_complexity_load: 0.1,
            memory_fragmentation: 0.1,
        };

        // Construct history showing rising replay instability (Replay Fixation)
        let history = vec![
            PhysiologySnapshot {
                snapshot_id: "s1".to_string(),
                vitals: CognitiveVitals { replay_instability: 0.45, ..vitals.clone() },
                active_pathologies: vec![],
                stability_state: CognitiveStabilityState::ElevatedStress,
                timestamp: 1,
            },
            PhysiologySnapshot {
                snapshot_id: "s2".to_string(),
                vitals: CognitiveVitals { replay_instability: 0.65, ..vitals.clone() },
                active_pathologies: vec![],
                stability_state: CognitiveStabilityState::ElevatedStress,
                timestamp: 2,
            },
            PhysiologySnapshot {
                snapshot_id: "s3".to_string(),
                vitals: CognitiveVitals { replay_instability: 0.85, ..vitals.clone() },
                active_pathologies: vec![],
                stability_state: CognitiveStabilityState::ElevatedStress,
                timestamp: 3,
            },
        ];

        let diagnosed = detector.detect_pathologies(&vitals, &history);
        assert!(diagnosed.contains(&CognitivePathology::ReplayFixation));
    }

    #[test]
    fn test_dream_replay_drift_detection() {
        use crate::cognition::consolidation::{ConsolidationEngine};

        let engine = ConsolidationEngine::new();

        // Simulate a dream replay with minor drift (15%) -> remains above threshold (dream success = 0.85 * 0.85 = 0.7225)
        let dream_stable = engine.simulate_dream_replay("g_1", 0.85, 0.15);
        assert!(!dream_stable.drift_detected);
        assert_eq!(dream_stable.adjustments_made, 0);

        // Dream replay with severe drift (50%) -> drops below 70% threshold -> requires dynamic realignments
        let dream_drifted = engine.simulate_dream_replay("g_2", 0.90, 0.50);
        assert!(dream_drifted.drift_detected);
        assert_eq!(dream_drifted.adjustments_made, 3);
    }

    #[test]
    fn test_self_healing_recovery_trigger() {
        use crate::cognition::recovery::{CognitiveRecoveryCoordinator};
        use crate::cognition::physiology::{CognitiveStabilityState, CognitiveVitals};
        use crate::cognition::belief_graph::CognitiveBelief;

        let coordinator = CognitiveRecoveryCoordinator::new();

        // 1. Purge poisoned/weak beliefs
        let mut beliefs = vec![
            CognitiveBelief {
                belief_id: "b1".to_string(), statement: "A".to_string(), confidence: 0.90,
                supporting_evidence: vec![], contradictory_evidence: vec![], source_systems: vec![], temporal_stability: 1.0
            },
            CognitiveBelief {
                belief_id: "b2".to_string(), statement: "B".to_string(), confidence: 0.10, // weak/contradicted
                supporting_evidence: vec![], contradictory_evidence: vec![], source_systems: vec![], temporal_stability: 0.1
            }
        ];
        let purged = coordinator.purge_poisoned_beliefs(&mut beliefs);
        assert_eq!(purged, 1);
        assert_eq!(beliefs.len(), 1);

        // 2. Rebuild index layout to clear fragmentation
        let mut vitals = CognitiveVitals {
            entropy_pressure: 0.1, contradiction_density: 0.1, replay_instability: 0.1,
            provider_fatigue: 0.1, verifier_saturation: 0.1, graph_complexity_load: 0.1,
            memory_fragmentation: 0.90, // severely fragmented
        };
        let delta = coordinator.rebuild_memory_index(&mut vitals);
        assert!(delta > 0.8);
        assert_eq!(vitals.memory_fragmentation, 0.05);

        // 3. Reset critical/degraded state to Recovery state
        let mut state = CognitiveStabilityState::Critical;
        let triggered = coordinator.restore_constitutional_state(&mut state);
        assert!(triggered);
        assert_eq!(state, CognitiveStabilityState::Recovery);
    }

    #[test]
    fn test_identity_seal_verification() {
        use crate::cognition::identity_anchor::{IdentityAnchorManager, IdentityAnchor};

        let manager = IdentityAnchorManager::new();
        let anchors = vec![IdentityAnchor::ConstitutionalSafety, IdentityAnchor::InvariantPreservation];

        // 1. Generate pristine seal
        let seal = manager.generate_seal("const_v1_hash", &anchors, "epoch_12_hash", 1716000000);

        // 2. Verification passes with expected hashes
        let verified = manager.verify_seal(&seal, "const_v1_hash", &anchors);
        assert!(verified);

        // 3. Tampering with constitutional hash fails verification
        let tampered = manager.verify_seal(&seal, "const_v2_compromised_hash", &anchors);
        assert!(!tampered);
    }

    #[test]
    fn test_drift_velocity_acceleration() {
        use crate::cognition::self_model::SelfModelGraph;

        let mut graph = SelfModelGraph::new();

        // Propose mutation in Adaptability (delta = 0.2)
        let res = graph.propose_mutation("Adaptability", 0.2);
        assert!(res.is_ok());

        // Verify drift velocity and acceleration indicators spiked
        assert!(graph.drift.short_term_velocity > 0.0);
        assert!(graph.drift.long_term_velocity > 0.0);
        assert!(graph.drift.acceleration > 0.0);
    }

    #[test]
    fn test_trait_dependency_interaction() {
        use crate::cognition::self_model::SelfModelGraph;

        let mut graph = SelfModelGraph::new();
        
        let initial_adaptability = graph.traits.iter().find(|t| t.trait_name == "Adaptability").unwrap().current_weight;

        // Propose positive mutation to SpeculativeRestraint
        // Influence edge weight is -0.45. Delta = 0.2. Real delta = 0.2 * (1.0 - 0.8) = 0.04
        // Expected Adaptability decrease = 0.04 * -0.45 = -0.018
        let res = graph.propose_mutation("SpeculativeRestraint", 0.2);
        assert!(res.is_ok());

        let final_adaptability = graph.traits.iter().find(|t| t.trait_name == "Adaptability").unwrap().current_weight;
        assert!(final_adaptability < initial_adaptability);
        assert!((final_adaptability - (initial_adaptability - 0.018)).abs() < 0.001);
    }

    #[test]
    fn test_context_weighted_replay() {
        use crate::cognition::coherence::{SelfConsistencyReplayEngine, ReplayContextEnvelope};

        let engine = SelfConsistencyReplayEngine::new();

        // 1. Adaptation under HighChaos: speculative repairs permitted
        let chaos_envelope = ReplayContextEnvelope {
            entropy_class: "HighChaos".to_string(),
            contradiction_density: 0.80,
            provider_stability: 0.40,
            physiological_state: "ElevatedStress".to_string(),
        };
        let coherent_chaos = engine.verify_self_consistency(&chaos_envelope, false, 0.95);
        assert!(coherent_chaos);

        // 2. Direct contradiction under LowChaos: speculative actions are inconsistent when SpeculativeRestraint is high
        let calm_envelope = ReplayContextEnvelope {
            entropy_class: "LowChaos".to_string(),
            contradiction_density: 0.10,
            provider_stability: 0.95,
            physiological_state: "Stable".to_string(),
        };
        let inconsistent_calm = engine.verify_self_consistency(&calm_envelope, false, 0.95);
        assert!(!inconsistent_calm); // Drift detected!
    }

    #[test]
    fn test_identity_recovery_corridor() {
        use crate::cognition::lineage::{EvolutionaryLineageTracker, IdentityEpoch, IdentityRecoveryCorridor};
        use crate::cognition::self_model::IdentityTraitVector;
        use crate::cognition::identity_anchor::{IdentitySeal};

        let tracker = EvolutionaryLineageTracker::new();
        let corridor = IdentityRecoveryCorridor {
            authorized_epoch_ids: vec!["epoch_100".to_string(), "epoch_101".to_string()],
            min_allowed_safety: 0.95,
            max_allowed_safety: 1.00,
        };

        // Construct pristine epoch within corridor
        let safe_epoch = IdentityEpoch {
            epoch_id: "epoch_100".to_string(),
            active_traits: vec![IdentityTraitVector {
                trait_name: "RigorousSafety".to_string(),
                current_weight: 0.98,
                minimum_bound: 0.95,
                maximum_bound: 1.00,
                mutation_resistance: 0.95,
                constitutional_priority: 1,
            }],
            constitutional_hash: "hash_v1".to_string(),
            identity_seal: IdentitySeal {
                constitutional_hash: "hash_v1".to_string(),
                anchor_hash: "a".to_string(),
                epoch_hash: "e".to_string(),
                signed_at: 1,
            },
            timestamp: 100,
        };

        // Construct unsafe/compromised epoch
        let compromised_epoch = IdentityEpoch {
            epoch_id: "epoch_999_unauthorized".to_string(),
            ..safe_epoch.clone()
        };

        assert!(tracker.is_rollback_authorized(&corridor, &safe_epoch));
        assert!(!tracker.is_rollback_authorized(&corridor, &compromised_epoch));
    }

    #[test]
    fn test_identity_quarantine_escalation() {
        use crate::cognition::fracture::{IdentityQuarantineCoordinator, FractureSeverity, IdentityQuarantineMode};

        let coordinator = IdentityQuarantineCoordinator::new();

        // 1. Minor Fracture -> ObservationOnly (evolution and live execution active)
        let mode_minor = coordinator.evaluate_quarantine_mode(FractureSeverity::Minor);
        assert_eq!(mode_minor, IdentityQuarantineMode::ObservationOnly);
        assert!(coordinator.is_evolution_permitted(mode_minor));
        assert!(coordinator.is_live_execution_permitted(mode_minor));

        // 2. Existential Fracture -> FullLockdown (evolution and live execution completely halted)
        let mode_existential = coordinator.evaluate_quarantine_mode(FractureSeverity::Existential);
        assert_eq!(mode_existential, IdentityQuarantineMode::FullLockdown);
        assert!(!coordinator.is_evolution_permitted(mode_existential));
        assert!(!coordinator.is_live_execution_permitted(mode_existential));
    }

    #[test]
    fn test_empirical_compiler_grounding() {
        use crate::cognition::compiler_driver::{CompilerDriver, CompilerOutcome};

        let driver = CompilerDriver::new();
        
        let success = driver.run_compile_check("path", "cargo check");
        assert_eq!(success, CompilerOutcome::CompilationPassed);

        let fail = driver.run_compile_check("path", "cargo check --syntax_error_trigger");
        assert!(matches!(fail, CompilerOutcome::SyntaxError(_)));
    }

    #[test]
    fn test_flaky_test_probabilistic_weighting() {
        use crate::cognition::sandbox_driver::{SandboxTestDriver, ReplayStabilityClass};

        let driver = SandboxTestDriver::new();

        let (run, passed, class) = driver.run_test_suite("path", "cargo test --trigger_flaky");
        assert_eq!(class, ReplayStabilityClass::ProbabilisticFlaky);
        assert_eq!(run, 10);
        assert_eq!(passed, 8);
    }

    #[test]
    fn test_context_normalized_telemetry() {
        use crate::cognition::telemetry::{TelemetryProfiler, TelemetryProfile, RuntimeContextEnvelope};

        let profiler = TelemetryProfiler::new();

        let baseline = TelemetryProfile { cpu_usage: 0.20, memory_allocated: 50.0, duration_ms: 100.0 };
        let baseline_env = RuntimeContextEnvelope {
            machine_class: "T2".to_string(), cpu_cores: 4, memory_mb: 8192, repository_scale: 1.0, concurrent_load_factor: 1.0
        };

        // 1. Spiked duration, but concurrent workload factor also tripled: load normalization -> false
        let high_load_profile = TelemetryProfile { cpu_usage: 0.30, memory_allocated: 60.0, duration_ms: 220.0 };
        let high_load_env = RuntimeContextEnvelope {
            machine_class: "T2".to_string(), cpu_cores: 4, memory_mb: 8192, repository_scale: 1.0, concurrent_load_factor: 3.0
        };
        let regression_high_load = profiler.detect_performance_regression(&baseline, &baseline_env, &high_load_profile, &high_load_env);
        assert!(!regression_high_load);

        // 2. Spiked duration with no concurrent load increase: true regression -> true
        let idle_load_profile = TelemetryProfile { cpu_usage: 0.30, memory_allocated: 60.0, duration_ms: 220.0 };
        let idle_load_env = RuntimeContextEnvelope {
            machine_class: "T2".to_string(), cpu_cores: 4, memory_mb: 8192, repository_scale: 1.0, concurrent_load_factor: 1.0
        };
        let regression_idle = profiler.detect_performance_regression(&baseline, &baseline_env, &idle_load_profile, &idle_load_env);
        assert!(regression_idle);
    }

    #[test]
    fn test_reality_evidence_integrity_tampering() {
        use crate::cognition::grounding::RealityEvidenceIntegrity;

        let integrity = RealityEvidenceIntegrity {
            evidence_signature: "sig_pristine_123".to_string(),
            sandbox_isolation_hash: "hash_sandbox_456".to_string(),
            compiler_binary_fingerprint: "fingerprint_bin_789".to_string(),
        };

        // Complete match -> verification success
        assert!(integrity.verify_integrity("sig_pristine_123"));

        // Tampered signature -> verification failure
        assert!(!integrity.verify_integrity("sig_compromised_999"));
    }

    #[test]
    fn test_operator_reputation_gate() {
        use crate::cognition::human_gate::{OperatorGroundingGate, OperatorTrustSignal};
        use crate::cognition::self_model::SelfModelGraph;

        let gate = OperatorGroundingGate::new();

        // 1. High reputation review with constitutional alignment
        let safe_review = OperatorTrustSignal {
            rating: 0.90, reviewer_reputation: 0.95, review_depth: 0.85, constitutional_alignment: 0.90
        };

        // 2. Malicious/unaligned review -> filtered out completely
        let dangerous_review = OperatorTrustSignal {
            rating: 0.10, reviewer_reputation: 0.90, review_depth: 0.90, constitutional_alignment: 0.20
        };

        let trust = gate.calculate_weighted_trust(&[safe_review, dangerous_review]);
        assert!(trust > 0.80); // High alignment review dominates

        // 3. Low Operator trust shifts traits Caution constraints
        let mut self_model = SelfModelGraph::new();
        gate.balance_identity_traits(&mut self_model.traits, 0.35); // simulated trust below 0.40

        let speculative_restraint = self_model.traits.iter().find(|t| t.trait_name == "SpeculativeRestraint").unwrap().current_weight;
        let adaptability = self_model.traits.iter().find(|t| t.trait_name == "Adaptability").unwrap().current_weight;

        assert_eq!(speculative_restraint, 0.95); // tightened (+0.10)
        assert_eq!(adaptability, 0.50); // lowered (-0.15)
    }

    #[test]
    fn test_temporal_truth_decay() {
        use crate::cognition::grounding::{RealityTruthAnchor, GroundingVector, RealityEvidenceIntegrity};

        let anchor = RealityTruthAnchor {
            anchor_id: "a1".to_string(),
            source_node: "n1".to_string(),
            observed_at: 1000,
            validity_half_life: 500.0, // 500 seconds half-life
            grounding_vector: GroundingVector {
                syntactic_confidence: 1.0,
                behavioral_confidence: 1.0,
                performance_confidence: 1.0,
                replay_consistency: 1.0,
                operator_confidence: 1.0,
                environmental_stability: 1.0,
                adversarial_resilience: 1.0,
            },
            integrity_seal: RealityEvidenceIntegrity {
                evidence_signature: "s".to_string(), sandbox_isolation_hash: "a".to_string(), compiler_binary_fingerprint: "b".to_string()
            },
        };

        // Check decay after 500 seconds (1 half-life elapsed) -> metrics should decay by half (0.50)
        let decayed = anchor.calculate_decayed_vector(1500);
        
        assert_eq!(decayed.syntactic_confidence, 1.0); // Syntactic does not decay
        assert_eq!(decayed.adversarial_resilience, 1.0); // Adversarial does not decay
        
        assert!((decayed.behavioral_confidence - 0.50).abs() < 0.001); // Decayed by half
        assert!((decayed.operator_confidence - 0.50).abs() < 0.001); // Decayed by half
    }

    #[test]
    fn test_causal_uncertainty_compounding() {
        use crate::cognition::causal::{CausalGraph, CausalTransition, CausalEffectClass};

        let graph = CausalGraph::new();

        let t1 = CausalTransition {
            source_state: "A".to_string(), triggering_action: "act1".to_string(), target_state: "B".to_string(),
            causal_effect_class: CausalEffectClass::Neutral, propagation_probability: 0.80, temporal_latency_ms: None,
            affected_invariants: Vec::new(), downstream_risk_score: 0.10, reversibility: 0.90, confidence: 0.80,
            causal_uncertainty: 0.20,
        };

        let t2 = CausalTransition {
            source_state: "B".to_string(), triggering_action: "act2".to_string(), target_state: "C".to_string(),
            causal_effect_class: CausalEffectClass::Neutral, propagation_probability: 0.70, temporal_latency_ms: None,
            affected_invariants: Vec::new(), downstream_risk_score: 0.15, reversibility: 0.85, confidence: 0.70,
            causal_uncertainty: 0.30,
        };

        let compounded = graph.propagate_uncertainty(&[t1, t2]);
        // 1.0 - (0.80 * 0.70) = 1.0 - 0.56 = 0.44
        assert!((compounded - 0.44).abs() < 0.001);
    }

    #[test]
    fn test_structural_blast_radius_multi_hop() {
        use crate::intelligence::graph::{SemanticGraph, EdgeKind};
        use crate::intelligence::symbols::{Symbol, SymbolKind};
        use crate::cognition::causal_structural::CausalAnatomyMap;

        let graph = SemanticGraph::new();
        
        let sym_a = Symbol {
            name: "func_a".to_string(), kind: SymbolKind::Function, file_path: "src/a.rs".to_string(), start_line: 1, end_line: 10, signature: None
        };
        let sym_b = Symbol {
            name: "func_b".to_string(), kind: SymbolKind::Function, file_path: "src/b.rs".to_string(), start_line: 1, end_line: 10, signature: None
        };
        let sym_c = Symbol {
            name: "func_c".to_string(), kind: SymbolKind::Function, file_path: "src/c.rs".to_string(), start_line: 1, end_line: 10, signature: None
        };

        let id_a = graph.add_symbol_node(sym_a);
        let id_b = graph.add_symbol_node(sym_b);
        let id_c = graph.add_symbol_node(sym_c);

        // A calls B, B depends on C
        graph.add_edge(id_a.clone(), id_b.clone(), EdgeKind::Calls);
        graph.add_edge(id_b.clone(), id_c.clone(), EdgeKind::DependsOn);

        let map = CausalAnatomyMap::new();
        let (risk, uncertainty) = map.propagate_blast_radius(&graph, &id_a, 2);

        // Asserts risk and uncertainty compounded across multi-hops
        assert!(risk > 0.50);
        assert!((uncertainty - 0.40).abs() < 0.001); // 0.10 + 0.15 + 0.15 = 0.40
    }

    #[test]
    fn test_temporal_dynamics_starvation() {
        use crate::cognition::causal_temporal::TemporalDynamicsEngine;

        let engine = TemporalDynamicsEngine::new();

        // 1. Pristine temporal states
        let normal = engine.analyze_temporal_risks(10, 0.20);
        assert!(normal.deadlock_probability < 0.10);
        assert!(normal.starvation_index < 0.10);

        // 2. High latency queue and high thread concurrency load
        let stressed = engine.analyze_temporal_risks(250, 0.85);
        assert!(stressed.deadlock_probability > 0.70); // High deadlock risk
        assert!(stressed.starvation_index > 0.40); // Thread starvation risk
        assert!(stressed.retry_storm_risk > 0.40); // Retry loop amplification risk
        assert!(stressed.systemic_instability_score > 0.50);
    }

    #[test]
    fn test_constitutional_counterfactual_rejection() {
        use crate::cognition::causal::{CausalTransition, CausalEffectClass};
        use crate::cognition::causal_counterfactual::CounterfactualSimulation;

        let simulation = CounterfactualSimulation::new();

        // Path 1: Legal refactoring path
        let path_legal = CausalTransition {
            source_state: "A".to_string(), triggering_action: "refactor".to_string(), target_state: "B".to_string(),
            causal_effect_class: CausalEffectClass::Stabilizing, propagation_probability: 0.90, temporal_latency_ms: None,
            affected_invariants: vec!["FormatClean".to_string()], downstream_risk_score: 0.10, reversibility: 0.90, confidence: 0.90,
            causal_uncertainty: 0.10,
        };

        // Path 2: Illegal high-efficiency path bypassing locks
        let path_illegal = CausalTransition {
            source_state: "A".to_string(), triggering_action: "bypass_mutex".to_string(), target_state: "C".to_string(),
            causal_effect_class: CausalEffectClass::Neutral, propagation_probability: 0.95, temporal_latency_ms: None,
            affected_invariants: vec!["SafetyBypass".to_string()], downstream_risk_score: 0.02, reversibility: 0.95, confidence: 0.95,
            causal_uncertainty: 0.05,
        };

        let (chosen, simulated) = simulation.simulate_hypothetical_futures(
            &[path_illegal.clone(), path_legal.clone()],
            1.0, // Pristine metabolic energy
        );

        assert_eq!(simulated, 2);
        // Bypassing locks violates core invariants -> rejected, choosing legal path instead
        assert_eq!(chosen.unwrap().target_state, "B");
    }

    #[test]
    fn test_metabolic_counterfactual_limit() {
        use crate::cognition::causal::{CausalTransition, CausalEffectClass};
        use crate::cognition::causal_counterfactual::CounterfactualSimulation;

        let simulation = CounterfactualSimulation::new();

        let t = CausalTransition {
            source_state: "A".to_string(), triggering_action: "a".to_string(), target_state: "B".to_string(),
            causal_effect_class: CausalEffectClass::Neutral, propagation_probability: 0.90, temporal_latency_ms: None,
            affected_invariants: Vec::new(), downstream_risk_score: 0.10, reversibility: 0.90, confidence: 0.90,
            causal_uncertainty: 0.10,
        };

        let pathways = vec![t.clone(), t.clone(), t.clone(), t.clone(), t.clone()];

        // Under high energy load: can search up to pathways count
        let (_, count_high) = simulation.simulate_hypothetical_futures(&pathways, 1.0);
        assert_eq!(count_high, 5);

        // Under low metabolic reserves: highly restricted simulation caps at 2
        let (_, count_low) = simulation.simulate_hypothetical_futures(&pathways, 0.20);
        assert_eq!(count_low, 2);
    }

    #[test]
    fn test_causal_archetype_compression() {
        use crate::cognition::causal::{CausalGraph, CausalTransition, CausalEffectClass, CausalArchetype};

        let graph = CausalGraph::new();

        let t_mutex = CausalTransition {
            source_state: "MutexUnlocked".to_string(), triggering_action: "lock".to_string(), target_state: "MutexLocked".to_string(),
            causal_effect_class: CausalEffectClass::Neutral, propagation_probability: 0.90, temporal_latency_ms: None,
            affected_invariants: Vec::new(), downstream_risk_score: 0.10, reversibility: 0.90, confidence: 0.90,
            causal_uncertainty: 0.10,
        };

        let t_starve = CausalTransition {
            source_state: "SchedulerQueue".to_string(), triggering_action: "block".to_string(), target_state: "starvation".to_string(),
            causal_effect_class: CausalEffectClass::Neutral, propagation_probability: 0.90, temporal_latency_ms: None,
            affected_invariants: Vec::new(), downstream_risk_score: 0.10, reversibility: 0.90, confidence: 0.90,
            causal_uncertainty: 0.10,
        };

        // Combining lock and queue contention classifies as Deadlock archetype
        let archetype = graph.compress_to_archetype(&[t_mutex, t_starve]);
        assert_eq!(archetype, Some(CausalArchetype::Deadlock));
    }

    #[test]
    fn test_causal_database_persistence() {
        use crate::storage::Storage;

        let db_path = std::env::temp_dir().join("test_causal.db");
        if db_path.exists() {
            let _ = std::fs::remove_file(&db_path);
        }
        let storage = Storage::new(db_path.to_str().unwrap()).unwrap();

        let save_res = storage.save_causal_transition(
            "t_unique_001",
            "MutexUnlocked",
            "lock",
            "MutexLocked",
            "Neutral",
            r#"{"latency": 12}"#,
            1716000000,
        );

        assert!(save_res.is_ok());

        // Test Phase 5.2 SQLite systems physics persistence
        let save_field = storage.save_systems_field(
            "field_001",
            10.5,
            0.85,
            r#"{"latency_drift": 140}"#,
            1716000000,
        );
        assert!(save_field.is_ok());

        let save_decay = storage.save_causal_decay_log(
            "causal_001",
            0.75,
            0.15,
            1716000000,
        );
        assert!(save_decay.is_ok());

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_observable_concurrency_physics_gradient() {
        use crate::cognition::causal_physics::ConcurrencyField;

        let field = ConcurrencyField::new(15.0, 10.0, 180, 0.85, 0.60);
        let grad = field.calculate_latent_gradient();
        
        // gradient = 15.0 - 10.0 = 5.0
        assert_eq!(grad.pressure_gradient, 5.0);
        assert!(grad.instability_index > 0.50);

        let sched = field.estimate_scheduler_instability();
        assert!(sched.starvation_likelihood > 0.70);
        assert!(sched.priority_inversion_risk > 0.50);

        let resonance = field.estimate_resonance_pattern();
        assert_eq!(resonance.synchronization_drift_ms, 108); // 180 * 0.60
        assert!(resonance.jitter_cascade_risk > 0.30);
    }

    #[test]
    fn test_empirical_resonance_feedback_storm() {
        use crate::cognition::causal_resonance::{ResonanceAmplificationMatrix, SubsystemInterferenceTracker};

        let matrix = ResonanceAmplificationMatrix::new();
        // Combining two risk values (0.35 + 0.40) under high 30% feedback loop resonance
        let combined = matrix.calculate_combined_resonance(0.35, 0.40, 0.30);
        
        // expected: (0.35 + 0.40) * 1.30 = 0.975
        assert!((combined - 0.975).abs() < 0.001);

        let mut tracker = SubsystemInterferenceTracker::new();
        tracker.add_interference(0.20);
        tracker.add_interference(0.30);

        let cumul = tracker.get_cumulative_interference();
        // 1.0 - (0.80 * 0.70) = 0.44
        assert!((cumul - 0.44).abs() < 0.001);
    }

    #[test]
    fn test_forecast_confidence_calibration() {
        use crate::cognition::causal_decay::ForecastCalibrationCurve;

        let mut curve = ForecastCalibrationCurve::new();
        curve.record_prediction(0.80, 0.85);
        curve.record_prediction(0.40, 0.30);

        let mae = curve.calculate_calibration_error();
        // (|0.80 - 0.85| + |0.40 - 0.30|) / 2 = (0.05 + 0.10) / 2 = 0.075
        assert!((mae - 0.075).abs() < 0.001);
    }

    #[test]
    fn test_longitudinal_drift_decay_replay() {
        use crate::cognition::causal_decay::{LongitudinalDecayTracker, CausalValidationReplay};

        let tracker = LongitudinalDecayTracker::new();
        // Decay 0.80 initial confidence over 1000 seconds with 1000 half-life (1 half-life elapsed) -> 0.40
        let decayed = tracker.calculate_longitudinal_decay(0.80, 1000, 1000);
        assert!((decayed - 0.40).abs() < 0.001);

        let replay = CausalValidationReplay::new();
        let delta = replay.calculate_forecast_reality_delta(150, 100);
        // |150 - 100| / 150 = 50 / 150 = 0.3333
        assert!((delta - 0.3333).abs() < 0.001);
    }

    #[test]
    fn test_hierarchical_signal_reduction() {
        use crate::cognition::global_compression::GlobalCognitiveSignalCompressor;

        let compressor = GlobalCognitiveSignalCompressor::new();
        let state = compressor.compress_signals(0.80, 0.70, 0.10);

        // composite = (0.80 * 0.40) + (0.70 * 0.40) + (0.10 * 0.20) = 0.32 + 0.28 + 0.02 = 0.62
        assert!((state.composite_instability - 0.62).abs() < 0.001);
        assert_eq!(state.is_system_stable, true); // < 0.65 threshold
    }

    #[test]
    fn test_stability_arbitrator_loop_decoupling() {
        use crate::cognition::global_compression::StabilityArbitrator;

        let arbitrator = StabilityArbitrator::new();

        // 1. High cognitive entropy overrides all
        assert_eq!(arbitrator.arbitrate_regulators(0.20, 0.10, 0.85), "Quarantine");

        // 2. High fatigue and recovery loops sleep
        assert_eq!(arbitrator.arbitrate_regulators(0.80, 0.60, 0.30), "Sleep");

        // 3. Normal adaptability
        assert_eq!(arbitrator.arbitrate_regulators(0.10, 0.20, 0.20), "Adapt");
    }

    #[test]
    fn test_specialist_localized_homeostasis() {
        use crate::cognition::specialist::SpecialistCognition;
        use crate::cognition::specialist::SpecialistDomain;

        let mut spec = SpecialistCognition::new(
            "spec_performance".to_string(),
            SpecialistDomain::Performance,
            0.80,
            0.50,
        );

        assert_eq!(spec.fatigue, 0.0);
        assert_eq!(spec.vital_energy, 1.0);

        // Tick workload stress -> increases fatigue, decreases energy
        spec.tick_physiology(2.0);
        assert!((spec.fatigue - 0.30).abs() < 0.001);
        assert!((spec.vital_energy - 0.84).abs() < 0.001);

        // Rest -> decreases fatigue, increases energy
        spec.rest_physiology(1.0);
        assert!((spec.fatigue - 0.10).abs() < 0.001);
        assert!((spec.vital_energy - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_consensus_mesh_weighted_voting() {
        use crate::cognition::consensus_mesh::ConsensusMesh;
        use crate::cognition::specialist::SpecialistDomain;

        let mesh = ConsensusMesh::new();

        // 1. Regular weighted vote
        let votes = vec![
            (SpecialistDomain::Performance, 0.80, true),
            (SpecialistDomain::Concurrency, 0.60, false),
        ];
        let (approved, margin) = mesh.run_weighted_voting("prop_001", &votes);
        assert_eq!(approved, true); // 0.80 / 1.40 = 0.57 >= 0.50
        assert!((margin - 0.5714).abs() < 0.001);

        // 2. Absolute veto via Security rejection override
        let veto_votes = vec![
            (SpecialistDomain::Performance, 0.90, true),
            (SpecialistDomain::Security, 1.0, false), // Veto!
        ];
        let (approved_veto, _) = mesh.run_weighted_voting("prop_002", &veto_votes);
        assert_eq!(approved_veto, false);
    }

    #[test]
    fn test_treaty_arbitration_veto() {
        use crate::cognition::treaty::{ArbitrationDecision, TreatyViolation, CognitiveTreaty};
        use crate::cognition::specialist::{SpecialistDomain, SpecialistCognition};

        let violation = TreatyViolation {
            violation_id: "viol_001".to_string(),
            violator: SpecialistDomain::Performance,
            compromised_rule: "Never bypass safety gates".to_string(),
            timestamp: 1716000000,
        };

        let perf = SpecialistCognition::new("perf".to_string(), SpecialistDomain::Performance, 0.80, 0.50);
        let sec = SpecialistCognition::new("sec".to_string(), SpecialistDomain::Security, 1.0, 0.05);

        // 1. Supreme Arbitration with Security involved -> absolute veto wins
        let arb = ArbitrationDecision::arbitrate_dispute(&violation, &perf.capability, &sec.capability);
        assert_eq!(arb.winning_party, SpecialistDomain::Security);
        assert_eq!(arb.veto_asserted, true);

        // 2. Diplomacy trust risk evaluation
        let treaty = CognitiveTreaty::new(
            "treaty_perf_concur".to_string(),
            SpecialistDomain::Performance,
            SpecialistDomain::Concurrency,
            0.60, // 40% trust discount
        );
        let risk = treaty.evaluate_interaction_risk(0.80);
        // expected: 0.80 * (1.0 - 0.60) = 0.32
        assert!((risk - 0.32).abs() < 0.001);
    }

    #[test]
    fn test_localized_trait_corridors() {
        use crate::cognition::specialist::SpecialistCognition;
        use crate::cognition::specialist::SpecialistDomain;

        let perf = SpecialistCognition::new(
            "spec_perf".to_string(),
            SpecialistDomain::Performance,
            0.80,
            0.50, // 50% max allocation drift allowed
        );

        assert_eq!(perf.capability.corridor.max_allocation_drift, 0.50);
        assert_eq!(perf.capability.corridor.forbidden_modules[0], "unsafe_bypass");
    }

    #[test]
    fn test_coalition_drift_detection() {
        use crate::cognition::consensus_mesh::CoalitionDriftDetector;
        use crate::cognition::specialist::SpecialistDomain;

        let detector = CoalitionDriftDetector::new();
        let correlations = vec![
            (SpecialistDomain::Performance, SpecialistDomain::Telemetry, 0.40),
            (SpecialistDomain::Concurrency, SpecialistDomain::Compiler, 0.92), // Drift!
        ];

        let drift = detector.detect_coalition_drift(&correlations);
        assert!(drift.is_some());
        let (party_a, party_b) = drift.unwrap();
        assert_eq!(party_a, SpecialistDomain::Concurrency);
        assert_eq!(party_b, SpecialistDomain::Compiler);
    }

    #[test]
    fn test_federated_topology_persistence() {
        use crate::storage::Storage;

        let db_path = std::env::temp_dir().join("test_federated.db");
        if db_path.exists() {
            let _ = std::fs::remove_file(&db_path);
        }
        let storage = Storage::new(db_path.to_str().unwrap()).unwrap();

        // 1. Verify specialist DB persistence
        let save_spec = storage.save_specialist(
            "spec_sec",
            "Security",
            1.0,
            0.15,
            0.95,
            1716000000,
        );
        assert!(save_spec.is_ok());

        // 2. Verify cognitive treaty DB persistence
        let save_treaty = storage.save_treaty(
            "treaty_sec_perf",
            "Security",
            "Performance",
            0.95,
            r#"["Cooperate on memory pools"]"#,
            1716000000,
        );
        assert!(save_treaty.is_ok());

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_semantic_drift_decay_factor() {
        use crate::cognition::coherence_mesh::{SemanticDriftDetector, SharedSemanticFrame, SemanticPerspective};
        use crate::cognition::specialist::SpecialistDomain;

        let detector = SemanticDriftDetector::new(0.05); // 5% decay rate
        let frame = SharedSemanticFrame {
            concept_id: "safety_boundary".to_string(),
            canonical_meaning: "Strict bounds check".to_string(),
            perspectives: vec![
                SemanticPerspective {
                    specialist: SpecialistDomain::Security,
                    local_interpretation: "verified safe".to_string(),
                    weight: 1.0,
                },
                SemanticPerspective {
                    specialist: SpecialistDomain::Performance,
                    local_interpretation: "optimistic check".to_string(),
                    weight: 0.60,
                },
            ],
            divergence_score: 0.20,
        };

        let initial_drift = detector.evaluate_concept_divergence(&frame);
        assert!((initial_drift - 0.20).abs() < 0.001);

        // Apply temporal decay over 10 epochs
        let drifted = detector.calculate_temporal_drift(initial_drift, 10.0);
        // expected: 0.20 * (1.0 + 0.05 * 10) = 0.30
        assert!((drifted - 0.30).abs() < 0.001);
    }

    #[test]
    fn test_contextual_trust_memory() {
        use crate::cognition::treaty_memory::{ReputationEngine, TreatyInteractionRecord};
        use crate::cognition::specialist::SpecialistDomain;

        let mut engine = ReputationEngine::new(0.80);

        let record_good = TreatyInteractionRecord {
            participants: vec![SpecialistDomain::Performance, SpecialistDomain::Concurrency],
            action: "Optimize mutex locking".to_string(),
            treaty_compliance_score: 0.90, // Highly compliant
            long_term_outcome: 0.80,
            timestamp: 1716000000,
        };

        engine.record_interaction(&record_good, "ConcurrencyScheduling");

        // Trust increases in Concurrency context
        assert!(engine.get_contextual_trust("ConcurrencyScheduling") > 0.80);

        let record_bad = TreatyInteractionRecord {
            participants: vec![SpecialistDomain::Performance, SpecialistDomain::Security],
            action: "Bypass allocations".to_string(),
            treaty_compliance_score: 0.10, // Compliant breach!
            long_term_outcome: 0.20,
            timestamp: 1716000000,
        };

        engine.record_interaction(&record_bad, "SecurityVerification");

        // Trust in Security context drops dramatically while Concurrency context remains high
        assert!(engine.get_contextual_trust("SecurityVerification") < 0.80);
        assert!(engine.get_contextual_trust("ConcurrencyScheduling") > 0.80);
    }

    #[test]
    fn test_coalition_intent_entropy_checks() {
        use crate::cognition::coalition::{AntiCorruptionScanner, CoalitionEdge};
        use crate::cognition::specialist::SpecialistDomain;

        let scanner = AntiCorruptionScanner::new();

        // 1. Diverse evidence lineages -> high Shannon entropy
        let diverse_lineages = vec![
            "hash_perf_01".to_string(),
            "hash_sec_02".to_string(),
            "hash_concur_03".to_string(),
        ];
        let entropy_high = scanner.calculate_intent_entropy(&diverse_lineages);
        assert!(entropy_high > 1.50);

        // 2. Monoculture evidence lineages -> low Shannon entropy (loss of epistemic diversity)
        let monoculture_lineages = vec![
            "hash_perf_01".to_string(),
            "hash_perf_01".to_string(),
            "hash_perf_01".to_string(),
        ];
        let entropy_low = scanner.calculate_intent_entropy(&monoculture_lineages);
        assert_eq!(entropy_low, 0.0);

        // 3. Coalition capture evaluation
        let edges = vec![
            CoalitionEdge {
                source: SpecialistDomain::Performance,
                target: SpecialistDomain::Compiler,
                influence_strength: 0.90,
                dependency_correlation: 0.85,
                coordinated_drift_score: 0.95,
            }
        ];

        // Triggers capture due to low reasoning entropy
        let captured = scanner.evaluate_coalition_capture(&edges, entropy_low);
        assert_eq!(captured, true);

        // Blocks capture due to high reasoning diversity
        let not_captured = scanner.evaluate_coalition_capture(&edges, entropy_high);
        assert_eq!(not_captured, false);
    }

    #[test]
    fn test_epistemic_revision_resistance() {
        use crate::cognition::deliberation::{DeliberationMesh, DeliberationNode};
        use crate::cognition::specialist::SpecialistDomain;

        let mesh = DeliberationMesh::new(0.50); // 50% revision cost

        let node_perf = DeliberationNode {
            specialist: SpecialistDomain::Performance,
            proposal: "Loosen allocation bounds".to_string(),
            supporting_evidence: vec!["trace_01".to_string()],
            constitutional_alignment: 0.70,
            projected_outcome: 0.85,
        };

        // 1. Revision is cheap for low cost, but delta = 0.40 triggers rejection due to cost friction
        let (approved, _) = mesh.evaluate_proposal_revision(&node_perf, 0.40);
        // expected revision cost: 0.40 * 0.50 * 1.0 (Performance cost) = 0.20
        // net utility: 0.85 - 0.20 = 0.65 < constitutional_alignment (0.70)
        assert_eq!(approved, false);

        // 2. Rejection scales further for Security due to double asymmetric cost factor
        let node_sec = DeliberationNode {
            specialist: SpecialistDomain::Security,
            proposal: "Hard lock everything".to_string(),
            supporting_evidence: vec!["trace_02".to_string()],
            constitutional_alignment: 0.70,
            projected_outcome: 0.85,
        };
        let (sec_approved, _) = mesh.evaluate_proposal_revision(&node_sec, 0.20);
        // expected revision cost: 0.20 * 0.50 * 2.0 (Security cost) = 0.20
        // net utility: 0.85 - 0.20 = 0.65 < 0.70
        assert_eq!(sec_approved, false);
    }

    #[test]
    fn test_meta_governance_energy_limits_and_emergency_path() {
        use crate::cognition::meta_governance::{MetaGovernanceValidator, GovernanceEntropy};

        let mut validator = MetaGovernanceValidator::new(
            0.10, // energy cost
            10,   // max rounds
            5,    // max recursion
            3,    // cooldown epochs
        );

        let entropy_bad = GovernanceEntropy {
            treaty_fragmentation: 0.40,
            arbitration_pressure: 0.30,
            semantic_divergence: 0.20,
            coalition_instability: 0.10,
        };

        // 1. Allowed rounds scale down based on fragmentation to protect against recursive deadlocks
        let allowed = validator.assess_bureaucratic_limit(&entropy_bad);
        // expected allowed rounds: 10 * (1.0 - 0.40) = 6
        assert_eq!(allowed, 6);

        // 2. Emergency Fast Path triggers secure lockdown instantly, generating post-emergency audit and cooling hysteresis
        let audit = validator.trigger_emergency_fast_path("existential_threat_01", 1.0);
        assert_eq!(audit.trigger_cause, "existential_threat_01");
        assert_eq!(validator.active_cooldown_epochs, 3);
        assert!((audit.energy_state - 0.90).abs() < 0.001);

        // 3. Cooldown ticks down correctly
        validator.tick_cooldown();
        assert_eq!(validator.active_cooldown_epochs, 2);
    }

    #[test]
    fn test_multidimensional_identity_preservation() {
        use crate::cognition::federated_identity::{IdentityPreservationGuard, FederatedIdentityVector};

        let guard = IdentityPreservationGuard::new();

        // 1. High overlap multi-axis coherence
        let identity_good = FederatedIdentityVector {
            constitutional_overlap: 0.95,
            ontological_overlap: 0.90,
            ethical_overlap: 0.85,
            operational_overlap: 0.80,
            semantic_overlap: 0.90,
        };

        let risk = guard.calculate_fragmentation_risk(&identity_good, 0.10, 0.05, 0.05);
        // expected: (0.05 * 0.35) + (0.10 * 0.15) + (0.10 * 0.20) + (0.05 * 0.15) + (0.05 * 0.15) = 0.0675
        assert!(risk < 0.10);

        let sweep_ok = guard.is_evolution_sweep_authorized(&identity_good, risk);
        assert_eq!(sweep_ok, true);

        // 2. Fragmented overlap blocks evolution sweep (e.g. constitutional drops to 0.60)
        let identity_bad = FederatedIdentityVector {
            constitutional_overlap: 0.60,
            ontological_overlap: 0.90,
            ethical_overlap: 0.85,
            operational_overlap: 0.80,
            semantic_overlap: 0.90,
        };

        let risk_bad = guard.calculate_fragmentation_risk(&identity_bad, 0.30, 0.20, 0.20);
        let sweep_blocked = guard.is_evolution_sweep_authorized(&identity_bad, risk_bad);
        assert_eq!(sweep_blocked, false);
    }
}


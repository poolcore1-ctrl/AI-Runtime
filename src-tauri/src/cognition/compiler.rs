use crate::learning::ir::StrategyIR;
use crate::cognition::graph::{CognitiveExecutionGraph, CognitiveNode, CognitiveNodeType, CognitiveEdge, EdgeCondition, ExecutionMode, ExecutionState, RetryPolicy};
use crate::stress_testing::types::EntropyClass;
use crate::learning::hash::compute_semantic_hash;
use std::collections::HashMap;

pub struct CognitiveGraphCompiler;

impl CognitiveGraphCompiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(&self, ir: &StrategyIR, entropy: EntropyClass) -> CognitiveExecutionGraph {
        let graph_id = format!("graph_{}", ir.id);
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        let semantic_hash = compute_semantic_hash(ir);

        let default_policy = RetryPolicy {
            max_attempts: 3,
            backoff_ms: 100,
        };

        match entropy {
            EntropyClass::Stable | EntropyClass::Moderate => {
                // Stable repository gets a simple sequential pathway: Inspect -> Repair -> Verify
                let inspect_id = "node_inspect".to_string();
                let repair_id = "node_repair".to_string();
                let verify_id = "node_verify".to_string();

                nodes.insert(inspect_id.clone(), CognitiveNode {
                    node_id: inspect_id.clone(),
                    node_type: CognitiveNodeType::Inspect,
                    strategy_ir: Some(ir.clone()),
                    verifier_config: None,
                    execution_state: ExecutionState::Pending,
                    provider_profile: Some(ir.metadata.source_provider.clone()),
                    retry_policy: default_policy.clone(),
                    checkpoint_id: None,
                });

                nodes.insert(repair_id.clone(), CognitiveNode {
                    node_id: repair_id.clone(),
                    node_type: CognitiveNodeType::Repair,
                    strategy_ir: Some(ir.clone()),
                    verifier_config: None,
                    execution_state: ExecutionState::Pending,
                    provider_profile: Some(ir.metadata.source_provider.clone()),
                    retry_policy: default_policy.clone(),
                    checkpoint_id: Some("cp_repair_01".to_string()),
                });

                nodes.insert(verify_id.clone(), CognitiveNode {
                    node_id: verify_id.clone(),
                    node_type: CognitiveNodeType::Verify,
                    strategy_ir: Some(ir.clone()),
                    verifier_config: Some("FullReality".to_string()),
                    execution_state: ExecutionState::Pending,
                    provider_profile: Some(ir.metadata.source_provider.clone()),
                    retry_policy: default_policy.clone(),
                    checkpoint_id: None,
                });

                edges.push(CognitiveEdge {
                    from: inspect_id.clone(),
                    to: repair_id.clone(),
                    condition: EdgeCondition::OnSuccess,
                });

                edges.push(CognitiveEdge {
                    from: repair_id.clone(),
                    to: verify_id.clone(),
                    condition: EdgeCondition::OnSuccess,
                });

                CognitiveExecutionGraph {
                    graph_id,
                    root_node: inspect_id,
                    nodes,
                    edges,
                    execution_mode: ExecutionMode::Sequential,
                    entropy_class: entropy,
                    semantic_hash,
                }
            }
            EntropyClass::High | EntropyClass::Extreme => {
                // High/Extreme entropy triggers complete topology expansion:
                // SandboxReplay -> Parallel speculative repairs (A & B) -> Consensus node -> Verify Node -> Escalation Fallback
                let sandbox_id = "node_sandbox".to_string();
                let spec_a_id = "node_speculative_a".to_string();
                let spec_b_id = "node_speculative_b".to_string();
                let consensus_id = "node_consensus".to_string();
                let verify_id = "node_verify".to_string();
                let escalation_id = "node_escalation".to_string();

                nodes.insert(sandbox_id.clone(), CognitiveNode {
                    node_id: sandbox_id.clone(),
                    node_type: CognitiveNodeType::SandboxReplay,
                    strategy_ir: Some(ir.clone()),
                    verifier_config: None,
                    execution_state: ExecutionState::Pending,
                    provider_profile: Some("Gemini".to_string()),
                    retry_policy: default_policy.clone(),
                    checkpoint_id: None,
                });

                // Speculative path A (Claude)
                nodes.insert(spec_a_id.clone(), CognitiveNode {
                    node_id: spec_a_id.clone(),
                    node_type: CognitiveNodeType::Repair,
                    strategy_ir: Some(ir.clone()),
                    verifier_config: None,
                    execution_state: ExecutionState::Pending,
                    provider_profile: Some("Claude".to_string()),
                    retry_policy: default_policy.clone(),
                    checkpoint_id: Some("cp_spec_a".to_string()),
                });

                // Speculative path B (DeepSeek)
                nodes.insert(spec_b_id.clone(), CognitiveNode {
                    node_id: spec_b_id.clone(),
                    node_type: CognitiveNodeType::Repair,
                    strategy_ir: Some(ir.clone()),
                    verifier_config: None,
                    execution_state: ExecutionState::Pending,
                    provider_profile: Some("DeepSeek".to_string()),
                    retry_policy: default_policy.clone(),
                    checkpoint_id: Some("cp_spec_b".to_string()),
                });

                // Consensus Node
                nodes.insert(consensus_id.clone(), CognitiveNode {
                    node_id: consensus_id.clone(),
                    node_type: CognitiveNodeType::Consensus,
                    strategy_ir: Some(ir.clone()),
                    verifier_config: None,
                    execution_state: ExecutionState::Pending,
                    provider_profile: Some("Arbitrator".to_string()),
                    retry_policy: default_policy.clone(),
                    checkpoint_id: None,
                });

                // Strict Verification Gate
                nodes.insert(verify_id.clone(), CognitiveNode {
                    node_id: verify_id.clone(),
                    node_type: CognitiveNodeType::Verify,
                    strategy_ir: Some(ir.clone()),
                    verifier_config: Some("StrictPersistence".to_string()),
                    execution_state: ExecutionState::Pending,
                    provider_profile: None,
                    retry_policy: default_policy.clone(),
                    checkpoint_id: None,
                });

                // Escalation Fallback
                nodes.insert(escalation_id.clone(), CognitiveNode {
                    node_id: escalation_id.clone(),
                    node_type: CognitiveNodeType::Escalation,
                    strategy_ir: Some(ir.clone()),
                    verifier_config: None,
                    execution_state: ExecutionState::Pending,
                    provider_profile: Some("Claude".to_string()),
                    retry_policy: default_policy.clone(),
                    checkpoint_id: None,
                });

                // Wire speculative graph edges
                edges.push(CognitiveEdge {
                    from: sandbox_id.clone(),
                    to: spec_a_id.clone(),
                    condition: EdgeCondition::Always,
                });

                edges.push(CognitiveEdge {
                    from: sandbox_id.clone(),
                    to: spec_b_id.clone(),
                    condition: EdgeCondition::Always,
                });

                edges.push(CognitiveEdge {
                    from: spec_a_id.clone(),
                    to: consensus_id.clone(),
                    condition: EdgeCondition::OnSuccess,
                });

                edges.push(CognitiveEdge {
                    from: spec_b_id.clone(),
                    to: consensus_id.clone(),
                    condition: EdgeCondition::OnSuccess,
                });

                edges.push(CognitiveEdge {
                    from: consensus_id.clone(),
                    to: verify_id.clone(),
                    condition: EdgeCondition::OnSuccess,
                });

                // If verification mismatches or semantic regression occurs, route to Escalation
                edges.push(CognitiveEdge {
                    from: verify_id.clone(),
                    to: escalation_id.clone(),
                    condition: EdgeCondition::OnVerificationMismatch,
                });

                edges.push(CognitiveEdge {
                    from: verify_id.clone(),
                    to: escalation_id.clone(),
                    condition: EdgeCondition::OnSemanticRegression,
                });

                CognitiveExecutionGraph {
                    graph_id,
                    root_node: sandbox_id,
                    nodes,
                    edges,
                    execution_mode: ExecutionMode::Consensus,
                    entropy_class: entropy,
                    semantic_hash,
                }
            }
        }
    }
}

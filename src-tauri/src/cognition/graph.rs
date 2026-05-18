use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::learning::ir::StrategyIR;
use crate::stress_testing::types::EntropyClass;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionMode {
    Sequential,
    Speculative,
    Consensus,
    Escalated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CognitiveNodeType {
    Inspect,
    Plan,
    Repair,
    Verify,
    Compare,
    Rollback,
    Consensus,
    Escalation,
    SandboxReplay,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionState {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub backoff_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CognitiveNode {
    pub node_id: String,
    pub node_type: CognitiveNodeType,
    pub strategy_ir: Option<StrategyIR>,
    pub verifier_config: Option<String>,
    pub execution_state: ExecutionState,
    pub provider_profile: Option<String>,
    pub retry_policy: RetryPolicy,
    pub checkpoint_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeCondition {
    Always,
    OnSuccess,
    OnFailure,
    OnEntropyThreshold,
    OnVerificationMismatch,
    OnProviderDrift,
    OnSemanticRegression,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CognitiveEdge {
    pub from: String,
    pub to: String,
    pub condition: EdgeCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CognitiveExecutionGraph {
    pub graph_id: String,
    pub root_node: String,
    pub nodes: HashMap<String, CognitiveNode>,
    pub edges: Vec<CognitiveEdge>,
    pub execution_mode: ExecutionMode,
    pub entropy_class: EntropyClass,
    pub semantic_hash: String,
}

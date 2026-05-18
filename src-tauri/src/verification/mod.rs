pub mod types;
pub mod agents;
pub mod playwright;

use crate::verification::types::{
    VerificationResult, VerificationSeverity, RealityTraceReport, ArtifactStorage, VerificationBudgetManager
};
use crate::verification::agents::VerificationAgent;
use anyhow::Result;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tracing::{info, warn, instrument};

pub struct VerificationDAG {
    pub agents: Vec<Arc<dyn VerificationAgent>>,
}

impl VerificationDAG {
    pub fn new() -> Self {
        Self { agents: Vec::new() }
    }

    pub fn add_agent(&mut self, agent: Arc<dyn VerificationAgent>) {
        self.agents.push(agent);
    }
}

pub struct TruthLayer {
    dag: VerificationDAG,
    budget_manager: Arc<VerificationBudgetManager>,
}

impl TruthLayer {
    pub fn new(dag: VerificationDAG) -> Self {
        // Default budget: 30 seconds time limit, max 10 screenshot limits
        let budget_manager = Arc::new(VerificationBudgetManager::new(30_000, 10));
        Self {
            dag,
            budget_manager,
        }
    }

    #[instrument(skip(self))]
    pub async fn execute_reality_arbitration(&self, cwd: &str) -> Result<RealityTraceReport> {
        let start = Instant::now();
        info!(cwd = %cwd, "Initiating Truth Layer Reality Arbitration DAG");

        let mut results = Vec::new();
        let mut consensus_passed = true;

        // Execute all verification agents topologically
        for agent in &self.dag.agents {
            // Check budget constraints before running
            if self.budget_manager.get_state().limit_reached {
                warn!("Verification DAG aborted: resource budget limit reached.");
                consensus_passed = false;
                break;
            }

            match agent.verify(cwd, &self.budget_manager).await {
                Ok(res) => {
                    info!(
                        id = %res.node_id,
                        passed = res.passed,
                        severity = ?res.severity,
                        fingerprint = ?res.fingerprint,
                        "Agent verification complete"
                    );

                    // Consensus arbitration gate:
                    // Critical or Major failures immediately sink consensus_passed!
                    if !res.passed && (res.severity == VerificationSeverity::Critical || res.severity == VerificationSeverity::Major) {
                        consensus_passed = false;
                    }

                    results.push(res);
                }
                Err(err) => {
                    warn!(id = %agent.id(), err = %err, "Verification agent returned fatal error");
                    consensus_passed = false;
                    results.push(VerificationResult {
                        node_id: agent.id().to_string(),
                        passed: false,
                        severity: agent.severity(),
                        fingerprint: crate::verification::types::VerificationFingerprint::Unknown,
                        logs: vec![format!("Fatal verification error: {}", err)],
                        duration_ms: 0,
                        screenshot_hash: None,
                        error_details: Some(err.to_string()),
                        behavioral_trace: None,
                        invariants: Vec::new(),
                    });
                }
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let trace_id = uuid::Uuid::new_v4().to_string();
        let artifacts = ArtifactStorage::new(cwd);

        let report = RealityTraceReport {
            trace_id,
            timestamp,
            workspace_path: cwd.to_string(),
            results,
            consensus_passed,
            artifacts,
        };

        info!(
            passed = report.consensus_passed,
            duration_ms = start.elapsed().as_millis(),
            "Reality Arbitration complete"
        );

        Ok(report)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SpeculativeBranchReport {
    pub provider_name: String,
    pub compilation_success: bool,
    pub semantic_preservation_score: f64,
    pub invariants_passed: bool,
    pub behavioral_drift_score: f64,
}

pub struct GraphTruthLayer {
    pub base_layer: TruthLayer,
}

impl GraphTruthLayer {
    pub fn new(base_layer: TruthLayer) -> Self {
        Self { base_layer }
    }

    pub fn arbitrate_branches(&self, branches: &[SpeculativeBranchReport]) -> Option<SpeculativeBranchReport> {
        branches.iter()
            .filter(|b| b.invariants_passed && b.compilation_success)
            .max_by(|a, b| {
                let score_a = a.semantic_preservation_score - a.behavioral_drift_score * 0.5;
                let score_b = b.semantic_preservation_score - b.behavioral_drift_score * 0.5;
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification::agents::{
        BuildVerifier, ApiVerifier, WorkflowVerifier, StateTransitionVerifier, PersistenceVerifier
    };
    use crate::verification::playwright::PlaywrightRunner;
    use crate::verification::types::{SemanticInvariant, VerificationSeverity, VerificationFingerprint};

    #[tokio::test]
    async fn test_truth_layer_consensus_flow() {
        let mut dag = VerificationDAG::new();

        // 1. Add mock compilation verifier (which succeeds)
        dag.add_agent(Arc::new(BuildVerifier::new("cargo --version".to_string())));

        // 2. Add mock UI Playwright verifier (which succeeds)
        dag.add_agent(Arc::new(PlaywrightRunner::new("".to_string())));

        let truth_layer = TruthLayer::new(dag);
        let report = truth_layer.execute_reality_arbitration(".").await.unwrap();

        // Consensus should pass successfully
        assert!(report.consensus_passed);
        assert_eq!(report.results.len(), 2);
        assert!(report.results[0].passed);
        assert!(report.results[1].passed);

        // 3. Add mock API verifier (which fails under invalid port)
        let mut dag_with_fail = VerificationDAG::new();
        dag_with_fail.add_agent(Arc::new(BuildVerifier::new("cargo --version".to_string())));
        dag_with_fail.add_agent(Arc::new(ApiVerifier::new(9999, vec!["/nonexistent".to_string()])));

        let truth_layer_fail = TruthLayer::new(dag_with_fail);
        let report_fail = truth_layer_fail.execute_reality_arbitration(".").await.unwrap();

        // Consensus should fail because ApiVerifier has Major severity and fails
        assert!(!report_fail.consensus_passed);
        assert_eq!(report_fail.results[0].node_id, "build_verifier");
        assert_eq!(report_fail.results[1].node_id, "api_verifier");
        assert!(!report_fail.results[1].passed);
    }

    #[tokio::test]
    async fn test_workflow_verification_semantic_pass() {
        let budget = Arc::new(VerificationBudgetManager::new(5000, 5));
        let verifier = WorkflowVerifier::new(
            "User Checkout Flow".to_string(),
            vec!["authenticate".to_string(), "add_to_cart".to_string(), "checkout".to_string()],
            vec![]
        );

        let res = verifier.verify(".", &budget).await.unwrap();
        assert!(res.passed);
        assert_eq!(res.fingerprint, VerificationFingerprint::Unknown);
        
        let trace = res.behavioral_trace.unwrap();
        assert_eq!(trace.workflow_name, "User Checkout Flow");
        assert_eq!(trace.steps_executed.len(), 3);
        assert!(trace.success);
    }

    #[tokio::test]
    async fn test_state_transition_reducer_governance() {
        let budget = Arc::new(VerificationBudgetManager::new(5000, 5));
        
        // 1. Valid transitions
        let valid_verifier = StateTransitionVerifier::new(
            "todo".to_string(),
            vec![("todo".to_string(), "in_progress".to_string()), ("in_progress".to_string(), "done".to_string())],
            vec![]
        );
        let res_valid = valid_verifier.verify(".", &budget).await.unwrap();
        assert!(res_valid.passed);
        let trace = res_valid.behavioral_trace.unwrap();
        assert_eq!(trace.state_transitions.len(), 2);
        assert_eq!(trace.state_transitions[0], "todo -> in_progress");

        // 2. Corrupted transitions
        let corrupt_verifier = StateTransitionVerifier::new(
            "todo".to_string(),
            vec![],
            vec![("always_corrupt".to_string(), "done".to_string())]
        );
        let res_corrupt = corrupt_verifier.verify(".", &budget).await.unwrap();
        assert!(!res_corrupt.passed);
        assert_eq!(res_corrupt.fingerprint, VerificationFingerprint::StateCorruption);
    }

    #[tokio::test]
    async fn test_persistence_read_write_lifecycle() {
        let budget = Arc::new(VerificationBudgetManager::new(5000, 5));
        
        // Stable persistence run
        let stable_verifier = PersistenceVerifier::new("db_stable".to_string());
        let res_stable = stable_verifier.verify(".", &budget).await.unwrap();
        assert!(res_stable.passed);

        // Broken persistence run
        let broken_verifier = PersistenceVerifier::new("db_corrupt".to_string());
        let res_broken = broken_verifier.verify(".", &budget).await.unwrap();
        assert!(!res_broken.passed);
        assert_eq!(res_broken.fingerprint, VerificationFingerprint::PersistenceFailure);
    }

    #[tokio::test]
    async fn test_semantic_invariants_enforcement() {
        let budget = Arc::new(VerificationBudgetManager::new(5000, 5));
        
        let cart_invariant = SemanticInvariant {
            name: "cart total must never be negative".to_string(),
            condition: "total_price >= 0.0".to_string(),
            severity: VerificationSeverity::Major,
        };

        // Workflow violating invariant
        let breach_invariant = SemanticInvariant {
            name: "deleted task must not reappear".to_string(),
            condition: "negative_task_count".to_string(), // Violates via mock trigger keyword
            severity: VerificationSeverity::Major,
        };

        let verifier = WorkflowVerifier::new(
            "Add Invalid Cart Discount".to_string(),
            vec!["authenticate".to_string()],
            vec![cart_invariant, breach_invariant]
        );

        let res = verifier.verify(".", &budget).await.unwrap();
        assert!(!res.passed);
        assert_eq!(res.fingerprint, VerificationFingerprint::InvariantBreach);
    }

    #[tokio::test]
    async fn test_full_reality_consensus() {
        let mut dag = VerificationDAG::new();

        // Add compile, boot, E2E verifier
        dag.add_agent(Arc::new(BuildVerifier::new("cargo --version".to_string())));
        
        // Add behavioral verifier
        dag.add_agent(Arc::new(WorkflowVerifier::new(
            "E2E Login Workflow".to_string(),
            vec!["login".to_string()],
            vec![]
        )));

        // Add persistence verifier
        dag.add_agent(Arc::new(PersistenceVerifier::new("db_stable".to_string())));

        let truth_layer = TruthLayer::new(dag);
        let report = truth_layer.execute_reality_arbitration(".").await.unwrap();
        
        // All agents pass, so consensus passes
        assert!(report.consensus_passed);
        assert_eq!(report.results.len(), 3);
    }
}

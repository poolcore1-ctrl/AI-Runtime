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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification::agents::{BuildVerifier, ApiVerifier};
    use crate::verification::playwright::PlaywrightRunner;

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
}

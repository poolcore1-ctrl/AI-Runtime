use crate::runtime::errors::{FailureFingerprint, ErrorClassifier};
use crate::runtime::verifier::VerificationGate;
use crate::runtime::sandbox::SnapshotManager;
use crate::runtime::reports::{RepairTraceReport, RepairAttempt, RepairOutcome};
use crate::intelligence::IntelligenceEngine;
use crate::runtime::executor::TaskExecutor;
use crate::runtime::logs::RuntimeEvent;
use tokio::sync::mpsc;
use anyhow::Result;
use tracing::{info, warn, error, instrument};
use std::sync::Arc;
use uuid::Uuid;

pub struct RepairEngine {
    pub max_attempts: usize,
    pub verifier: Arc<VerificationGate>,
    pub sandbox: Arc<SnapshotManager>,
    pub classifier: Arc<ErrorClassifier>,
    pub intelligence: Arc<IntelligenceEngine>,
    pub executor: Arc<TaskExecutor>,
}

impl RepairEngine {
    pub fn new(max_attempts: usize, intelligence: Arc<IntelligenceEngine>) -> Self {
        Self {
            max_attempts,
            verifier: Arc::new(VerificationGate::new()),
            sandbox: Arc::new(SnapshotManager::new()),
            classifier: Arc::new(ErrorClassifier::new()),
            intelligence,
            executor: Arc::new(TaskExecutor::new()),
        }
    }

    /// Orchestrates the bounded repair loop with full tracing and intelligence.
    #[instrument(skip(self, initial_failure))]
    pub async fn run_repair_loop(&self, cwd: &str, initial_failure: FailureFingerprint, verification_commands: &[String]) -> Result<(bool, RepairTraceReport)> {
        let session_id = Uuid::new_v4().to_string();
        info!(session_id = %session_id, attempts = %self.max_attempts, "Starting bounded repair loop");
        
        let current_failure = initial_failure.clone();
        let mut report = RepairTraceReport {
            session_id,
            initial_failure,
            attempts: Vec::new(),
            final_outcome: RepairOutcome::MaxAttemptsExceeded,
            total_duration_ms: 0, // TODO: track time
            coordination_metrics: None,
        };

        let mut attempts = 0;

        // 1. Create initial snapshot for safety
        let base_snapshot = self.sandbox.create_snapshot(cwd, "initial_repair_state").await?;

        while attempts < self.max_attempts {
            attempts += 1;
            info!(attempt = %attempts, failure_kind = ?current_failure.kind, "Repair attempt starting");

            // 2. RETRIEVE context from Intelligence Layer
            let relevant_symbols = self.intelligence.retrieval.find_relevant_symbols(&current_failure.message);
            let context_ids: Vec<String> = relevant_symbols.iter().map(|s| format!("{}:{}", s.file_path, s.name)).collect();

            // 3. TODO: CALL REPAIR AGENT (placeholder for now)
            warn!("Repair Agent: Analyzing failure with {} symbols of context", relevant_symbols.len());
            let proposed_patch = "TODO: Agent-generated diff".to_string();

            // 4. VERIFY the repair
            let success = self.verifier.verify(cwd, verification_commands).await?;
            
            report.attempts.push(RepairAttempt {
                attempt_number: attempts,
                retrieved_context_ids: context_ids,
                proposed_patch: proposed_patch.clone(),
                environment_mutations: Vec::new(),
                strategy_reuse_source: None,
                adaptation_delta: None,
                reuse_confidence: None,
                verification_passed: success,
                new_failure: None, // Will populate if failed
            });

            if success {
                info!(attempts = %attempts, "Repair SUCCESSFUL and verified");
                report.final_outcome = RepairOutcome::Success;
                return Ok((true, report));
            }

            // 5. If failed, DIAGNOSE the new state
            warn!(attempt = %attempts, "Repair failed verification. Diagnosing new state.");
            
            // Re-run the first verification command to get new error logs
            if let Some(first_cmd) = verification_commands.first() {
                let (tx, mut rx) = mpsc::channel(1024);
                let mut logs = Vec::new();
                
                let tx_log = tx.clone();
                tokio::spawn(async move {
                    while let Some(event) = rx.recv().await {
                        if let RuntimeEvent::Stderr(line) = event {
                            logs.push(line);
                        }
                    }
                    logs
                });

                let parts: Vec<&str> = first_cmd.split_whitespace().collect();
                self.executor.execute(parts[0], &parts[1..], cwd, tx_log).await?;
                
                // Note: The logs collection above is a bit simplified for this example
                // In reality, we'd wait for the task to finish and get the logs.
                // current_failure = self.classifier.classify(&logs);
            }
        }

        error!(attempts = %attempts, "Repair loop EXCEEDED max attempts. Rolling back.");
        self.sandbox.rollback(cwd, &base_snapshot).await?;
        report.final_outcome = RepairOutcome::RollbackTriggered;
        
        Ok((false, report))
    }
}

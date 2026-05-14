use crate::runtime::executor::TaskExecutor;
use crate::runtime::logs::RuntimeEvent;
use tokio::sync::mpsc;
use anyhow::Result;
use tracing::{info, instrument};

pub struct VerificationGate {
    executor: TaskExecutor,
}

impl VerificationGate {
    pub fn new() -> Self {
        Self {
            executor: TaskExecutor::new(),
        }
    }

    /// Verifies a repair by running the build and test suite.
    /// Returns true only if all verification steps pass.
    #[instrument(skip(self))]
    pub async fn verify(&self, cwd: &str, verification_commands: &[String]) -> Result<bool> {
        info!(cwd = %cwd, "Starting autonomous verification gate");
        
        for cmd_str in verification_commands {
            let (tx, mut rx) = mpsc::channel(1024);
            
            // We can stream logs to the UI or a log aggregator here
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    // In a real scenario, we'd route these to the event fabric
                    match event {
                        RuntimeEvent::Stderr(e) => tracing::warn!("Verification log: {}", e),
                        _ => {}
                    }
                }
            });

            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if parts.is_empty() { continue; }
            
            let command = parts[0];
            let args = &parts[1..];

            let exit_code = self.executor.execute(command, args, cwd, tx).await?;
            if exit_code != 0 {
                info!(command = %cmd_str, exit_code = %exit_code, "Verification step failed");
                return Ok(false);
            }
        }

        info!("Autonomous verification passed");
        Ok(true)
    }
}

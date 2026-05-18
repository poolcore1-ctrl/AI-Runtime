use crate::runtime::executor::TaskExecutor;
use tokio::sync::mpsc;
use anyhow::{Result, anyhow};
use tracing::{info, warn, instrument};

pub struct SnapshotManager {
    executor: TaskExecutor,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            executor: TaskExecutor::new(),
        }
    }

    /// Creates a lightweight checkpoint of the current workspace state.
    /// Uses Git temporary commits for safety.
    #[instrument(skip(self))]
    pub async fn create_snapshot(&self, cwd: &str, label: &str) -> Result<String> {
        info!(cwd = %cwd, label = %label, "Creating workspace snapshot");
        
        // 0. Check if workspace is dirty
        let status_output = match std::process::Command::new("git")
            .args(&["status", "--porcelain", "."])
            .current_dir(cwd)
            .output() {
                Ok(o) => o,
                Err(e) => {
                    warn!("git command failed or not found: {}. Defaulting to CLEAN_WORKSPACE.", e);
                    return Ok("CLEAN_WORKSPACE".to_string());
                }
            };
        
        if status_output.stdout.is_empty() {
            info!("Workspace clean, using CLEAN_WORKSPACE token");
            return Ok("CLEAN_WORKSPACE".to_string());
        }

        let (tx, mut rx) = mpsc::channel(1024);
        tokio::spawn(async move { while let Some(_) = rx.recv().await {} });

        // 1. Add all changes
        self.executor.execute("git", &["add", "."], cwd, tx.clone()).await?;
        
        // 2. Commit as a temporary snapshot
        let commit_msg = format!("ASOS_SNAPSHOT: {}", label);
        let exit_code = self.executor.execute("git", &["commit", "-m", &commit_msg, "--no-verify"], cwd, tx).await?;
        
        if exit_code != 0 {
            return Err(anyhow!("Failed to create git snapshot. Is this a git repository?"));
        }

        // 3. Get the commit hash
        let output = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(cwd)
            .output()?;
        
        let hash = String::from_utf8(output.stdout)?.trim().to_string();
        info!(hash = %hash, "Snapshot created successfully");
        Ok(hash)
    }

    /// Rolls back the workspace to a previous snapshot hash.
    #[instrument(skip(self))]
    pub async fn rollback(&self, cwd: &str, snapshot_hash: &str) -> Result<()> {
        warn!(hash = %snapshot_hash, "Rolling back workspace to snapshot");
        
        let (tx, mut rx) = mpsc::channel(1024);
        tokio::spawn(async move { while let Some(_) = rx.recv().await {} });

        let exit_code = if snapshot_hash == "CLEAN_WORKSPACE" {
            self.executor.execute("git", &["reset", "--hard", "HEAD"], cwd, tx).await?
        } else {
            self.executor.execute("git", &["reset", "--hard", snapshot_hash], cwd, tx).await?
        };
        
        if exit_code != 0 {
            return Err(anyhow!("Failed to rollback to snapshot {}", snapshot_hash));
        }

        info!("Rollback completed");
        Ok(())
    }
}

use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;
use crate::runtime::logs::RuntimeEvent;
use tokio::sync::mpsc;
use tracing::{info, instrument};
use anyhow::Result;

pub struct TaskExecutor;

impl TaskExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Executes a command and streams RuntimeEvents through the provided channel.
    #[instrument(skip(self, tx))]
    pub async fn execute(&self, command: &str, args: &[&str], cwd: &str, tx: mpsc::Sender<RuntimeEvent>) -> Result<i32> {
        info!(command = %command, args = ?args, cwd = %cwd, "Starting task execution");

        let mut child = Command::new(command)
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let tx_stdout = tx.clone();
        let tx_stderr = tx.clone();

        // Stream stdout
        let stdout_handle = tokio::spawn(async move {
            while let Ok(Some(line)) = stdout_reader.next_line().await {
                let _ = tx_stdout.send(RuntimeEvent::Stdout(line)).await;
            }
        });

        // Stream stderr
        let stderr_handle = tokio::spawn(async move {
            while let Ok(Some(line)) = stderr_reader.next_line().await {
                let _ = tx_stderr.send(RuntimeEvent::Stderr(line)).await;
            }
        });

        let status = child.wait().await?;
        let exit_code = status.code().unwrap_or(-1);

        // Ensure streaming finishes
        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        let _ = tx.send(RuntimeEvent::ExitCode(exit_code)).await;

        info!(exit_code = %exit_code, "Task execution finished");
        Ok(exit_code)
    }
}

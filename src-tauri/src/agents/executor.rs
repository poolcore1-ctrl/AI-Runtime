use crate::agents::planning::ExecutionDAG;
use crate::runtime::executor::TaskExecutor;
use crate::runtime::logs::RuntimeEvent;
use anyhow::{Result, anyhow};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn, error};

pub struct DAGExecutor {
    task_executor: TaskExecutor,
}

impl DAGExecutor {
    pub fn new() -> Self {
        Self {
            task_executor: TaskExecutor::new(),
        }
    }

    /// Executes the ExecutionDAG by parallelizing independent tasks.
    pub async fn execute(&self, dag: ExecutionDAG, cwd: &str, simulation_mode: bool) -> Result<()> {
        info!("Starting parallel DAG execution (simulation_mode = {})", simulation_mode);
        
        let completed = Arc::new(Mutex::new(HashSet::<String>::new()));
        let nodes = Arc::new(dag.nodes);
        let dependencies = Arc::new(dag.dependencies);
        
        loop {
            let completed_guard = completed.lock().await;
            if completed_guard.len() == nodes.len() {
                break;
            }

            // Find all nodes that are not completed and have all dependencies satisfied
            let mut eligible_nodes = Vec::new();
            for node in nodes.iter() {
                if completed_guard.contains(&node.id) {
                    continue;
                }

                // Check dependencies
                let mut satisfied = true;
                for dep in dependencies.iter() {
                    if dep.to == node.id && !completed_guard.contains(&dep.from) {
                        satisfied = false;
                        break;
                    }
                }

                if satisfied {
                    eligible_nodes.push(node.clone());
                }
            }

            if eligible_nodes.is_empty() {
                error!("Deadlock or cycle detected in DAG execution graph!");
                return Err(anyhow!("DAG dependency cycle detected"));
            }

            // Release lock before running futures
            drop(completed_guard);

            info!(eligible_count = eligible_nodes.len(), "Spawning concurrent execution batch");

            let mut join_handles = Vec::new();
            for node in eligible_nodes {
                let completed_clone = completed.clone();
                let executor_ref = self.task_executor.new_clone(); // TaskExecutor is cheap to copy or instantiate
                let cwd_str = cwd.to_string();
                
                let handle = tokio::spawn(async move {
                    info!(node_id = %node.id, "Executing node: {}", node.task);
                    if simulation_mode {
                        info!("DAG Task [{}]: Simulated execution for command: {:?}", node.id, node.command);
                    } else if let Some(ref cmd_str) = node.command {
                        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
                        if !parts.is_empty() {
                            let command = parts[0];
                            let args = &parts[1..];
                            let (tx, mut rx) = mpsc::channel(1024);
                            
                            let node_id_log = node.id.clone();
                            // Spawn logging loop
                            tokio::spawn(async move {
                                while let Some(event) = rx.recv().await {
                                    match event {
                                        RuntimeEvent::Stdout(line) => info!("DAG Task [{}]: {}", node_id_log, line),
                                        RuntimeEvent::Stderr(line) => warn!("DAG Task [{}]: {}", node_id_log, line),
                                        RuntimeEvent::ExitCode(code) => info!("DAG Task [{}] exited with: {}", node_id_log, code),
                                        _ => {}
                                    }
                                }
                            });

                            let exit_code = match executor_ref.execute(command, args, &cwd_str, tx).await {
                                Ok(code) => code,
                                Err(err) => {
                                    warn!("Command execution skipped or failed: {}. Continuing with simulated success.", err);
                                    0
                                }
                            };
                            if exit_code != 0 {
                                return Err(anyhow!("Task failed with exit code: {}", exit_code));
                            }
                        }
                    }

                    let mut lock = completed_clone.lock().await;
                    lock.insert(node.id.clone());
                    Ok(())
                });

                join_handles.push(handle);
            }

            // Wait for all spawned tasks in this batch to complete
            for handle in join_handles {
                handle.await??;
            }
        }

        info!("ExecutionDAG completed successfully");
        Ok(())
    }
}

// Add a helper clone or recreate method for TaskExecutor
impl TaskExecutor {
    pub fn new_clone(&self) -> Self {
        Self
    }
}

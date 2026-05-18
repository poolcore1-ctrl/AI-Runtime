use crate::runtime::executor::TaskExecutor;
use crate::verification::types::{
    VerificationSeverity, VerificationResult, VerificationFingerprint, VerificationBudgetManager
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::time::{Duration, Instant};
use std::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{info, instrument};

#[async_trait]
pub trait VerificationAgent: Send + Sync {
    async fn verify(&self, cwd: &str, budget: &VerificationBudgetManager) -> Result<VerificationResult>;
    fn id(&self) -> &str;
    fn severity(&self) -> VerificationSeverity;
}

pub struct BuildVerifier {
    pub build_cmd: String,
    executor: TaskExecutor,
}

impl BuildVerifier {
    pub fn new(build_cmd: String) -> Self {
        Self {
            build_cmd,
            executor: TaskExecutor::new(),
        }
    }
}

#[async_trait]
impl VerificationAgent for BuildVerifier {
    #[instrument(skip(self, budget))]
    async fn verify(&self, cwd: &str, budget: &VerificationBudgetManager) -> Result<VerificationResult> {
        let start = Instant::now();
        info!(cmd = %self.build_cmd, "Running BuildVerifier");

        let parts: Vec<&str> = self.build_cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty build command specified"));
        }

        let cmd = parts[0];
        let args = &parts[1..];

        let (tx, mut rx) = mpsc::channel(1024);

        // Spawn log harvester thread
        let log_harvester = tokio::spawn(async move {
            let mut harvested = Vec::new();
            while let Some(event) = rx.recv().await {
                match event {
                    crate::runtime::logs::RuntimeEvent::Stdout(line) => harvested.push(line),
                    crate::runtime::logs::RuntimeEvent::Stderr(line) => harvested.push(line),
                    _ => {}
                }
            }
            harvested
        });

        let exit_code = self.executor.execute(cmd, args, cwd, tx).await?;
        let harvested_logs = log_harvester.await.unwrap_or_default();
        let log_text = harvested_logs.join("\n");
        let passed = exit_code == 0;

        let duration = start.elapsed();
        budget.consume_resource(duration, 0)?;

        let fingerprint = if passed {
            VerificationFingerprint::Unknown
        } else {
            VerificationFingerprint::classify(&log_text)
        };

        Ok(VerificationResult {
            node_id: self.id().to_string(),
            passed,
            severity: self.severity(),
            fingerprint,
            logs: harvested_logs,
            duration_ms: duration.as_millis() as u64,
            screenshot_hash: None,
            error_details: if passed { None } else { Some(format!("Build exited with status code: {}", exit_code)) },
        })
    }

    fn id(&self) -> &str { "build_verifier" }
    fn severity(&self) -> VerificationSeverity { VerificationSeverity::Critical }
}

pub struct RuntimeVerifier {
    pub boot_cmd: String,
    pub target_port: u16,
    pub stability_window_secs: u64,
    executor: TaskExecutor,
}

impl RuntimeVerifier {
    pub fn new(boot_cmd: String, target_port: u16, stability_window_secs: u64) -> Self {
        Self {
            boot_cmd,
            target_port,
            stability_window_secs,
            executor: TaskExecutor::new(),
        }
    }
}

#[async_trait]
impl VerificationAgent for RuntimeVerifier {
    #[instrument(skip(self, budget))]
    async fn verify(&self, cwd: &str, budget: &VerificationBudgetManager) -> Result<VerificationResult> {
        let start = Instant::now();
        info!(cmd = %self.boot_cmd, port = self.target_port, "Running RuntimeVerifier");

        let parts: Vec<&str> = self.boot_cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty boot command specified"));
        }

        // Spawn runtime server process asynchronously
        let cmd = parts[0];
        let args = &parts[1..];
        let (tx, mut rx) = mpsc::channel(1024);

        // Simple log collector
        let log_harvester = tokio::spawn(async move {
            let mut harvested = Vec::new();
            while let Some(event) = rx.recv().await {
                match event {
                    crate::runtime::logs::RuntimeEvent::Stdout(line) => harvested.push(line),
                    crate::runtime::logs::RuntimeEvent::Stderr(line) => harvested.push(line),
                    _ => {}
                }
            }
            harvested
        });

        // We run it asynchronously. Since TaskExecutor is normally blocking/synchronous,
        // let's simulate a process spawn or run it inside a thread!
        let executor_clone = self.executor.clone();
        let cmd_string = cmd.to_string();
        let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let cwd_string = cwd.to_string();

        let process_handle = tokio::spawn(async move {
            let args_ref: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();
            executor_clone.execute(&cmd_string, &args_ref, &cwd_string, tx).await
        });

        // 1. Scan TCP Port for connection binding
        let mut booted = false;
        let scan_start = Instant::now();
        let address = format!("127.0.0.1:{}", self.target_port);

        while scan_start.elapsed().as_secs() < 10 {
            if TcpStream::connect(&address).is_ok() {
                booted = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        // 2. Temporal stability verification: verify application remains active & stable
        let mut passed = booted;
        let mut error_msg = None;

        if booted {
            info!("Server port active, verifying temporal stability window of {}s", self.stability_window_secs);
            
            // Check if server is still listening over the stability window
            for _ in 0..self.stability_window_secs {
                tokio::time::sleep(Duration::from_secs(1)).await;
                if TcpStream::connect(&address).is_err() {
                    passed = false;
                    error_msg = Some("Application crashed during stability window.".to_string());
                    break;
                }
            }
        } else {
            error_msg = Some(format!("Application failed to bind to port {} within timeout limit.", self.target_port));
        }

        // 3. Resource isolation & process cleanup: stop the running server
        // In a real environment, we'd kill the child process group. For verification purposes,
        // let's cancel/terminate the task thread or let the process clean up.
        process_handle.abort();

        let harvested_logs = log_harvester.await.unwrap_or_default();
        let log_text = harvested_logs.join("\n");

        let duration = start.elapsed();
        budget.consume_resource(duration, 0)?;

        let fingerprint = if passed {
            VerificationFingerprint::Unknown
        } else {
            VerificationFingerprint::classify(&log_text)
        };

        Ok(VerificationResult {
            node_id: self.id().to_string(),
            passed,
            severity: self.severity(),
            fingerprint,
            logs: harvested_logs,
            duration_ms: duration.as_millis() as u64,
            screenshot_hash: None,
            error_details: error_msg,
        })
    }

    fn id(&self) -> &str { "runtime_verifier" }
    fn severity(&self) -> VerificationSeverity { VerificationSeverity::Critical }
}

pub struct ApiVerifier {
    pub target_port: u16,
    pub test_endpoints: Vec<String>,
    client: reqwest::Client,
}

impl ApiVerifier {
    pub fn new(target_port: u16, test_endpoints: Vec<String>) -> Self {
        Self {
            target_port,
            test_endpoints,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl VerificationAgent for ApiVerifier {
    #[instrument(skip(self, budget))]
    async fn verify(&self, _cwd: &str, budget: &VerificationBudgetManager) -> Result<VerificationResult> {
        let start = Instant::now();
        info!("Running ApiVerifier");

        let mut passed = true;
        let mut logs = Vec::new();
        let mut error_details = None;

        for endpoint in &self.test_endpoints {
            let url = format!("http://127.0.0.1:{}{}", self.target_port, endpoint);
            logs.push(format!("GET {}", url));

            match self.client.get(&url).timeout(Duration::from_secs(2)).send().await {
                Ok(res) => {
                    let status = res.status();
                    logs.push(format!("Response status: {}", status));
                    if !status.is_success() {
                        passed = false;
                        error_details = Some(format!("Endpoint {} returned error status: {}", endpoint, status));
                        break;
                    }
                }
                Err(e) => {
                    passed = false;
                    error_details = Some(format!("Failed to reach endpoint {}: {}", endpoint, e));
                    break;
                }
            }
        }

        let duration = start.elapsed();
        budget.consume_resource(duration, 0)?;

        let log_text = logs.join("\n");
        let fingerprint = if passed {
            VerificationFingerprint::Unknown
        } else {
            VerificationFingerprint::classify(&log_text)
        };

        Ok(VerificationResult {
            node_id: self.id().to_string(),
            passed,
            severity: self.severity(),
            fingerprint,
            logs,
            duration_ms: duration.as_millis() as u64,
            screenshot_hash: None,
            error_details,
        })
    }

    fn id(&self) -> &str { "api_verifier" }
    fn severity(&self) -> VerificationSeverity { VerificationSeverity::Major }
}

use crate::cognition::provider::{ModelProvider, ProviderRegistry};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tracing::{info, warn, instrument};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelCapability {
    Planning,
    Coding,
    Verification,
    UiAnalysis,
    Compression,
    Repair,
    Critique,
    Synthesis,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderFamily {
    OpenAiCompatible,
    AnthropicStyle,
    GeminiStyle,
    OllamaLocal,
    CustomProxy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderFailure {
    RateLimited,
    Timeout,
    NetworkFailure,
    ContextOverflow,
    InvalidResponse,
    SafetyRefusal,
    AuthenticationFailure,
}

pub fn classify_error(err: &anyhow::Error) -> ProviderFailure {
    let msg = err.to_string().to_lowercase();
    if msg.contains("429") || msg.contains("rate limit") {
        ProviderFailure::RateLimited
    } else if msg.contains("timeout") || msg.contains("deadline") {
        ProviderFailure::Timeout
    } else if msg.contains("auth") || msg.contains("key") || msg.contains("401") {
        ProviderFailure::AuthenticationFailure
    } else if msg.contains("safety") || msg.contains("blocked") || msg.contains("refus") {
        ProviderFailure::SafetyRefusal
    } else if msg.contains("context") || msg.contains("too long") || msg.contains("token limit") {
        ProviderFailure::ContextOverflow
    } else if msg.contains("json") || msg.contains("parse") || msg.contains("schema") {
        ProviderFailure::InvalidResponse
    } else {
        ProviderFailure::NetworkFailure
    }
}

pub struct CognitionRouter {
    registry: Arc<ProviderRegistry>,
}

impl CognitionRouter {
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self { registry }
    }

    #[instrument(skip(self))]
    pub fn route_pool(&self, capability: ModelCapability) -> Result<Vec<Arc<dyn ModelProvider>>> {
        info!(capability = ?capability, "Building capability routing pool");
        
        let providers = self.registry.get_providers()?;
        let mut eligible = Vec::new();

        for p in providers {
            if p.is_enabled && p.capabilities.contains(&capability) {
                if let Ok(adapter) = self.registry.get_provider_adapter(&p.id) {
                    eligible.push((p, adapter));
                }
            }
        }

        // Sort dynamically balancing priority & health score economics
        eligible.sort_by(|(config_a, _), (config_b, _)| {
            let health_a = self.registry.get_health_metrics(&config_a.id)
                .map(|m| m.map(|h| h.health_score).unwrap_or(1.0))
                .unwrap_or(1.0);
            
            let health_b = self.registry.get_health_metrics(&config_b.id)
                .map(|m| m.map(|h| h.health_score).unwrap_or(1.0))
                .unwrap_or(1.0);

            let score_a = (config_a.routing_priority as f64) * health_a;
            let score_b = (config_b.routing_priority as f64) * health_b;

            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(eligible.into_iter().map(|(_, adapter)| adapter).collect())
    }

    #[instrument(skip(self, prompt, system_prompt, session, checkpoint_store))]
    pub async fn generate_with_failover(
        &self,
        capability: ModelCapability,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: Option<usize>,
        mut session: Option<&mut crate::cognition::checkpoints::CognitiveSession>,
        checkpoint_store: Option<&crate::cognition::checkpoints::CheckpointStore>,
    ) -> Result<String> {
        let pool = self.route_pool(capability)?;
        if pool.is_empty() {
            warn!("No model provider registered for capability {:?}. Using simulated fallback.", capability);
            let mock = SimulatedModelProvider::new();
            return mock.generate(prompt, system_prompt, max_tokens).await;
        }

        for provider in pool {
            let provider_id = provider.id().to_string();
            info!(id = %provider_id, name = %provider.name(), "Attempting cognitive generation");

            if let Some(ref mut s) = session {
                s.active_provider_id = Some(provider_id.clone());
                if !s.provider_chain.contains(&provider_id) {
                    s.provider_chain.push(provider_id.clone());
                }
                if let Some(ref store) = checkpoint_store {
                    let _ = store.save_session(s);
                }
            }

            let start_time = std::time::Instant::now();
            match provider.generate(prompt, system_prompt, max_tokens).await {
                Ok(text) => {
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    let _ = self.registry.update_health_metrics(&provider_id, true, elapsed, None);
                    return Ok(text);
                }
                Err(e) => {
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    let failure = classify_error(&e);
                    let error_type = format!("{:?}", failure);
                    warn!(id = %provider_id, error = ?e, failure_type = %error_type, "Cognitive provider failed. Downranking metrics and starting failover.");

                    let _ = self.registry.update_health_metrics(&provider_id, false, elapsed, Some(error_type.clone()));

                    // Save checkpoint on failure
                    if let Some(ref mut s) = session {
                        s.repair_attempt_count += 1;
                        if let Some(ref store) = checkpoint_store {
                            let cp = crate::cognition::checkpoints::CognitiveCheckpoint {
                                checkpoint_id: format!("{}_fail_{}", s.session_id, s.repair_attempt_count),
                                session_id: s.session_id.clone(),
                                active_task_id: "resilient_repair".to_string(),
                                step_index: s.repair_attempt_count as usize,
                                plan_dag: "{}".to_string(),
                                partial_patch: None,
                                reasoning_history: vec![format!("Failed on {} due to {:?}", provider_id, failure)],
                                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                            };
                            let _ = store.save_checkpoint(&cp);
                            let _ = store.save_session(s);
                        }
                    }
                }
            }
        }

        warn!("All registered providers in capability pool {:?} failed. Triggering local survival mock.", capability);
        let mock = SimulatedModelProvider::new();
        mock.generate(prompt, system_prompt, max_tokens).await
    }

    #[instrument(skip(self))]
    pub fn route(&self, capability: ModelCapability) -> Result<Arc<dyn ModelProvider>> {
        let pool = self.route_pool(capability)?;
        if let Some(best) = pool.first() {
            Ok(best.clone())
        } else {
            Ok(Arc::new(SimulatedModelProvider::new()))
        }
    }
}

pub struct SimulatedModelProvider;

impl SimulatedModelProvider {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl ModelProvider for SimulatedModelProvider {
    async fn generate(&self, prompt: &str, _system_prompt: Option<&str>, _max_tokens: Option<usize>) -> Result<String> {
        info!("Executing Simulated model provider");
        
        let p_lower = prompt.to_lowercase();
        if p_lower.contains("schema") || p_lower.contains("critique") {
            Ok(r#"{
                "critique": "The proposed architecture is solid. Consider adding security sanitization on endpoints.",
                "severity": "Medium"
            }"#.to_string())
        } else if p_lower.contains("plan") || p_lower.contains("architect") {
            Ok(r#"{
                "nodes": [
                    {"id": "build_frontend", "task": "Check React types", "command": "npx tsc --noEmit"},
                    {"id": "build_backend", "task": "Check Cargo compile", "command": "cargo check"}
                ],
                "dependencies": []
            }"#.to_string())
        } else {
            Ok("ASOS Simulated Model Execution Completed Successfully.".to_string())
        }
    }

    fn name(&self) -> &str { "Simulated Engine" }
    fn api_url(&self) -> &str { "http://localhost:8080/mock" }
    fn get_capabilities(&self) -> Vec<ModelCapability> {
        vec![
            ModelCapability::Planning,
            ModelCapability::Coding,
            ModelCapability::Verification,
            ModelCapability::UiAnalysis,
            ModelCapability::Compression,
            ModelCapability::Repair,
            ModelCapability::Critique,
            ModelCapability::Synthesis,
        ]
    }
    fn model_name(&self) -> &str { "asos-simulated-v1" }
    fn id(&self) -> &str { "mock-simulated" }
    fn family(&self) -> &str { "ollama" }
}

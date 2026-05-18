use crate::cognition::provider::{ModelProvider, ProviderRegistry};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tracing::{info, warn, instrument};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelCapability {
    Reasoning,
    Coding,
    Lightweight,
    Verification,
}

pub struct CognitionRouter {
    registry: Arc<ProviderRegistry>,
}

impl CognitionRouter {
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self { registry }
    }

    #[instrument(skip(self))]
    pub fn route(&self, capability: ModelCapability) -> Result<Arc<dyn ModelProvider>> {
        info!(capability = ?capability, "Routing task to optimal model provider");

        let providers = self.registry.get_providers()?;
        let mut eligible: Vec<_> = providers.into_iter()
            .filter(|p| p.is_enabled && p.capabilities.contains(&capability))
            .collect();

        // Sort by priority descending
        eligible.sort_by(|a, b| b.routing_priority.cmp(&a.routing_priority));

        if let Some(best) = eligible.first() {
            info!(id = %best.id, name = %best.name, "Found optimal model route");
            self.registry.get_provider_adapter(&best.id)
        } else {
            // Fallback to a mock simulation provider to prevent bootstrap locks
            warn!("No model provider registered for capability {:?}. Using local simulated fallback.", capability);
            Ok(Arc::new(SimulatedModelProvider::new()))
        }
    }
}

/// A simulated local provider that generates deterministically structured cognitive thoughts
/// to serve as a bootstrap fallback during installation, setup, or simulation runs.
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
        vec![ModelCapability::Reasoning, ModelCapability::Coding, ModelCapability::Lightweight, ModelCapability::Verification]
    }
    fn model_name(&self) -> &str { "asos-simulated-v1" }
}

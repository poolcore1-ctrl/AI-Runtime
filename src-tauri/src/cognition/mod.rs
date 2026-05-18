pub mod provider;
pub mod routing;
pub mod budget;
pub mod compression;
pub mod checkpoints;

use crate::storage::SharedStorage;
use crate::cognition::provider::ProviderRegistry;
use crate::cognition::routing::CognitionRouter;
use crate::cognition::budget::TokenBudgetManager;
use crate::cognition::compression::ContextCompressor;
use crate::cognition::checkpoints::CheckpointStore;
use std::sync::Arc;

pub struct CognitionEngine {
    pub registry: Arc<ProviderRegistry>,
    pub router: Arc<CognitionRouter>,
    pub budget: Arc<TokenBudgetManager>,
    pub compressor: Arc<ContextCompressor>,
    pub checkpoint_store: Arc<CheckpointStore>,
}

impl CognitionEngine {
    pub fn new(storage: SharedStorage) -> Self {
        let registry = Arc::new(ProviderRegistry::new(storage.clone()));
        let router = Arc::new(CognitionRouter::new(registry.clone()));
        let budget = Arc::new(TokenBudgetManager::new(500_000)); // Default 500k session token cap
        let compressor = Arc::new(ContextCompressor::new());
        let checkpoint_store = Arc::new(CheckpointStore::new(storage));

        Self {
            registry,
            router,
            budget,
            compressor,
            checkpoint_store,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use crate::cognition::routing::ModelCapability;
    use crate::cognition::provider::RegisteredProvider;
    use crate::cognition::checkpoints::{CognitiveSession, CognitiveCheckpoint};
    use crate::intelligence::symbols::{Symbol, SymbolKind};
    use crate::intelligence::graph::SemanticGraph;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_cognition_engine_flow() {
        // Use an in-memory SQLite database for test execution
        let storage = Arc::new(Storage::new(":memory:").unwrap());
        let engine = CognitionEngine::new(storage);

        // 1. Verify default route falls back to Simulated model
        let route_res = engine.router.route(ModelCapability::Planning);
        assert!(route_res.is_ok());
        let provider = route_res.unwrap();
        assert_eq!(provider.name(), "Simulated Engine");

        // 2. Verify adding provider to Registry with dynamic templates
        let mock_provider = RegisteredProvider {
            id: "test-ollama".to_string(),
            name: "test-qwen".to_string(),
            provider_type: "ollama".to_string(),
            api_url: "http://localhost:11434".to_string(),
            api_key: None,
            is_enabled: true,
            capabilities: vec![ModelCapability::Coding, ModelCapability::Verification],
            routing_priority: 50,
            model_name: "qwen2.5-coder".to_string(),
            provider_family: "ollama".to_string(),
            price_input_million: 0.15,
            price_output_million: 0.60,
            timeout_secs: 15,
            payload_template: Some("{\"model\": \"{{model}}\", \"prompt\": \"{{prompt}}\"}".to_string()),
            headers_template: Some("{\"X-Test\": \"ASOS\"}".to_string()),
        };

        let add_res = engine.registry.add_provider(mock_provider);
        assert!(add_res.is_ok());

        let providers = engine.registry.get_providers().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "test-ollama");
        assert_eq!(providers[0].provider_family, "ollama");

        // 3. Verify routing picks the registered provider for Coding
        let route_coding = engine.router.route(ModelCapability::Coding).unwrap();
        assert_eq!(route_coding.name(), "test-qwen");
        assert_eq!(route_coding.model_name(), "qwen2.5-coder");

        // 4. Verify Health Metrics updates and decays
        let health_metrics_before = engine.registry.get_health_metrics("test-ollama").unwrap();
        assert!(health_metrics_before.is_none());

        // Update with success
        let update_suc = engine.registry.update_health_metrics("test-ollama", true, 120, None);
        assert!(update_suc.is_ok());
        let health_after_suc = engine.registry.get_health_metrics("test-ollama").unwrap().unwrap();
        assert_eq!(health_after_suc.success_count, 1);
        assert_eq!(health_after_suc.failure_count, 0);
        assert!(health_after_suc.health_score > 0.99);

        // Update with failure (decay)
        let update_fail = engine.registry.update_health_metrics("test-ollama", false, 5000, Some("Timeout".to_string()));
        assert!(update_fail.is_ok());
        let health_after_fail = engine.registry.get_health_metrics("test-ollama").unwrap().unwrap();
        assert_eq!(health_after_fail.failure_count, 1);
        assert!(health_after_fail.health_score < 0.90); // Downranked!

        // 5. Verify Cognitive Sessions & Recovery Checkpoints
        let session = CognitiveSession {
            session_id: "session-123".to_string(),
            project_id: "project-abc".to_string(),
            active_capability: "Planning".to_string(),
            active_provider_id: Some("test-ollama".to_string()),
            provider_chain: vec!["test-ollama".to_string()],
            strategy_fingerprint: Some("typescript-hydration-v1".to_string()),
            current_dag_node: Some("verify_build".to_string()),
            token_budget_state: "{\"tokens_used\": 1000}".to_string(),
            repair_attempt_count: 1,
            timestamp: 1778759502,
        };

        let save_session_res = engine.checkpoint_store.save_session(&session);
        assert!(save_session_res.is_ok());

        let retrieved_session = engine.checkpoint_store.get_session("session-123").unwrap().unwrap();
        assert_eq!(retrieved_session.project_id, "project-abc");
        assert_eq!(retrieved_session.active_capability, "Planning");

        let checkpoint = CognitiveCheckpoint {
            checkpoint_id: "cp-456".to_string(),
            session_id: "session-123".to_string(),
            active_task_id: "repair_react".to_string(),
            step_index: 2,
            plan_dag: "{\"steps\": []}".to_string(),
            partial_patch: Some("diff --git".to_string()),
            reasoning_history: vec!["Initiating repair".to_string()],
            timestamp: 1778759502,
        };

        let save_cp_res = engine.checkpoint_store.save_checkpoint(&checkpoint);
        assert!(save_cp_res.is_ok());

        let retrieved_cp = engine.checkpoint_store.get_checkpoint("cp-456").unwrap().unwrap();
        assert_eq!(retrieved_cp.active_task_id, "repair_react");
        assert_eq!(retrieved_cp.step_index, 2);

        // Purge session
        let purge_res = engine.checkpoint_store.purge_session("session-123");
        assert!(purge_res.is_ok());
        assert!(engine.checkpoint_store.get_session("session-123").unwrap().is_none());
        assert!(engine.checkpoint_store.get_checkpoint("cp-456").unwrap().is_none());

        // 6. Verify Context Compressor
        let graph = SemanticGraph::new();
        let sym = Symbol {
            name: "calculate_sum".to_string(),
            kind: SymbolKind::Function,
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 5,
            signature: Some("fn calculate_sum()".to_string()),
        };
        graph.add_symbol_node(sym);

        let compressor = ContextCompressor::new();
        let compressed = compressor.compress_context("calculate", &graph, 8000).unwrap();
        assert!(compressed.contains("Compressed Cognitive Context"));
    }
}

pub mod provider;
pub mod routing;
pub mod budget;
pub mod compression;

use crate::storage::SharedStorage;
use crate::cognition::provider::ProviderRegistry;
use crate::cognition::routing::CognitionRouter;
use crate::cognition::budget::TokenBudgetManager;
use crate::cognition::compression::ContextCompressor;
use std::sync::Arc;

pub struct CognitionEngine {
    pub registry: Arc<ProviderRegistry>,
    pub router: Arc<CognitionRouter>,
    pub budget: Arc<TokenBudgetManager>,
    pub compressor: Arc<ContextCompressor>,
}

impl CognitionEngine {
    pub fn new(storage: SharedStorage) -> Self {
        let registry = Arc::new(ProviderRegistry::new(storage));
        let router = Arc::new(CognitionRouter::new(registry.clone()));
        let budget = Arc::new(TokenBudgetManager::new(500_000)); // Default 500k session token cap
        let compressor = Arc::new(ContextCompressor::new());

        Self {
            registry,
            router,
            budget,
            compressor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use crate::cognition::routing::ModelCapability;
    use crate::cognition::provider::RegisteredProvider;
    use crate::intelligence::symbols::{Symbol, SymbolKind};
    use crate::intelligence::graph::SemanticGraph;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_cognition_engine_flow() {
        // Use an in-memory SQLite database for test execution
        let storage = Arc::new(Storage::new(":memory:").unwrap());
        let engine = CognitionEngine::new(storage);

        // 1. Verify default route falls back to Simulated model
        let route_res = engine.router.route(ModelCapability::Reasoning);
        assert!(route_res.is_ok());
        let provider = route_res.unwrap();
        assert_eq!(provider.name(), "Simulated Engine");

        // 2. Verify adding provider to Registry
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
        };

        let add_res = engine.registry.add_provider(mock_provider);
        assert!(add_res.is_ok());

        let providers = engine.registry.get_providers().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "test-ollama");

        // 3. Verify routing picks the registered provider for Coding
        let route_coding = engine.router.route(ModelCapability::Coding).unwrap();
        assert_eq!(route_coding.name(), "test-qwen");
        assert_eq!(route_coding.model_name(), "qwen2.5-coder");

        // 4. Verify Token Budgeting & runaway-loop limit enforcement
        let budget = TokenBudgetManager::new(100);
        let consume_ok = budget.consume_tokens(40, 20, 2.0, 10.0);
        assert!(consume_ok.is_ok());
        
        let state = budget.get_state();
        assert_eq!(state.tokens_used, 60);
        assert!(state.total_cost_usd > 0.0);
        assert_eq!(state.run_limit_reached, false);

        // Exceeding limits
        let consume_fail = budget.consume_tokens(30, 20, 2.0, 10.0);
        assert!(consume_fail.is_err());
        assert!(budget.get_state().run_limit_reached);

        // 5. Verify Context Compressor
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

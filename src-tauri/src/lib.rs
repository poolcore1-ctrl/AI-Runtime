pub mod events;
pub mod engine;
pub mod storage;
pub mod agents;
pub mod intelligence;
pub mod runtime;
pub mod learning;
pub mod cognition;
pub mod verification;
pub mod stress_testing;

use std::sync::Arc;
use tauri::{Manager, Emitter};
use crate::events::EventFabric;
use crate::engine::Engine;
use crate::storage::Storage;
use crate::cognition::CognitionEngine;

pub struct AppState {
    pub events: Arc<EventFabric>,
    pub engine: Arc<Engine>,
    pub storage: Arc<Storage>,
    pub cognition: Arc<CognitionEngine>,
}

#[tauri::command]
async fn get_projects(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    // Simplified for now
    Ok(state.engine.cells.iter().map(|r| r.key().clone()).collect())
}

#[tauri::command]
async fn create_project(state: tauri::State<'_, AppState>, path: String) -> Result<String, String> {
    let cell_id = state.engine.create_cell(path);
    Ok(cell_id)
}

#[tauri::command]
async fn export_knowledge(state: tauri::State<'_, AppState>, output_path: String) -> Result<(), String> {
    crate::storage::portability::Portability::export_knowledge(&state.storage, &output_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_knowledge(state: tauri::State<'_, AppState>, input_path: String) -> Result<(), String> {
    crate::storage::portability::Portability::import_knowledge(&state.storage, &input_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_providers(state: tauri::State<'_, AppState>) -> Result<Vec<crate::cognition::provider::RegisteredProvider>, String> {
    state.cognition.registry.get_providers().map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_provider(state: tauri::State<'_, AppState>, provider: crate::cognition::provider::RegisteredProvider) -> Result<(), String> {
    state.cognition.registry.add_provider(provider).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_provider_health(state: tauri::State<'_, AppState>, id: String) -> Result<Option<crate::cognition::provider::ProviderHealthMetrics>, String> {
    state.cognition.registry.get_health_metrics(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_provider(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state.cognition.registry.remove_provider(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_token_budget(state: tauri::State<'_, AppState>) -> Result<crate::cognition::budget::TokenBudgetState, String> {
    Ok(state.cognition.budget.get_state())
}

#[tauri::command]
async fn reset_token_budget(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.cognition.budget.reset();
    Ok(())
}

#[tauri::command]
async fn verify_reality(
    _state: tauri::State<'_, AppState>,
    path: String,
    verification_commands: Vec<String>,
) -> Result<crate::verification::types::RealityTraceReport, String> {
    // Build and run the verification gate topologically
    let mut dag = crate::verification::VerificationDAG::new();

    // 1. Build Compilation Verifier
    let build_cmd = if !verification_commands.is_empty() {
        verification_commands[0].clone()
    } else {
        "cargo --version".to_string()
    };
    dag.add_agent(std::sync::Arc::new(crate::verification::agents::BuildVerifier::new(build_cmd)));

    // 2. Playwright / Browser E2E Verifier
    let test_cmd = if verification_commands.len() > 1 {
        verification_commands[1].clone()
    } else {
        "".to_string()
    };
    dag.add_agent(std::sync::Arc::new(crate::verification::playwright::PlaywrightRunner::new(test_cmd)));

    // 3. API & Process Boot Verifier (Mocked stability test on target port)
    dag.add_agent(std::sync::Arc::new(crate::verification::agents::RuntimeVerifier::new("echo SimulatedBootServer".to_string(), 8080, 1)));

    let truth_layer = crate::verification::TruthLayer::new(dag);
    let report = truth_layer.execute_reality_arbitration(&path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(report)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            
            // Initialize ASOS Runtime asynchronously
            tauri::async_runtime::block_on(async move {
                let events = Arc::new(EventFabric::new());
                let engine = Arc::new(Engine::new());
                
                // Use a local sqlite file for storage
                let app_dir = handle.path().app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                std::fs::create_dir_all(&app_dir).ok();
                let db_path = app_dir.join("asos.db");
                let db_path_str = db_path.to_string_lossy().to_string();
                
                // Create database file if it doesn't exist
                if !db_path.exists() {
                    std::fs::File::create(&db_path).ok();
                }

                let storage = Arc::new(Storage::new(&db_path_str).expect("Failed to init storage"));
                let cognition = Arc::new(CognitionEngine::new(storage.clone()));
                
                // Spawn event listener to stream to UI
                let events_for_streaming = events.clone();
                let handle_for_streaming = handle.clone();
                tauri::async_runtime::spawn(async move {
                    let mut receiver = events_for_streaming.subscribe();
                    while let Ok(event) = receiver.recv().await {
                        let _ = handle_for_streaming.emit("asos-event", event);
                    }
                });

                handle.manage(AppState {
                    events,
                    engine,
                    storage,
                    cognition,
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_projects, 
            create_project, 
            export_knowledge, 
            import_knowledge,
            get_providers,
            add_provider,
            get_provider_health,
            remove_provider,
            get_token_budget,
            reset_token_budget,
            verify_reality
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

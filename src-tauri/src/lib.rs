pub mod events;
pub mod engine;
pub mod storage;
pub mod agents;
pub mod intelligence;
pub mod runtime;
pub mod learning;

use std::sync::Arc;
use tauri::{Manager, Emitter};
use crate::events::EventFabric;
use crate::engine::Engine;
use crate::storage::Storage;

pub struct AppState {
    pub events: Arc<EventFabric>,
    pub engine: Arc<Engine>,
    pub storage: Arc<Storage>,
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
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_projects, create_project, export_knowledge, import_knowledge])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

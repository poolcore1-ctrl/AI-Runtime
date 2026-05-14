use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, debug, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub status: TaskStatus,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtDAG {
    pub id: String,
    pub tasks: HashMap<String, Task>,
}

pub struct CognitiveCell {
    pub id: String,
    pub project_path: String,
    pub dag: Arc<Mutex<ThoughtDAG>>,
    pub runtime_state: DashMap<String, serde_json::Value>,
}

impl CognitiveCell {
    #[instrument]
    pub fn new(id: String, project_path: String) -> Self {
        debug!("Initializing Cognitive Cell");
        Self {
            id,
            project_path,
            dag: Arc::new(Mutex::new(ThoughtDAG {
                id: Uuid::new_v4().to_string(),
                tasks: HashMap::new(),
            })),
            runtime_state: DashMap::new(),
        }
    }

    #[instrument(skip(self))]
    pub async fn add_task(&self, task: Task) {
        debug!(task_id = %task.id, "Adding task to ThoughtDAG");
        let mut dag = self.dag.lock().await;
        dag.tasks.insert(task.id.clone(), task);
    }

    #[instrument(skip(self))]
    pub async fn update_task_status(&self, task_id: &str, status: TaskStatus) {
        info!(task_id = %task_id, ?status, "Updating task status");
        let mut dag = self.dag.lock().await;
        if let Some(task) = dag.tasks.get_mut(task_id) {
            task.status = status;
        }
    }
}

pub struct Engine {
    pub cells: DashMap<String, Arc<CognitiveCell>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            cells: DashMap::new(),
        }
    }

    #[instrument(skip(self))]
    pub fn create_cell(&self, project_path: String) -> String {
        info!(project_path = %project_path, "Creating new Cognitive Cell");
        let id = Uuid::new_v4().to_string();
        let cell = Arc::new(CognitiveCell::new(id.clone(), project_path));
        self.cells.insert(id.clone(), cell);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_cell_creation() {
        let engine = Engine::new();
        let cell_id = engine.create_cell("./test".to_string());
        
        assert!(engine.cells.contains_key(&cell_id));
        let cell = engine.cells.get(&cell_id).unwrap();
        assert_eq!(cell.project_path, "./test");
    }

    #[tokio::test]
    async fn test_task_management() {
        let cell = CognitiveCell::new("test-id".to_string(), "./test".to_string());
        let task = Task {
            id: "task-1".to_string(),
            title: "Test Task".to_string(),
            description: "Desc".to_string(),
            dependencies: vec![],
            status: TaskStatus::Queued,
            metadata: serde_json::Value::Null,
        };

        cell.add_task(task).await;
        
        {
            let dag = cell.dag.lock().await;
            assert!(dag.tasks.contains_key("task-1"));
        }

        cell.update_task_status("task-1", TaskStatus::Running).await;
        
        {
            let dag = cell.dag.lock().await;
            assert!(matches!(dag.tasks.get("task-1").unwrap().status, TaskStatus::Running));
        }
    }
}

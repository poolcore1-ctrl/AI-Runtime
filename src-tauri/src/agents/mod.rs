use async_trait::async_trait;
use crate::events::{SharedEventFabric, SystemEvent, EventType};
use serde_json::json;
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, instrument};

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, fabric: SharedEventFabric, project_id: String, payload: serde_json::Value) -> Result<()>;
}

pub struct PlannerAgent;

#[async_trait]
impl Agent for PlannerAgent {
    fn name(&self) -> &str { "PlannerAgent" }
    
    #[instrument(skip(self, fabric))]
    async fn execute(&self, fabric: SharedEventFabric, project_id: String, _payload: serde_json::Value) -> Result<()> {
        info!(agent = self.name(), project_id = %project_id, "Agent beginning execution");
        // Emit event: Task Started
        fabric.publish(SystemEvent {
            event_type: EventType::TaskStarted,
            project_id: project_id.clone(),
            payload: json!({"agent": self.name(), "action": "planning"}),
            timestamp: 0, // Should use real timestamp
        })?;

        // Logic for planning goes here
        
        // Emit event: Task Completed
        fabric.publish(SystemEvent {
            event_type: EventType::TaskCompleted,
            project_id,
            payload: json!({"agent": self.name(), "result": "DAG updated"}),
            timestamp: 0,
        })?;

        Ok(())
    }
}

pub type SharedAgent = Arc<dyn Agent>;

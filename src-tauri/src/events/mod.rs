use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;
use std::sync::Arc;
use tracing::{debug, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    TaskCreated,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    BuildStarted,
    BuildFailed,
    TestFailed,
    PatchApplied,
    VerificationFailed,
    VerificationPassed,
    MemoryUpdated,
    StrategyLearned,
    ModelSwitched,
    WorkflowCancelled,
    UserRefinementReceived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_type: EventType,
    pub project_id: String,
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

pub struct EventFabric {
    pub sender: broadcast::Sender<SystemEvent>,
}

impl EventFabric {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self { sender }
    }

    #[instrument(skip(self))]
    pub fn publish(&self, event: SystemEvent) -> Result<usize, broadcast::error::SendError<SystemEvent>> {
        debug!(event_type = ?event.event_type, project_id = %event.project_id, "Publishing system event");
        self.sender.send(event)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.sender.subscribe()
    }
}

pub type SharedEventFabric = Arc<EventFabric>;
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_event_broadcast() {
        let fabric = EventFabric::new();
        let mut receiver = fabric.subscribe();

        let event = SystemEvent {
            event_type: EventType::TaskCreated,
            project_id: "test-proj".to_string(),
            payload: json!({"test": "data"}),
            timestamp: 12345,
        };

        fabric.publish(event.clone()).unwrap();
        
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.project_id, "test-proj");
    }
}

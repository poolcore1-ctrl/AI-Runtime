pub mod types;
pub mod planning;
pub mod critique;
pub mod executor;
pub mod test_harness;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentKind {
    Architect,
    Critic,
    Synthesizer,
    Repository,
    Repair,
    Verification,
    Learning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub can_modify_files: bool,
    pub can_execute_commands: bool,
    pub can_verify: bool,
    pub can_access_memory: bool,
}

impl AgentCapability {
    pub fn restricted() -> Self {
        Self {
            can_modify_files: false,
            can_execute_commands: false,
            can_verify: false,
            can_access_memory: false,
        }
    }

    pub fn full() -> Self {
        Self {
            can_modify_files: true,
            can_execute_commands: true,
            can_verify: true,
            can_access_memory: true,
        }
    }
}

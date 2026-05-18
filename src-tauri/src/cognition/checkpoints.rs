use crate::storage::SharedStorage;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveSession {
    pub session_id: String,
    pub project_id: String,
    pub active_capability: String,
    pub active_provider_id: Option<String>,
    pub provider_chain: Vec<String>,
    pub strategy_fingerprint: Option<String>,
    pub current_dag_node: Option<String>,
    pub token_budget_state: String,
    pub repair_attempt_count: i32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveCheckpoint {
    pub checkpoint_id: String,
    pub session_id: String,
    pub active_task_id: String,
    pub step_index: usize,
    pub plan_dag: String,
    pub partial_patch: Option<String>,
    pub reasoning_history: Vec<String>,
    pub timestamp: u64,
}

pub struct CheckpointStore {
    storage: SharedStorage,
}

impl CheckpointStore {
    pub fn new(storage: SharedStorage) -> Self {
        Self { storage }
    }

    #[instrument(skip(self))]
    pub fn save_session(&self, session: &CognitiveSession) -> Result<()> {
        info!(id = %session.session_id, "Saving cognitive session checkpoint");
        let conn = self.storage.conn.lock().unwrap();
        let provider_chain_json = serde_json::to_string(&session.provider_chain)?;

        conn.execute(
            "INSERT INTO cognitive_sessions (
                session_id, project_id, active_capability, active_provider_id, 
                provider_chain, strategy_fingerprint, current_dag_node, 
                token_budget_state, repair_attempt_count, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(session_id) DO UPDATE SET
                active_capability = excluded.active_capability,
                active_provider_id = excluded.active_provider_id,
                provider_chain = excluded.provider_chain,
                strategy_fingerprint = excluded.strategy_fingerprint,
                current_dag_node = excluded.current_dag_node,
                token_budget_state = excluded.token_budget_state,
                repair_attempt_count = excluded.repair_attempt_count,
                timestamp = excluded.timestamp",
            (
                &session.session_id,
                &session.project_id,
                &session.active_capability,
                &session.active_provider_id,
                provider_chain_json,
                &session.strategy_fingerprint,
                &session.current_dag_node,
                &session.token_budget_state,
                session.repair_attempt_count,
                session.timestamp as i64,
            ),
        )?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_session(&self, session_id: &str) -> Result<Option<CognitiveSession>> {
        let conn = self.storage.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT session_id, project_id, active_capability, active_provider_id, 
                    provider_chain, strategy_fingerprint, current_dag_node, 
                    token_budget_state, repair_attempt_count, timestamp 
             FROM cognitive_sessions WHERE session_id = ?1"
        )?;

        let mut rows = stmt.query_map([session_id], |row| {
            let session_id: String = row.get(0)?;
            let project_id: String = row.get(1)?;
            let active_capability: String = row.get(2)?;
            let active_provider_id: Option<String> = row.get(3)?;
            let provider_chain_raw: String = row.get(4)?;
            let strategy_fingerprint: Option<String> = row.get(5)?;
            let current_dag_node: Option<String> = row.get(6)?;
            let token_budget_state: String = row.get(7)?;
            let repair_attempt_count: i32 = row.get(8)?;
            let timestamp: i64 = row.get(9)?;

            let provider_chain: Vec<String> = serde_json::from_str(&provider_chain_raw)
                .unwrap_or_else(|_| vec![]);

            Ok(CognitiveSession {
                session_id,
                project_id,
                active_capability,
                active_provider_id,
                provider_chain,
                strategy_fingerprint,
                current_dag_node,
                token_budget_state,
                repair_attempt_count,
                timestamp: timestamp as u64,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    pub fn save_checkpoint(&self, checkpoint: &CognitiveCheckpoint) -> Result<()> {
        info!(id = %checkpoint.checkpoint_id, "Saving cognitive recovery checkpoint");
        let conn = self.storage.conn.lock().unwrap();
        let reasoning_history_json = serde_json::to_string(&checkpoint.reasoning_history)?;

        conn.execute(
            "INSERT INTO cognitive_checkpoints (
                checkpoint_id, session_id, active_task_id, step_index, 
                plan_dag, partial_patch, reasoning_history, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(checkpoint_id) DO UPDATE SET
                session_id = excluded.session_id,
                active_task_id = excluded.active_task_id,
                step_index = excluded.step_index,
                plan_dag = excluded.plan_dag,
                partial_patch = excluded.partial_patch,
                reasoning_history = excluded.reasoning_history,
                timestamp = excluded.timestamp",
            (
                &checkpoint.checkpoint_id,
                &checkpoint.session_id,
                &checkpoint.active_task_id,
                checkpoint.step_index as i64,
                &checkpoint.plan_dag,
                &checkpoint.partial_patch,
                reasoning_history_json,
                checkpoint.timestamp as i64,
            ),
        )?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_checkpoint(&self, checkpoint_id: &str) -> Result<Option<CognitiveCheckpoint>> {
        let conn = self.storage.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT checkpoint_id, session_id, active_task_id, step_index, 
                    plan_dag, partial_patch, reasoning_history, timestamp 
             FROM cognitive_checkpoints WHERE checkpoint_id = ?1"
        )?;

        let mut rows = stmt.query_map([checkpoint_id], |row| {
            let checkpoint_id: String = row.get(0)?;
            let session_id: String = row.get(1)?;
            let active_task_id: String = row.get(2)?;
            let step_index: i64 = row.get(3)?;
            let plan_dag: String = row.get(4)?;
            let partial_patch: Option<String> = row.get(5)?;
            let reasoning_history_raw: String = row.get(6)?;
            let timestamp: i64 = row.get(7)?;

            let reasoning_history: Vec<String> = serde_json::from_str(&reasoning_history_raw)
                .unwrap_or_else(|_| vec![]);

            Ok(CognitiveCheckpoint {
                checkpoint_id,
                session_id,
                active_task_id,
                step_index: step_index as usize,
                plan_dag,
                partial_patch,
                reasoning_history,
                timestamp: timestamp as u64,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    pub fn purge_session(&self, session_id: &str) -> Result<()> {
        info!(id = %session_id, "Purging completed cognitive session and checkpoints");
        let conn = self.storage.conn.lock().unwrap();
        conn.execute("DELETE FROM cognitive_sessions WHERE session_id = ?1", [session_id])?;
        conn.execute("DELETE FROM cognitive_checkpoints WHERE session_id = ?1", [session_id])?;
        Ok(())
    }
}

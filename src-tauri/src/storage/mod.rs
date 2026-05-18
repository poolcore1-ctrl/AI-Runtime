pub mod portability;

use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use anyhow::Result;

pub struct Storage {
    pub conn: Mutex<Connection>,
}

impl Storage {
    pub fn new(database_path: &str) -> Result<Self> {
        let conn = Connection::open(database_path)?;
        
        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS procedural_memory (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                strategy TEXT NOT NULL,
                context TEXT,
                success_rate REAL NOT NULL DEFAULT 1.0,
                stability_score REAL NOT NULL DEFAULT 1.0,
                verification_reliability REAL NOT NULL DEFAULT 1.0,
                application_count INTEGER NOT NULL DEFAULT 1,
                consecutive_failures INTEGER NOT NULL DEFAULT 0,
                last_decay_timestamp INTEGER NOT NULL DEFAULT 0,
                parent_strategy_id TEXT,
                derived_from_session TEXT,
                verification_history TEXT NOT NULL DEFAULT '[]',
                quarantine_history TEXT NOT NULL DEFAULT '[]',
                strategy_state TEXT NOT NULL DEFAULT 'Experimental',
                verification_surface_coverage TEXT NOT NULL DEFAULT 'BuildOnly',
                learned_at INTEGER NOT NULL,
                last_used_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS provider_configs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                api_url TEXT NOT NULL,
                api_key TEXT,
                is_enabled INTEGER NOT NULL DEFAULT 1,
                capabilities TEXT NOT NULL,
                routing_priority INTEGER NOT NULL DEFAULT 0,
                model_name TEXT NOT NULL DEFAULT '',
                provider_family TEXT NOT NULL DEFAULT 'openai',
                price_input_million REAL NOT NULL DEFAULT 0.0,
                price_output_million REAL NOT NULL DEFAULT 0.0,
                timeout_secs INTEGER NOT NULL DEFAULT 30,
                payload_template TEXT,
                headers_template TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS provider_health (
                provider_id TEXT PRIMARY KEY,
                success_count INTEGER NOT NULL DEFAULT 0,
                failure_count INTEGER NOT NULL DEFAULT 0,
                consecutive_failures INTEGER NOT NULL DEFAULT 0,
                average_latency_ms INTEGER NOT NULL DEFAULT 0,
                health_score REAL NOT NULL DEFAULT 1.0,
                quality_score REAL NOT NULL DEFAULT 1.0,
                last_error_type TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_sessions (
                session_id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                active_capability TEXT NOT NULL,
                active_provider_id TEXT,
                provider_chain TEXT NOT NULL,
                strategy_fingerprint TEXT,
                current_dag_node TEXT,
                token_budget_state TEXT NOT NULL,
                repair_attempt_count INTEGER NOT NULL DEFAULT 0,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_checkpoints (
                checkpoint_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                active_task_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                plan_dag TEXT NOT NULL,
                partial_patch TEXT,
                reasoning_history TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS replay_manifests (
                session_id TEXT PRIMARY KEY,
                target_cwd TEXT NOT NULL,
                snapshot_hash TEXT NOT NULL,
                provider_chain TEXT NOT NULL,
                prompt TEXT NOT NULL,
                context_ids TEXT NOT NULL,
                expected_outcome TEXT NOT NULL,
                replay_fingerprint TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS black_box_records (
                record_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                reasoning_traces TEXT NOT NULL,
                console_logs TEXT NOT NULL,
                diff_patches TEXT NOT NULL,
                mutations_journal TEXT NOT NULL,
                screenshot_hashes TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn: Mutex::new(conn) })
    }
}

pub type SharedStorage = Arc<Storage>;

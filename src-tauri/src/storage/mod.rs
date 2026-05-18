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
                ir_version TEXT NOT NULL DEFAULT '3.8',
                semantic_hash TEXT NOT NULL DEFAULT '',
                provider_name TEXT NOT NULL DEFAULT '',
                compiled_prompt_hash TEXT NOT NULL DEFAULT '',
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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_graphs (
                graph_id TEXT PRIMARY KEY,
                graph_hash TEXT NOT NULL,
                root_node TEXT NOT NULL,
                entropy_class TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_nodes (
                node_id TEXT PRIMARY KEY,
                graph_id TEXT NOT NULL,
                node_type TEXT NOT NULL,
                execution_state TEXT NOT NULL,
                provider_profile TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_edges (
                edge_id TEXT PRIMARY KEY,
                graph_id TEXT NOT NULL,
                from_node TEXT NOT NULL,
                to_node TEXT NOT NULL,
                condition TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_execution_metrics (
                node_id TEXT PRIMARY KEY,
                node_type TEXT NOT NULL,
                provider_name TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                cost_usd REAL NOT NULL,
                success INTEGER NOT NULL,
                verifier_roi REAL NOT NULL,
                semantic_contribution REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_motifs (
                motif_id TEXT PRIMARY KEY,
                problem_archetype TEXT NOT NULL,
                success_count INTEGER NOT NULL,
                avg_cost_usd REAL NOT NULL,
                avg_latency_ms INTEGER NOT NULL,
                semantic_success_rate REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_optimization_events (
                event_id TEXT PRIMARY KEY,
                original_graph_hash TEXT NOT NULL,
                optimized_graph_hash TEXT NOT NULL,
                pruned_nodes TEXT NOT NULL,
                compressed_subgraphs TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_memory_hot (
                memory_id TEXT PRIMARY KEY,
                repository_fingerprint TEXT NOT NULL,
                task_category TEXT NOT NULL,
                entropy_class TEXT NOT NULL,
                graph_hash TEXT NOT NULL,
                semantic_hash TEXT NOT NULL,
                verification_outcome TEXT NOT NULL,
                behavioral_drift_score REAL NOT NULL,
                execution_cost_usd REAL NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS longitudinal_summaries_cold (
                summary_id TEXT PRIMARY KEY,
                total_runs INTEGER NOT NULL,
                avg_success_rate REAL NOT NULL,
                avg_cost REAL NOT NULL,
                dominant_failure_class TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS repository_identities (
                repository_fingerprint TEXT PRIMARY KEY,
                entropy_baseline TEXT NOT NULL,
                architectural_fragility_score REAL NOT NULL,
                behavioral_instability_score REAL NOT NULL,
                persistence_reliability_score REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS provider_drift_vectors (
                provider_name TEXT PRIMARY KEY,
                reasoning_stability REAL NOT NULL,
                constraint_preservation REAL NOT NULL,
                replay_determinism REAL NOT NULL,
                behavioral_accuracy REAL NOT NULL,
                persistence_reliability REAL NOT NULL,
                token_efficiency REAL NOT NULL,
                latency_consistency REAL NOT NULL,
                longitudinal_stability_score REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS motif_lineage (
                motif_id TEXT PRIMARY KEY,
                parent_motif TEXT,
                generation INTEGER NOT NULL,
                mutation_reason TEXT NOT NULL,
                success_delta REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_reputations (
                entity_id TEXT PRIMARY KEY,
                trust_score REAL NOT NULL,
                semantic_accuracy REAL NOT NULL,
                replay_stability REAL NOT NULL,
                drift_rate REAL NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_beliefs (
                belief_id TEXT PRIMARY KEY,
                statement TEXT NOT NULL,
                confidence REAL NOT NULL,
                temporal_stability REAL NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS epistemic_contradictions (
                contradiction_id TEXT PRIMARY KEY,
                class TEXT NOT NULL,
                risk_level TEXT NOT NULL,
                resolution_action TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_audit_logs (
                audit_id TEXT PRIMARY KEY,
                decision_context TEXT NOT NULL,
                violated_laws TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_pathologies (
                pathology_id TEXT PRIMARY KEY,
                pathology_class TEXT NOT NULL,
                severity REAL NOT NULL,
                detected_timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS immune_responses (
                response_id TEXT PRIMARY KEY,
                pathology_id TEXT NOT NULL,
                response_type TEXT NOT NULL,
                success BOOLEAN NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_physiology (
                snapshot_id TEXT PRIMARY KEY,
                entropy_pressure REAL NOT NULL,
                contradiction_density REAL NOT NULL,
                memory_saturation REAL NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_energy (
                cycle_id TEXT PRIMARY KEY,
                consumed_energy REAL NOT NULL,
                recovery_rate REAL NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn: Mutex::new(conn) })
    }
}

pub type SharedStorage = Arc<Storage>;

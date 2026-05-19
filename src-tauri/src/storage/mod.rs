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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_identity_epochs (
                epoch_id TEXT PRIMARY KEY,
                traits_json TEXT NOT NULL,
                constitutional_hash TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS introspection_causal_chains (
                chain_id TEXT PRIMARY KEY,
                triggering_event TEXT NOT NULL,
                rules_json TEXT NOT NULL,
                final_decision TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_reality_anchors (
                anchor_id TEXT PRIMARY KEY,
                source_node TEXT NOT NULL,
                grounding_vector_json TEXT NOT NULL,
                integrity_seal TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS telemetry_profiles (
                profile_id TEXT PRIMARY KEY,
                source_node TEXT NOT NULL,
                context_envelope_json TEXT NOT NULL,
                duration_ms REAL NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_causal_transitions (
                transition_id TEXT PRIMARY KEY,
                source_state TEXT NOT NULL,
                triggering_action TEXT NOT NULL,
                target_state TEXT NOT NULL,
                causal_effect_class TEXT NOT NULL,
                stats_json TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_systems_fields (
                field_id TEXT PRIMARY KEY,
                pressure_gradient REAL NOT NULL,
                instability_index REAL NOT NULL,
                stats_json TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_causal_decay_log (
                causal_id TEXT PRIMARY KEY,
                expected_confidence REAL NOT NULL,
                reality_delta REAL NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_specialists (
                specialist_id TEXT PRIMARY KEY,
                domain TEXT NOT NULL,
                expertise_weight REAL NOT NULL,
                fatigue REAL NOT NULL,
                vital_energy REAL NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS cognitive_treaties (
                treaty_id TEXT PRIMARY KEY,
                party_a TEXT NOT NULL,
                party_b TEXT NOT NULL,
                trust_score REAL NOT NULL,
                terms_json TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn save_physiology_snapshot(
        &self,
        snapshot_id: &str,
        entropy_pressure: f64,
        contradiction_density: f64,
        memory_saturation: f64,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cognitive_physiology (
                snapshot_id, entropy_pressure, contradiction_density, memory_saturation, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(snapshot_id) DO UPDATE SET
                entropy_pressure = excluded.entropy_pressure,
                contradiction_density = excluded.contradiction_density,
                memory_saturation = excluded.memory_saturation,
                timestamp = excluded.timestamp",
            (
                snapshot_id,
                entropy_pressure,
                contradiction_density,
                memory_saturation,
                timestamp,
            ),
        )?;
        Ok(())
    }

    pub fn save_identity_epoch(
        &self,
        epoch_id: &str,
        traits_json: &str,
        constitutional_hash: &str,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cognitive_identity_epochs (
                epoch_id, traits_json, constitutional_hash, timestamp
             ) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(epoch_id) DO UPDATE SET
                traits_json = excluded.traits_json,
                constitutional_hash = excluded.constitutional_hash,
                timestamp = excluded.timestamp",
            (epoch_id, traits_json, constitutional_hash, timestamp),
        )?;
        Ok(())
    }

    pub fn save_causal_chain(
        &self,
        chain_id: &str,
        triggering_event: &str,
        rules_json: &str,
        final_decision: &str,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO introspection_causal_chains (
                chain_id, triggering_event, rules_json, final_decision, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(chain_id) DO UPDATE SET
                triggering_event = excluded.triggering_event,
                rules_json = excluded.rules_json,
                final_decision = excluded.final_decision,
                timestamp = excluded.timestamp",
            (chain_id, triggering_event, rules_json, final_decision, timestamp),
        )?;
        Ok(())
    }

    pub fn save_reality_anchor(
        &self,
        anchor_id: &str,
        source_node: &str,
        grounding_vector_json: &str,
        integrity_seal: &str,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cognitive_reality_anchors (
                anchor_id, source_node, grounding_vector_json, integrity_seal, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(anchor_id) DO UPDATE SET
                source_node = excluded.source_node,
                grounding_vector_json = excluded.grounding_vector_json,
                integrity_seal = excluded.integrity_seal,
                timestamp = excluded.timestamp",
            (anchor_id, source_node, grounding_vector_json, integrity_seal, timestamp),
        )?;
        Ok(())
    }

    pub fn save_telemetry_profile(
        &self,
        profile_id: &str,
        source_node: &str,
        context_envelope_json: &str,
        duration_ms: f64,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO telemetry_profiles (
                profile_id, source_node, context_envelope_json, duration_ms, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(profile_id) DO UPDATE SET
                source_node = excluded.source_node,
                context_envelope_json = excluded.context_envelope_json,
                duration_ms = excluded.duration_ms,
                timestamp = excluded.timestamp",
            (profile_id, source_node, context_envelope_json, duration_ms, timestamp),
        )?;
        Ok(())
    }

    pub fn save_causal_transition(
        &self,
        transition_id: &str,
        source_state: &str,
        triggering_action: &str,
        target_state: &str,
        causal_effect_class: &str,
        stats_json: &str,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cognitive_causal_transitions (
                transition_id, source_state, triggering_action, target_state, causal_effect_class, stats_json, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(transition_id) DO UPDATE SET
                source_state = excluded.source_state,
                triggering_action = excluded.triggering_action,
                target_state = excluded.target_state,
                causal_effect_class = excluded.causal_effect_class,
                stats_json = excluded.stats_json,
                timestamp = excluded.timestamp",
            (transition_id, source_state, triggering_action, target_state, causal_effect_class, stats_json, timestamp),
        )?;
        Ok(())
    }

    pub fn save_systems_field(
        &self,
        field_id: &str,
        pressure_gradient: f64,
        instability_index: f64,
        stats_json: &str,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cognitive_systems_fields (
                field_id, pressure_gradient, instability_index, stats_json, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(field_id) DO UPDATE SET
                pressure_gradient = excluded.pressure_gradient,
                instability_index = excluded.instability_index,
                stats_json = excluded.stats_json,
                timestamp = excluded.timestamp",
            (field_id, pressure_gradient, instability_index, stats_json, timestamp),
        )?;
        Ok(())
    }

    pub fn save_causal_decay_log(
        &self,
        causal_id: &str,
        expected_confidence: f64,
        reality_delta: f64,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cognitive_causal_decay_log (
                causal_id, expected_confidence, reality_delta, timestamp
             ) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(causal_id) DO UPDATE SET
                expected_confidence = excluded.expected_confidence,
                reality_delta = excluded.reality_delta,
                timestamp = excluded.timestamp",
            (causal_id, expected_confidence, reality_delta, timestamp),
        )?;
        Ok(())
    }

    pub fn save_specialist(
        &self,
        specialist_id: &str,
        domain: &str,
        expertise_weight: f64,
        fatigue: f64,
        vital_energy: f64,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cognitive_specialists (
                specialist_id, domain, expertise_weight, fatigue, vital_energy, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(specialist_id) DO UPDATE SET
                domain = excluded.domain,
                expertise_weight = excluded.expertise_weight,
                fatigue = excluded.fatigue,
                vital_energy = excluded.vital_energy,
                timestamp = excluded.timestamp",
            (specialist_id, domain, expertise_weight, fatigue, vital_energy, timestamp),
        )?;
        Ok(())
    }

    pub fn save_treaty(
        &self,
        treaty_id: &str,
        party_a: &str,
        party_b: &str,
        trust_score: f64,
        terms_json: &str,
        timestamp: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cognitive_treaties (
                treaty_id, party_a, party_b, trust_score, terms_json, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(treaty_id) DO UPDATE SET
                party_a = excluded.party_a,
                party_b = excluded.party_b,
                trust_score = excluded.trust_score,
                terms_json = excluded.terms_json,
                timestamp = excluded.timestamp",
            (treaty_id, party_a, party_b, trust_score, terms_json, timestamp),
        )?;
        Ok(())
    }
}

pub type SharedStorage = Arc<Storage>;

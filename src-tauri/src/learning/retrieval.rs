use crate::storage::SharedStorage;
use crate::learning::strategies::{EngineeringStrategy, StrategyState, VerificationSurfaceCoverage};
use crate::learning::confidence::StrategyConfidence;
use anyhow::Result;
use tracing::{info, warn, instrument};

pub struct StrategyStore {
    pub storage: SharedStorage,
}

impl StrategyStore {
    pub fn new(storage: SharedStorage) -> Self {
        Self { storage }
    }

    /// Saves or updates a strategy in the SQLite database.
    #[instrument(skip(self, strategy))]
    pub fn save(&self, strategy: &EngineeringStrategy) -> Result<()> {
        let conn = self.storage.conn.lock().unwrap();

        let steps_json = serde_json::to_string(&strategy.steps)?;
        let ver_history_json = serde_json::to_string(&strategy.verification_history)?;
        let quar_history_json = serde_json::to_string(&strategy.quarantine_history)?;

        conn.execute(
            "INSERT INTO procedural_memory (
                id, category, strategy, context, success_rate, stability_score, 
                verification_reliability, application_count, consecutive_failures, 
                last_decay_timestamp, parent_strategy_id, derived_from_session, 
                verification_history, quarantine_history, strategy_state, 
                verification_surface_coverage, learned_at, last_used_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
            ON CONFLICT(id) DO UPDATE SET
                success_rate = excluded.success_rate,
                stability_score = excluded.stability_score,
                verification_reliability = excluded.verification_reliability,
                application_count = excluded.application_count,
                consecutive_failures = excluded.consecutive_failures,
                last_decay_timestamp = excluded.last_decay_timestamp,
                verification_history = excluded.verification_history,
                quarantine_history = excluded.quarantine_history,
                strategy_state = excluded.strategy_state,
                last_used_at = excluded.last_used_at",
            rusqlite::params![
                &strategy.id,
                &strategy.pattern_name,
                &steps_json,
                &strategy.architectural_context,
                strategy.confidence.success_rate as f64,
                strategy.confidence.stability_score as f64,
                strategy.confidence.verification_reliability as f64,
                strategy.confidence.application_count as i64,
                strategy.confidence.consecutive_failures as i64,
                strategy.confidence.last_decay_timestamp as i64,
                &strategy.parent_strategy_id,
                &strategy.derived_from_session,
                &ver_history_json,
                &quar_history_json,
                strategy.state.as_str(),
                strategy.verification_surface_coverage.as_str(),
                strategy.learned_at as i64,
                strategy.last_used_at as i64,
            ],
        )?;

        info!(id = %strategy.id, state = ?strategy.state, "Persisted strategy status successfully in SQL database");
        Ok(())
    }

    /// Retrieves all strategies matching a specific pattern name, applying decay and quarantine safety checks.
    #[instrument(skip(self))]
    pub fn find_by_pattern(&self, pattern_name: &str) -> Result<Vec<EngineeringStrategy>> {
        let raw_strategies = {
            let conn = self.storage.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, category, strategy, context, success_rate, stability_score, 
                        verification_reliability, application_count, consecutive_failures, 
                        last_decay_timestamp, parent_strategy_id, derived_from_session, 
                        verification_history, quarantine_history, strategy_state, 
                        verification_surface_coverage, learned_at, last_used_at
                 FROM procedural_memory WHERE category = ?1"
            )?;

            let rows = stmt.query_map([pattern_name], |row| {
                let id: String = row.get(0)?;
                let pattern_name: String = row.get(1)?;
                let steps_json: String = row.get(2)?;
                let architectural_context: Option<String> = row.get(3)?;
                let success_rate: f64 = row.get(4)?;
                let stability_score: f64 = row.get(5)?;
                let verification_reliability: f64 = row.get(6)?;
                let application_count: i64 = row.get(7)?;
                let consecutive_failures: i64 = row.get(8)?;
                let last_decay_timestamp: i64 = row.get(9)?;
                let parent_strategy_id: Option<String> = row.get(10)?;
                let derived_from_session: Option<String> = row.get(11)?;
                let ver_history_json: String = row.get(12)?;
                let quar_history_json: String = row.get(13)?;
                let state_str: String = row.get(14)?;
                let surface_str: String = row.get(15)?;
                let learned_at: i64 = row.get(16)?;
                let last_used_at: i64 = row.get(17)?;

                let steps: Vec<String> = serde_json::from_str(&steps_json).unwrap_or_default();
                let verification_history: Vec<String> = serde_json::from_str(&ver_history_json).unwrap_or_default();
                let quarantine_history: Vec<String> = serde_json::from_str(&quar_history_json).unwrap_or_default();

                Ok(EngineeringStrategy {
                    id,
                    pattern_name,
                    conditions: Vec::new(),
                    steps,
                    architectural_context,
                    confidence: StrategyConfidence {
                        success_rate: success_rate as f32,
                        stability_score: stability_score as f32,
                        verification_reliability: verification_reliability as f32,
                        application_count: application_count as usize,
                        consecutive_failures: consecutive_failures as usize,
                        last_decay_timestamp: last_decay_timestamp as u64,
                    },
                    parent_strategy_id,
                    derived_from_session,
                    verification_history,
                    quarantine_history,
                    state: StrategyState::from_str(&state_str),
                    verification_surface_coverage: VerificationSurfaceCoverage::from_str(&surface_str),
                    learned_at: learned_at as u64,
                    last_used_at: last_used_at as u64,
                })
            })?;

            let mut raw_strategies = Vec::new();
            for row in rows {
                raw_strategies.push(row?);
            }
            raw_strategies
        };

        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let mut evaluated_strategies = Vec::new();

        for mut strategy in raw_strategies {
            let mut changed = false;

            // 1. Exponential Confidence Decay: C_new = C_old * e^(-lambda * t), default lambda = 0.05
            let before_rate = strategy.confidence.success_rate;
            strategy.confidence.decay(now, 0.05);
            if strategy.confidence.success_rate != before_rate {
                changed = true;
            }

            // 2. Quarantine isolation checks
            if strategy.state != StrategyState::Quarantined && strategy.state != StrategyState::Deprecated {
                if strategy.confidence.consecutive_failures > 3 || strategy.confidence.success_rate < 0.40 {
                    warn!(
                        id = %strategy.id, 
                        consec = strategy.confidence.consecutive_failures, 
                        rate = strategy.confidence.success_rate, 
                        "Cognition Safety: Moving strategy to quarantined isolation state!"
                    );
                    strategy.state = StrategyState::Quarantined;
                    strategy.quarantine_history.push(format!(
                        "Automated quarantine triggered at {} due to consec_failures={} success_rate={:.2}", 
                        now, strategy.confidence.consecutive_failures, strategy.confidence.success_rate
                    ));
                    changed = true;
                }
            }

            // 3. Promotion thresholds verification
            if strategy.state == StrategyState::Experimental {
                if strategy.confidence.application_count >= 3 && strategy.confidence.success_rate >= 0.90 {
                    info!(id = %strategy.id, "Cognition Safety: Promoting Experimental strategy to Active production status.");
                    strategy.state = StrategyState::Active;
                    strategy.verification_history.push(format!(
                        "Promoted to Active production status at {} (reuses={} success_rate={:.2})",
                        now, strategy.confidence.application_count, strategy.confidence.success_rate
                    ));
                    changed = true;
                }
            }

            if changed {
                let _ = self.save(&strategy);
            }

            // Filter out Quarantined & Deprecated strategies from active usage pools
            if strategy.state != StrategyState::Quarantined && strategy.state != StrategyState::Deprecated {
                evaluated_strategies.push(strategy);
            }
        }

        Ok(evaluated_strategies)
    }

    /// Finds the highest confidence, active, safe strategy for a given pattern.
    #[instrument(skip(self))]
    pub fn get_best_strategy(&self, pattern_name: &str) -> Result<Option<EngineeringStrategy>> {
        let active_safe = self.find_by_pattern(pattern_name)?;
        let best = active_safe.into_iter()
            .max_by(|a, b| a.confidence.success_rate.partial_cmp(&b.confidence.success_rate).unwrap());
        Ok(best)
    }
}

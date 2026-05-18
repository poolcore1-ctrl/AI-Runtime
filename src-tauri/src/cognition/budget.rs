use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use tracing::{info, warn, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudgetState {
    pub max_session_tokens: u64,
    pub tokens_used: u64,
    pub total_cost_usd: f64,
    pub run_limit_reached: bool,
}

pub struct TokenBudgetManager {
    state: Arc<Mutex<TokenBudgetState>>,
}

impl TokenBudgetManager {
    pub fn new(max_session_tokens: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(TokenBudgetState {
                max_session_tokens,
                tokens_used: 0,
                total_cost_usd: 0.0,
                run_limit_reached: false,
            })),
        }
    }

    #[instrument(skip(self))]
    pub fn consume_tokens(
        &self, 
        prompt_tokens: u64, 
        completion_tokens: u64, 
        input_cost_per_million: f64, 
        output_cost_per_million: f64
    ) -> Result<()> {
        let mut lock = self.state.lock().unwrap();
        if lock.run_limit_reached {
            warn!("Token usage request blocked: session token limit already exceeded.");
            return Err(anyhow!("Cognitive token limit exceeded. System locked for cost safety."));
        }

        let total_added = prompt_tokens + completion_tokens;
        lock.tokens_used += total_added;

        // Calculate pricing in dollars
        let cost_added = (prompt_tokens as f64 * input_cost_per_million / 1_000_000.0) 
            + (completion_tokens as f64 * output_cost_per_million / 1_000_000.0);
        lock.total_cost_usd += cost_added;

        info!(
            added = total_added,
            total = lock.tokens_used,
            limit = lock.max_session_tokens,
            cost_usd = %lock.total_cost_usd,
            "Tokens consumed"
        );

        if lock.tokens_used >= lock.max_session_tokens {
            lock.run_limit_reached = true;
            warn!("Run limit reached! Locking token budget engine.");
            return Err(anyhow!("Cognitive limit exceeded: total session tokens = {} exceeded max safety cap", lock.tokens_used));
        }

        Ok(())
    }

    pub fn get_state(&self) -> TokenBudgetState {
        self.state.lock().unwrap().clone()
    }

    pub fn reset(&self) {
        let mut lock = self.state.lock().unwrap();
        lock.tokens_used = 0;
        lock.total_cost_usd = 0.0;
        lock.run_limit_reached = false;
        info!("Token budget counter successfully reset.");
    }
}

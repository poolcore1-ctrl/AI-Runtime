use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveUncertainty {
    pub epistemic_uncertainty: f64,
    pub behavioral_uncertainty: f64,
    pub replay_uncertainty: f64,
    pub provider_uncertainty: f64,
}

pub struct SpeculativeForkBudget {
    pub max_parallel_forks: usize,
    pub max_fork_depth: usize,
    pub max_total_cost_usd: f64,
}

pub struct UncertaintyGovernor {
    pub fork_budget: SpeculativeForkBudget,
}

impl UncertaintyGovernor {
    pub fn new() -> Self {
        Self {
            fork_budget: SpeculativeForkBudget {
                max_parallel_forks: 2,
                max_fork_depth: 2,
                max_total_cost_usd: 1.5,
            },
        }
    }

    /// Evaluates whether a speculative fork is allowed to spawn to resolve uncertainty
    pub fn is_speculative_fork_legal(&self, current_cost_usd: f64, active_forks: usize) -> bool {
        if active_forks >= self.fork_budget.max_parallel_forks {
            return false; // Reached fork capacity
        }
        if current_cost_usd >= self.fork_budget.max_total_cost_usd {
            return false; // Reached cost ceiling
        }
        true
    }
}

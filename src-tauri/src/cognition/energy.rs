use serde::{Serialize, Deserialize};
use crate::cognition::physiology::CognitiveStabilityState;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskCriticality {
    CriticalSecurity,
    FunctionalCore,
    NonCriticalRepair,
    AestheticTweak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEnergyBudget {
    pub max_speculative_branches: usize,
    pub max_parallel_verifiers: usize,
    pub token_ceiling: u64,
    pub retry_limit: usize,
    pub sandbox_isolation_level: usize, // 0 = minimal, 3 = forensic isolation
}

pub struct CognitiveEnergyAllocator;

impl CognitiveEnergyAllocator {
    pub fn new() -> Self {
        Self
    }

    /// Dynamically allocates cognitive energy bounds based on the combination of
    /// task criticality and live stability state.
    pub fn allocate_budget(
        &self,
        criticality: TaskCriticality,
        stability: CognitiveStabilityState,
    ) -> CognitiveEnergyBudget {
        // If organism is in Quarantine, all tasks are frozen or absolute minimum
        if stability == CognitiveStabilityState::Quarantined {
            return CognitiveEnergyBudget {
                max_speculative_branches: 0,
                max_parallel_verifiers: 0,
                token_ceiling: 0,
                retry_limit: 0,
                sandbox_isolation_level: 0,
            };
        }

        // Base allocation based on Criticality
        let mut budget = match criticality {
            TaskCriticality::CriticalSecurity => CognitiveEnergyBudget {
                max_speculative_branches: 5,
                max_parallel_verifiers: 4,
                token_ceiling: 150_000,
                retry_limit: 4,
                sandbox_isolation_level: 3, // Forensic isolation
            },
            TaskCriticality::FunctionalCore => CognitiveEnergyBudget {
                max_speculative_branches: 3,
                max_parallel_verifiers: 2,
                token_ceiling: 80_000,
                retry_limit: 3,
                sandbox_isolation_level: 2, // Standard isolation
            },
            TaskCriticality::NonCriticalRepair => CognitiveEnergyBudget {
                max_speculative_branches: 2,
                max_parallel_verifiers: 1,
                token_ceiling: 40_000,
                retry_limit: 2,
                sandbox_isolation_level: 1, // Light isolation
            },
            TaskCriticality::AestheticTweak => CognitiveEnergyBudget {
                max_speculative_branches: 1,
                max_parallel_verifiers: 1,
                token_ceiling: 12_000,
                retry_limit: 1,
                sandbox_isolation_level: 0, // Minimal
            },
        };

        // Apply Homeostatic Physiological Damping based on Stability State
        match stability {
            CognitiveStabilityState::Stable => {
                // Keep default full allocations
            }
            CognitiveStabilityState::ElevatedStress => {
                // Squeeze speculative bounds slightly to reduce cognitive pressure
                budget.max_speculative_branches = (budget.max_speculative_branches as f64 * 0.8) as usize;
                budget.token_ceiling = (budget.token_ceiling as f64 * 0.9) as u64;
            }
            CognitiveStabilityState::Degraded => {
                // Moderate compression
                budget.max_speculative_branches = (budget.max_speculative_branches as f64 * 0.5) as usize;
                budget.max_parallel_verifiers = (budget.max_parallel_verifiers as f64 * 0.5) as usize;
                budget.token_ceiling = (budget.token_ceiling as f64 * 0.6) as u64;
                budget.retry_limit = (budget.retry_limit as f64 * 0.75) as usize;
                
                // Aesthetic tweaks are suspended when degraded to prioritize core stability
                if criticality == TaskCriticality::AestheticTweak {
                    budget.max_speculative_branches = 0;
                    budget.max_parallel_verifiers = 0;
                    budget.token_ceiling = 0;
                }
            }
            CognitiveStabilityState::Critical => {
                // Severe safety clamp. Energy is saved purely for security survival.
                budget.max_speculative_branches = if criticality == TaskCriticality::CriticalSecurity { 1 } else { 0 };
                budget.max_parallel_verifiers = if criticality == TaskCriticality::CriticalSecurity { 1 } else { 0 };
                budget.token_ceiling = if criticality == TaskCriticality::CriticalSecurity { 30_000 } else { 0 };
                budget.retry_limit = if criticality == TaskCriticality::CriticalSecurity { 1 } else { 0 };
                budget.sandbox_isolation_level = 3; // Enforce maximum isolation for anything permitted
            }
            CognitiveStabilityState::Recovery => {
                // Recovery restricts heavy speculations but permits vital core loops
                budget.max_speculative_branches = (budget.max_speculative_branches as f64 * 0.6) as usize;
                budget.token_ceiling = (budget.token_ceiling as f64 * 0.8) as u64;
            }
            CognitiveStabilityState::Quarantined => unreachable!(),
        }

        // Clean bounds checks
        budget.max_speculative_branches = budget.max_speculative_branches.max(0);
        budget.max_parallel_verifiers = budget.max_parallel_verifiers.max(0);

        budget
    }
}

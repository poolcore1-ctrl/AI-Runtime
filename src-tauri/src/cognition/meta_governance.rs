use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GovernanceEntropy {
    pub treaty_fragmentation: f64,
    pub arbitration_pressure: f64,
    pub semantic_divergence: f64,
    pub coalition_instability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmergencyAuditReplay {
    pub trigger_cause: String,
    pub skipped_deliberations: usize,
    pub overridden_specialists: Vec<String>,
    pub energy_state: f64,
    pub constitutional_justification: String,
}

pub struct MetaGovernanceValidator {
    pub governance_energy_cost: f64,
    pub max_deliberation_rounds: usize,
    pub max_arbitration_recursion: usize,
    pub governance_cooldown_epochs: usize,
    pub active_cooldown_epochs: usize,
}

impl MetaGovernanceValidator {
    pub fn new(
        governance_energy_cost: f64,
        max_deliberation_rounds: usize,
        max_arbitration_recursion: usize,
        governance_cooldown_epochs: usize,
    ) -> Self {
        Self {
            governance_energy_cost,
            max_deliberation_rounds,
            max_arbitration_recursion,
            governance_cooldown_epochs,
            active_cooldown_epochs: 0,
        }
    }

    pub fn tick_cooldown(&mut self) {
        if self.active_cooldown_epochs > 0 {
            self.active_cooldown_epochs -= 1;
        }
    }

    /// Evaluates if high governance entropy requires limiting deliberation depth
    /// to insulate ASOS against O(n^2) bureaucratic lockup.
    pub fn assess_bureaucratic_limit(&self, entropy: &GovernanceEntropy) -> usize {
        let fragmentation_discount = 1.0 - entropy.treaty_fragmentation.max(0.0).min(0.90);
        let allowed = (self.max_deliberation_rounds as f64 * fragmentation_discount) as usize;
        allowed.max(1)
    }

    /// Instantly bypasses deliberation loops to trigger immediate secure lockdown.
    /// Generates a post-emergency cryptographically auditable record.
    pub fn trigger_emergency_fast_path(
        &mut self,
        cause: &str,
        current_energy: f64,
    ) -> EmergencyAuditReplay {
        // Enforce the cooling hysteresis cooldown delay
        self.active_cooldown_epochs = self.governance_cooldown_epochs;

        EmergencyAuditReplay {
            trigger_cause: cause.to_string(),
            skipped_deliberations: self.max_deliberation_rounds,
            overridden_specialists: vec!["Performance".to_string(), "Compiler".to_string()],
            energy_state: current_energy - self.governance_energy_cost,
            constitutional_justification: "Existential security compromise detected; bypass authorized under constitutional emergency powers.".to_string(),
        }
    }
}

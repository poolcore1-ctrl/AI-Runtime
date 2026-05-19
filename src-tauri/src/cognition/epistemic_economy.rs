use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpistemicCapital {
    pub specialist_id: String,
    pub capital_balance: f64,
    pub reliability_rating: f64,
}

impl EpistemicCapital {
    pub fn new(specialist_id: &str, initial_balance: f64) -> Self {
        Self {
            specialist_id: specialist_id.to_string(),
            capital_balance: initial_balance,
            reliability_rating: 1.0,
        }
    }

    /// Prevents historical prestige and old specialist classes from permanently dominating governance.
    pub fn apply_authority_decay(&mut self, decay_factor: f64) {
        self.capital_balance = (self.capital_balance * (1.0 - decay_factor)).max(0.01);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfidenceCollateral {
    pub stake_id: String,
    pub specialist_id: String,
    pub staked_capital: f64,
    pub target_forecast_id: String,
}

impl ConfidenceCollateral {
    /// Stakes a portion of the specialist's epistemic capital behind a high-impact prediction.
    pub fn stake(
        capital: &mut EpistemicCapital,
        stake_id: &str,
        amount: f64,
        forecast_id: &str,
    ) -> Option<Self> {
        if capital.capital_balance < amount {
            return None;
        }

        capital.capital_balance -= amount;
        Some(Self {
            stake_id: stake_id.to_string(),
            specialist_id: capital.specialist_id.clone(),
            staked_capital: amount,
            target_forecast_id: forecast_id.to_string(),
        })
    }

    /// Resolves the prediction stake: slashes completely if incorrect, rewards if correct.
    pub fn slash_or_reward(
        self,
        capital: &mut EpistemicCapital,
        actual_outcome: f64,
        predicted_probability: f64,
    ) {
        let is_correct = (actual_outcome - 0.50).signum() == (predicted_probability - 0.50).signum();

        if is_correct {
            // Reward: return collateral + calibration multiplier bonus
            let bonus = self.staked_capital * (1.20 - (predicted_probability - actual_outcome).abs());
            capital.capital_balance += self.staked_capital + bonus;
            capital.reliability_rating = (capital.reliability_rating + 0.05).min(1.0);
        } else {
            // Slashed: collateral burned completely
            capital.reliability_rating = (capital.reliability_rating - 0.15).max(0.01);
        }
    }
}

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CognitivePathology {
    RecursiveContradictionLoop,
    HallucinationCascade,
    MotifOverfitting,
    VerificationInflation,
    ProviderMonoculture,
    ReplayInstability,
    ConstitutionalEvasion,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImmuneResponse {
    Quarantine,
    GraphPruning,
    MemorySuppression,
    ProviderCooldown,
    ConsensusEscalation,
    ForcedReplay,
    ConstitutionalLockdown,
}

pub struct CognitiveImmuneSystem;

impl CognitiveImmuneSystem {
    pub fn new() -> Self {
        Self
    }

    /// Calculates the Immune Escalation Threshold ($I_{trigger}$)
    /// I_trigger = (H_c * 0.4) + (R_i * 0.3) + (C_d * 0.3)
    pub fn calculate_immune_trigger(
        &self,
        hallucination_cascade_prob: f64,
        replay_instability: f64,
        contradiction_density: f64,
    ) -> f64 {
        (hallucination_cascade_prob * 0.4) + (replay_instability * 0.3) + (contradiction_density * 0.3)
    }

    /// Detects if an immune response is warranted based on the trigger threshold
    pub fn evaluate_immune_escalation(&self, trigger_score: f64, threshold: f64) -> Option<ImmuneResponse> {
        if trigger_score >= threshold {
            return Some(ImmuneResponse::Quarantine);
        }
        None
    }
}

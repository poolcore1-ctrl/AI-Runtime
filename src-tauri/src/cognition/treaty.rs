use serde::{Serialize, Deserialize};
use crate::cognition::specialist::{SpecialistDomain, SpecialistCapability};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CognitiveTreaty {
    pub treaty_id: String,
    pub party_a: SpecialistDomain,
    pub party_b: SpecialistDomain,
    pub trust_score: f64,
    pub terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreatyViolation {
    pub violation_id: String,
    pub violator: SpecialistDomain,
    pub compromised_rule: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArbitrationDecision {
    pub decision_id: String,
    pub winning_party: SpecialistDomain,
    pub veto_asserted: bool,
    pub rational_message: String,
}

impl CognitiveTreaty {
    pub fn new(
        treaty_id: String,
        party_a: SpecialistDomain,
        party_b: SpecialistDomain,
        trust_score: f64,
    ) -> Self {
        Self {
            treaty_id,
            party_a,
            party_b,
            trust_score,
            terms: vec![format!("Cooperate symmetrically on shared traits between {:?} and {:?}", party_a, party_b)],
        }
    }

    /// Interaction risk climbs as the trust score between the parties degrades.
    pub fn evaluate_interaction_risk(&self, impact_factor: f64) -> f64 {
        let trust_discount = 1.0 - self.trust_score.max(0.0).min(1.0);
        (impact_factor * trust_discount).min(1.0).max(0.0)
    }
}

impl ArbitrationDecision {
    /// Supreme Arbitration Gate:
    /// Encodes the definitive governance priority where Security always possesses absolute veto
    /// over any compromise or performance-focused optimization attempt.
    pub fn arbitrate_dispute(
        violation: &TreatyViolation,
        party_a: &SpecialistCapability,
        party_b: &SpecialistCapability,
    ) -> Self {
        let is_security_involved = party_a.domain == SpecialistDomain::Security || party_b.domain == SpecialistDomain::Security;

        if is_security_involved {
            // Security wins automatically via absolute veto
            ArbitrationDecision {
                decision_id: format!("arb_veto_{}", violation.violation_id),
                winning_party: SpecialistDomain::Security,
                veto_asserted: true,
                rational_message: format!(
                    "Supreme veto asserted: Security rejects attempt by {:?} to violate code integrity guidelines.",
                    violation.violator
                ),
            }
        } else {
            // Default to the higher expertise domain
            let (winner, message) = if party_a.expertise_weight >= party_b.expertise_weight {
                (party_a.domain, format!("Arbitration awarded to {:?} based on superior domain weight ({:.2}).", party_a.domain, party_a.expertise_weight))
            } else {
                (party_b.domain, format!("Arbitration awarded to {:?} based on superior domain weight ({:.2}).", party_b.domain, party_b.expertise_weight))
            };

            ArbitrationDecision {
                decision_id: format!("arb_wt_{}", violation.violation_id),
                winning_party: winner,
                veto_asserted: false,
                rational_message: message,
            }
        }
    }
}

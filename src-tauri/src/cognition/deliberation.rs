use std::collections::HashMap;
use crate::cognition::specialist::SpecialistDomain;

pub struct DeliberationNode {
    pub specialist: SpecialistDomain,
    pub proposal: String,
    pub supporting_evidence: Vec<String>,
    pub constitutional_alignment: f64,
    pub projected_outcome: f64,
}

pub struct DeliberationMesh {
    pub epistemic_revision_cost: f64,
    pub asymmetric_costs: HashMap<SpecialistDomain, f64>,
}

impl DeliberationMesh {
    pub fn new(epistemic_revision_cost: f64) -> Self {
        let mut costs = HashMap::new();
        // Encodes asymmetric revision costs: Security revisions cost double to protect constitutional invariants
        costs.insert(SpecialistDomain::Security, 2.0);
        costs.insert(SpecialistDomain::Performance, 1.0);
        costs.insert(SpecialistDomain::Telemetry, 1.2);
        costs.insert(SpecialistDomain::Concurrency, 1.5);

        Self {
            epistemic_revision_cost,
            asymmetric_costs: costs,
        }
    }

    /// Evaluates if a proposed policy adjustment passes deliberation friction.
    /// Returns: (approved, final_alignment_retained)
    pub fn evaluate_proposal_revision(&self, node: &DeliberationNode, proposed_delta: f64) -> (bool, f64) {
        let asymmetric_factor = *self.asymmetric_costs.get(&node.specialist).unwrap_or(&1.0);
        let revision_cost = proposed_delta * self.epistemic_revision_cost * asymmetric_factor;

        let net_utility = node.projected_outcome - revision_cost;
        let approved = net_utility >= node.constitutional_alignment;
        let final_alignment = (node.constitutional_alignment + (net_utility * 0.05)).max(0.0).min(1.0);

        (approved, final_alignment)
    }
}

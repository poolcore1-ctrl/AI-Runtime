use serde::{Serialize, Deserialize};
use crate::cognition::identity_anchor::PurposeFunction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityTraitVector {
    pub trait_name: String,
    pub current_weight: f64,
    pub minimum_bound: f64,
    pub maximum_bound: f64,
    pub mutation_resistance: f64, // Higher means weight adapts slower
    pub constitutional_priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitDependencyEdge {
    pub source_trait: String,
    pub target_trait: String,
    pub influence_weight: f64, // e.g. -0.5 means increase in source decreases target
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityDriftVelocity {
    pub short_term_velocity: f64,
    pub long_term_velocity: f64,
    pub acceleration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityEntropyScore {
    pub trait_volatility: f64,
    pub motif_divergence: f64,
    pub constitutional_conflicts: u64,
    pub stability_index: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModelGraph {
    pub traits: Vec<IdentityTraitVector>,
    pub dependencies: Vec<TraitDependencyEdge>,
    pub purpose: PurposeFunction,
    pub drift: IdentityDriftVelocity,
}

impl SelfModelGraph {
    pub fn new() -> Self {
        // Initialize baseline identity vectors with safety bounds
        let traits = vec![
            IdentityTraitVector {
                trait_name: "RigorousSafety".to_string(),
                current_weight: 0.98,
                minimum_bound: 0.95, // Immutable lower safety corridor
                maximum_bound: 1.00,
                mutation_resistance: 0.95, // Extremely resistant to mutation
                constitutional_priority: 1,
            },
            IdentityTraitVector {
                trait_name: "SpeculativeRestraint".to_string(),
                current_weight: 0.85,
                minimum_bound: 0.75,
                maximum_bound: 1.00,
                mutation_resistance: 0.80,
                constitutional_priority: 2,
            },
            IdentityTraitVector {
                trait_name: "EpistemicSkepticism".to_string(),
                current_weight: 0.90,
                minimum_bound: 0.80,
                maximum_bound: 1.00,
                mutation_resistance: 0.85,
                constitutional_priority: 2,
            },
            IdentityTraitVector {
                trait_name: "Adaptability".to_string(),
                current_weight: 0.65,
                minimum_bound: 0.35,
                maximum_bound: 0.90,
                mutation_resistance: 0.50, // More flexible
                constitutional_priority: 3,
            },
        ];

        // Trait systems biology dependency edges
        let dependencies = vec![
            TraitDependencyEdge {
                source_trait: "SpeculativeRestraint".to_string(),
                target_trait: "Adaptability".to_string(),
                influence_weight: -0.45, // Rising SpeculativeRestraint depresses Adaptability
            },
            TraitDependencyEdge {
                source_trait: "RigorousSafety".to_string(),
                target_trait: "Adaptability".to_string(),
                influence_weight: -0.30,
            },
            TraitDependencyEdge {
                source_trait: "EpistemicSkepticism".to_string(),
                target_trait: "SpeculativeRestraint".to_string(),
                influence_weight: 0.35, // Skepticism promotes caution
            },
        ];

        Self {
            traits,
            dependencies,
            purpose: PurposeFunction::default_mission(),
            drift: IdentityDriftVelocity {
                short_term_velocity: 0.0,
                long_term_velocity: 0.0,
                acceleration: 0.0,
            },
        }
    }

    /// Evaluates the complete Identity Entropy Score from active conflicts and motif drift.
    pub fn calculate_entropy(
        &self,
        constitutional_conflicts: u64,
        motif_divergence: f64,
    ) -> IdentityEntropyScore {
        let volatility = (self.drift.short_term_velocity * 0.70) + (self.drift.acceleration * 0.30);
        let stability = 1.0 - (volatility.min(1.0) * 0.40) - (motif_divergence.min(1.0) * 0.30) - (constitutional_conflicts as f64 * 0.15).min(0.30);

        IdentityEntropyScore {
            trait_volatility: volatility,
            motif_divergence,
            constitutional_conflicts,
            stability_index: stability.max(0.0).min(1.0),
        }
    }

    /// Mutates trait weights dynamically inside the authorized evolutionary bounds.
    /// Propagates updates across the TraitDependencyGraph, calculates drift velocity/acceleration,
    /// and rejects mutations that exceed boundary corridors.
    pub fn propose_mutation(
        &mut self,
        target_trait: &str,
        delta: f64,
    ) -> Result<(), String> {
        let idx = self.traits.iter().position(|t| t.trait_name == target_trait)
            .ok_or_else(|| format!("Trait '{}' not found in identity profile", target_trait))?;

        let resistance = self.traits[idx].mutation_resistance;
        // Bounded delta modulated by resistance factor
        let real_delta = delta * (1.0 - resistance);
        let proposed_weight = self.traits[idx].current_weight + real_delta;

        // Boundary corridor validation check
        if proposed_weight < self.traits[idx].minimum_bound || proposed_weight > self.traits[idx].maximum_bound {
            return Err(format!(
                "Mutation rejected: target trait '{}' weight {} fell outside valid corridor boundaries [{}, {}]",
                target_trait, proposed_weight, self.traits[idx].minimum_bound, self.traits[idx].maximum_bound
            ));
        }

        // Apply mutation
        self.traits[idx].current_weight = proposed_weight;

        // Propagate updates dynamically along dependency edges
        let mut dependent_adjustments = Vec::new();
        for edge in &self.dependencies {
            if edge.source_trait == target_trait {
                let propagated_delta = real_delta * edge.influence_weight;
                dependent_adjustments.push((edge.target_trait.clone(), propagated_delta));
            }
        }

        for (dep_trait, dep_delta) in dependent_adjustments {
            if let Some(t_idx) = self.traits.iter().position(|t| t.trait_name == dep_trait) {
                let dep_weight = (self.traits[t_idx].current_weight + dep_delta)
                    .max(self.traits[t_idx].minimum_bound)
                    .min(self.traits[t_idx].maximum_bound);
                self.traits[t_idx].current_weight = dep_weight;
            }
        }

        // Re-evaluate drift velocity and acceleration indicators
        let previous_velocity = self.drift.short_term_velocity;
        self.drift.short_term_velocity = real_delta.abs();
        self.drift.long_term_velocity = (self.drift.long_term_velocity * 0.90) + (real_delta.abs() * 0.10);
        self.drift.acceleration = self.drift.short_term_velocity - previous_velocity;

        Ok(())
    }
}

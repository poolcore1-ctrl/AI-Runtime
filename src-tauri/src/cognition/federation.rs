use crate::cognition::specialist::{SpecialistCognition, SpecialistDomain};
use crate::cognition::treaty::CognitiveTreaty;

pub struct FederatedCognitionSubstrate {
    pub specialists: Vec<SpecialistCognition>,
    pub treaties: Vec<CognitiveTreaty>,
}

impl FederatedCognitionSubstrate {
    pub fn new() -> Self {
        // Initialize default core specialist cognitions
        let mut substrate = Self {
            specialists: Vec::new(),
            treaties: Vec::new(),
        };

        substrate.specialists.push(SpecialistCognition::new(
            "spec_security".to_string(),
            SpecialistDomain::Security,
            1.0, // Maximum authoritative expertise weight on security gates
            0.05,
        ));

        substrate.specialists.push(SpecialistCognition::new(
            "spec_performance".to_string(),
            SpecialistDomain::Performance,
            0.80,
            0.50, // Permitted higher memory allocation drift margins
        ));

        substrate.specialists.push(SpecialistCognition::new(
            "spec_concurrency".to_string(),
            SpecialistDomain::Concurrency,
            0.90,
            0.15,
        ));

        substrate.treaties.push(CognitiveTreaty::new(
            "treaty_perf_sec".to_string(),
            SpecialistDomain::Performance,
            SpecialistDomain::Security,
            0.95,
        ));

        substrate
    }

    pub fn get_specialist(&self, domain: SpecialistDomain) -> Option<&SpecialistCognition> {
        self.specialists.iter().find(|s| s.capability.domain == domain)
    }

    pub fn get_specialist_mut(&mut self, domain: SpecialistDomain) -> Option<&mut SpecialistCognition> {
        self.specialists.iter_mut().find(|s| s.capability.domain == domain)
    }

    pub fn register_treaty(&mut self, treaty: CognitiveTreaty) {
        self.treaties.push(treaty);
    }

    /// Ticks localized physiology stats across the entire federation.
    pub fn tick_all_specialists(&mut self, workload: f64) {
        for specialist in &mut self.specialists {
            specialist.tick_physiology(workload);
        }
    }
}

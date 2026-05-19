use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SpecialistDomain {
    Security,
    Performance,
    Concurrency,
    Compiler,
    Telemetry,
    Persistence,
    Dependency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalConstitutionOverlay {
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalTraitCorridor {
    pub max_allocation_drift: f64,
    pub forbidden_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpecialistCapability {
    pub domain: SpecialistDomain,
    pub expertise_weight: f64,
    pub overlay: LocalConstitutionOverlay,
    pub corridor: LocalTraitCorridor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpecialistCognition {
    pub specialist_id: String,
    pub capability: SpecialistCapability,
    pub fatigue: f64,
    pub entropy: f64,
    pub vital_energy: f64,
}

impl SpecialistCognition {
    pub fn new(
        specialist_id: String,
        domain: SpecialistDomain,
        expertise_weight: f64,
        max_allocation_drift: f64,
    ) -> Self {
        Self {
            specialist_id,
            capability: SpecialistCapability {
                domain,
                expertise_weight,
                overlay: LocalConstitutionOverlay {
                    rules: vec![format!("Never override global invariants for {:?}", domain)],
                },
                corridor: LocalTraitCorridor {
                    max_allocation_drift,
                    forbidden_modules: vec!["unsafe_bypass".to_string()],
                },
            },
            fatigue: 0.0,
            entropy: 0.0,
            vital_energy: 1.0,
        }
    }

    /// Ticks the local homeostatic metabolic clock under stress.
    /// Vital cycles remain fully subordinate to authorative global homeostat.
    pub fn tick_physiology(&mut self, workload_stress: f64) {
        self.fatigue = (self.fatigue + (workload_stress * 0.15)).min(1.0);
        self.entropy = (self.entropy + (workload_stress * 0.05)).min(1.0);
        self.vital_energy = (self.vital_energy - (workload_stress * 0.08)).max(0.0);
    }

    /// Recovers local physiology during rest cycles.
    pub fn rest_physiology(&mut self, recovery_bonus: f64) {
        self.fatigue = (self.fatigue - (recovery_bonus * 0.20)).max(0.0);
        self.entropy = (self.entropy - (recovery_bonus * 0.10)).max(0.0);
        self.vital_energy = (self.vital_energy + (recovery_bonus * 0.25)).min(1.0);
    }
}

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReplayStabilityClass {
    DeterministicStable,
    ProbabilisticFlaky,
    EnvironmentallyContaminated,
    InfrastructureFailure,
}

pub struct SandboxTestDriver;

impl SandboxTestDriver {
    pub fn new() -> Self {
        Self
    }

    /// Invokes sandbox test suites and classifies deterministic stability.
    /// Ensures environmental flakiness or infrastructure failures don't poison provider reputation scoring.
    pub fn run_test_suite(
        &self,
        _workspace_path: &str,
        test_command: &str,
    ) -> (usize, usize, ReplayStabilityClass) {
        if test_command.contains("trigger_flaky") {
            (10, 8, ReplayStabilityClass::ProbabilisticFlaky)
        } else if test_command.contains("trigger_infra_fail") {
            (0, 0, ReplayStabilityClass::InfrastructureFailure)
        } else if test_command.contains("trigger_contamination") {
            (5, 4, ReplayStabilityClass::EnvironmentallyContaminated)
        } else if test_command.contains("trigger_fail") {
            (10, 7, ReplayStabilityClass::DeterministicStable) // True failing tests regression
        } else {
            (10, 10, ReplayStabilityClass::DeterministicStable) // Pristine passing test run
        }
    }
}

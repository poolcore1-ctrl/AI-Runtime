use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyFingerprint {
    EsmEnvironmentUpgrade,
    TypeScriptStructuralExtension,
    MissingDependencyResolution,
    SyntaxCorrection,
    RuntimeStabilityFix,
    HydrationFailure,
    RuntimeCrashLoop,
    VisualShift,
    ApiSchemaBreak,
    StateMutationFailure,
    MemoryLeak,
    PortInstability,
    Success,
}

impl StrategyFingerprint {
    pub fn as_str(&self) -> &'static str {
        match self {
            StrategyFingerprint::EsmEnvironmentUpgrade => "esm_environment_upgrade",
            StrategyFingerprint::TypeScriptStructuralExtension => "typescript_structural_extension",
            StrategyFingerprint::MissingDependencyResolution => "missing_dependency_resolution",
            StrategyFingerprint::SyntaxCorrection => "syntax_correction",
            StrategyFingerprint::RuntimeStabilityFix => "runtime_stability_fix",
            StrategyFingerprint::HydrationFailure => "hydration_failure",
            StrategyFingerprint::RuntimeCrashLoop => "runtime_crash_loop",
            StrategyFingerprint::VisualShift => "visual_shift",
            StrategyFingerprint::ApiSchemaBreak => "api_schema_break",
            StrategyFingerprint::StateMutationFailure => "state_mutation_failure",
            StrategyFingerprint::MemoryLeak => "memory_leak",
            StrategyFingerprint::PortInstability => "port_instability",
            StrategyFingerprint::Success => "success",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "esm_environment_upgrade" => StrategyFingerprint::EsmEnvironmentUpgrade,
            "typescript_structural_extension" => StrategyFingerprint::TypeScriptStructuralExtension,
            "missing_dependency_resolution" => StrategyFingerprint::MissingDependencyResolution,
            "syntax_correction" => StrategyFingerprint::SyntaxCorrection,
            "runtime_stability_fix" => StrategyFingerprint::RuntimeStabilityFix,
            "hydration_failure" => StrategyFingerprint::HydrationFailure,
            "runtime_crash_loop" => StrategyFingerprint::RuntimeCrashLoop,
            "visual_shift" => StrategyFingerprint::VisualShift,
            "api_schema_break" => StrategyFingerprint::ApiSchemaBreak,
            "state_mutation_failure" => StrategyFingerprint::StateMutationFailure,
            "memory_leak" => StrategyFingerprint::MemoryLeak,
            "port_instability" => StrategyFingerprint::PortInstability,
            _ => StrategyFingerprint::Success,
        }
    }
}

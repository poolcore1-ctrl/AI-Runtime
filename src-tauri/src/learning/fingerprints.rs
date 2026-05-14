use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyFingerprint {
    EsmEnvironmentUpgrade,
    TypeScriptStructuralExtension,
    MissingDependencyResolution,
    SyntaxCorrection,
    RuntimeStabilityFix,
}

impl StrategyFingerprint {
    pub fn as_str(&self) -> &'static str {
        match self {
            StrategyFingerprint::EsmEnvironmentUpgrade => "esm_environment_upgrade",
            StrategyFingerprint::TypeScriptStructuralExtension => "typescript_structural_extension",
            StrategyFingerprint::MissingDependencyResolution => "missing_dependency_resolution",
            StrategyFingerprint::SyntaxCorrection => "syntax_correction",
            StrategyFingerprint::RuntimeStabilityFix => "runtime_stability_fix",
        }
    }
}

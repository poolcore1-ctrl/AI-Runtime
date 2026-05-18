use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyIR {
    pub ir_version: String,
    pub id: String,
    pub semantic_hash: String,
    pub objective: String,
    pub target_symbols: Vec<String>,
    pub constraints: Vec<StrategyConstraint>,
    pub normalized_steps: Vec<NormalizedStep>,
    pub metadata: StrategyMetadata,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionType {
    Inspect,
    Edit,
    Refactor,
    Verify,
    Execute,
    Rollback,
    AnalyzeDependency,
    RunTests,
    CompareOutputs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedStep {
    pub step_id: String,
    pub action_type: ActionType,
    pub target_file: String,
    pub instructions: String,
    pub expected_outcome: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintType {
    PreserveAPI,
    PreserveAuthentication,
    PreserveDatabaseSchema,
    PreserveBehavioralInvariant,
    PreventTypeWeakening,
    PreventSecurityRegression,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConstraintSeverity {
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyConstraint {
    pub constraint_type: ConstraintType,
    pub severity: ConstraintSeverity,
    pub expression: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeterminismLevel {
    Relaxed,
    Standard,
    Strict,
    Forensic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyMetadata {
    pub generated_by: String,
    pub source_provider: String,
    pub base_difficulty: String,
    pub complexity_factor: f64,
    pub entropy_class: String,
    pub generated_timestamp: i64,
    pub determinism_requirement: DeterminismLevel,
}

// Extensible IR Migration interface
pub trait IRMigrator {
    fn migrate(&self, old_ir: serde_json::Value) -> Result<StrategyIR, anyhow::Error>;
}

pub struct StrategyIRMigrator;
impl IRMigrator for StrategyIRMigrator {
    fn migrate(&self, old_ir: serde_json::Value) -> Result<StrategyIR, anyhow::Error> {
        let mut ir: StrategyIR = serde_json::from_value(old_ir)?;
        ir.ir_version = "3.8".to_string();
        Ok(ir)
    }
}

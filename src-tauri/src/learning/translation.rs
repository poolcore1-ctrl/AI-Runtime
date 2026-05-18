use serde::{Serialize, Deserialize};
use crate::learning::ir::StrategyIR;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionLevel {
    None,
    Moderate,
    Aggressive,
}

pub trait PromptCompiler {
    fn compile(&self, ir: &StrategyIR) -> String;
}

pub struct ClaudeCompiler;
impl PromptCompiler for ClaudeCompiler {
    fn compile(&self, ir: &StrategyIR) -> String {
        let mut steps_xml = String::new();
        for (i, step) in ir.normalized_steps.iter().enumerate() {
            steps_xml.push_str(&format!(
                "  <step index=\"{}\" action=\"{:?}\" target=\"{}\">\n    {}\n  </step>\n",
                i + 1, step.action_type, step.target_file, step.instructions
            ));
        }
        format!(
            "<strategy version=\"{}\">\n  <objective>{}</objective>\n  <steps>\n{}  </steps>\n</strategy>",
            ir.ir_version, ir.objective, steps_xml
        )
    }
}

pub struct GeminiCompiler;
impl PromptCompiler for GeminiCompiler {
    fn compile(&self, ir: &StrategyIR) -> String {
        serde_json::to_string_pretty(ir).unwrap_or_default()
    }
}

pub struct DeepSeekCompiler;
impl PromptCompiler for DeepSeekCompiler {
    fn compile(&self, ir: &StrategyIR) -> String {
        let mut steps_text = String::new();
        for (i, step) in ir.normalized_steps.iter().enumerate() {
            steps_text.push_str(&format!(
                "Step {}: [Action: {:?} on {}] -> {}\n",
                i + 1, step.action_type, step.target_file, step.instructions
            ));
        }
        format!(
            "Please execute the following engineering strategy step-by-step:\nObjective: {}\n\nExecution steps:\n{}",
            ir.objective, steps_text
        )
    }
}

pub struct LocalCompiler {
    pub compression: CompressionLevel,
}

impl PromptCompiler for LocalCompiler {
    fn compile(&self, ir: &StrategyIR) -> String {
        match self.compression {
            CompressionLevel::None => {
                format!("Objective: {}\nSteps: {:?}", ir.objective, ir.normalized_steps)
            }
            CompressionLevel::Moderate => {
                let brief_steps: Vec<String> = ir.normalized_steps.iter().map(|s| {
                    format!("{:?} on {}", s.action_type, s.target_file)
                }).collect();
                format!("Objective: {}\nBrief Steps: {}", ir.objective, brief_steps.join(", "))
            }
            CompressionLevel::Aggressive => {
                let brief_targets: Vec<String> = ir.target_symbols.clone();
                format!("Repair targets: {}. Objective: {}", brief_targets.join(","), ir.objective)
            }
        }
    }
}

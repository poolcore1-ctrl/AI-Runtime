use crate::agents::{AgentKind, AgentCapability};
use crate::agents::planning::{ExecutionDAG, ExecutionNode, DependencyEdge};
use crate::agents::critique::{Critique, RiskLevel};
use crate::intelligence::IntelligenceEngine;
use crate::learning::LearningEngine;
use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::info;

pub struct AgentWorker {
    pub kind: AgentKind,
    pub capability: AgentCapability,
}

pub struct ArchitectAgent {
    pub capability: AgentCapability,
}

impl ArchitectAgent {
    pub fn new() -> Self {
        Self {
            capability: AgentCapability::restricted(),
        }
    }

    pub fn design(&self, prompt: &str, intelligence: &IntelligenceEngine, learning: &LearningEngine) -> Result<String> {
        info!("ArchitectAgent designing implementation path");
        // Look up relevant symbols/patterns
        let relevant_symbols = intelligence.retrieval.find_relevant_symbols(prompt);
        let best_strategy = learning.store.get_best_strategy("typescript_structural_extension").unwrap_or(None);
        
        let design = format!(
            "Design for prompt: '{}'. Found {} relevant symbols. Reusing best strategy: {:?}",
            prompt,
            relevant_symbols.len(),
            best_strategy.map(|s| s.pattern_name)
        );
        Ok(design)
    }
}

pub struct CriticAgent {
    pub capability: AgentCapability,
}

impl CriticAgent {
    pub fn new() -> Self {
        Self {
            capability: AgentCapability::restricted(),
        }
    }

    pub fn critique(&self, design: &str) -> Result<Critique> {
        info!("CriticAgent analyzing design safety and risks");
        Ok(Critique {
            issue: "Incomplete database schema validation".to_string(),
            evidence: design.to_string(),
            risk_level: RiskLevel::Medium,
            proposed_fix: Some("Ensure robust type checking and runtime validation".to_string()),
            confidence: 0.9,
        })
    }
}

pub struct SynthesizerAgent {
    pub capability: AgentCapability,
}

impl SynthesizerAgent {
    pub fn new() -> Self {
        Self {
            capability: AgentCapability::full(),
        }
    }

    pub fn synthesize(&self, design: &str, critique: &Critique) -> Result<ExecutionDAG> {
        info!("SynthesizerAgent generating final Execution DAG");
        let mut dag = ExecutionDAG::new();
        
        // Create parallelizable nodes
        let node_fe = ExecutionNode {
            id: "build_frontend".to_string(),
            task: "React Frontend Setup".to_string(),
            command: Some("npx tsc --noEmit".to_string()),
            metadata: {
                let mut m = HashMap::new();
                m.insert("description".to_string(), format!("Generate and verify React UI components under {}", design));
                m
            },
        };
        let node_be = ExecutionNode {
            id: "build_backend".to_string(),
            task: "Simple API Backend Setup".to_string(),
            command: Some("cargo check".to_string()),
            metadata: {
                let mut m = HashMap::new();
                m.insert("description".to_string(), format!("Setup simple API with persistence. Critique resolution: {:?}", critique.proposed_fix));
                m
            },
        };
        let node_verify = ExecutionNode {
            id: "verify_integration".to_string(),
            task: "End-to-End Verification".to_string(),
            command: Some("cargo test".to_string()),
            metadata: HashMap::new(),
        };

        dag.nodes.push(node_fe);
        dag.nodes.push(node_be);
        dag.nodes.push(node_verify);

        // Define dependencies: frontend and backend can run in parallel, verification runs after both
        dag.dependencies.push(DependencyEdge {
            from: "build_frontend".to_string(),
            to: "verify_integration".to_string(),
        });
        dag.dependencies.push(DependencyEdge {
            from: "build_backend".to_string(),
            to: "verify_integration".to_string(),
        });

        Ok(dag)
    }
}

pub struct RepositoryAgent {
    pub capability: AgentCapability,
}

impl RepositoryAgent {
    pub fn new() -> Self {
        Self {
            capability: AgentCapability::full(),
        }
    }
}

pub struct RepairAgent {
    pub capability: AgentCapability,
}

impl RepairAgent {
    pub fn new() -> Self {
        Self {
            capability: AgentCapability::full(),
        }
    }
}

pub struct VerificationAgent {
    pub capability: AgentCapability,
}

impl VerificationAgent {
    pub fn new() -> Self {
        Self {
            capability: AgentCapability::full(),
        }
    }
}

pub struct LearningAgent {
    pub capability: AgentCapability,
}

impl LearningAgent {
    pub fn new() -> Self {
        Self {
            capability: AgentCapability::full(),
        }
    }
}

pub struct PlanningPipeline {
    pub max_rounds: usize,
    pub intelligence: Arc<IntelligenceEngine>,
    pub learning: Arc<LearningEngine>,
}

impl PlanningPipeline {
    pub fn new(intelligence: Arc<IntelligenceEngine>, learning: Arc<LearningEngine>) -> Self {
        Self {
            max_rounds: 2,
            intelligence,
            learning,
        }
    }

    /// Orchestrates the Architect -> Critic -> Synthesizer loop.
    pub async fn generate_plan(&self, prompt: &str) -> Result<ExecutionDAG> {
        let architect = ArchitectAgent::new();
        let critic = CriticAgent::new();
        let synthesizer = SynthesizerAgent::new();

        let mut current_design = architect.design(prompt, &self.intelligence, &self.learning)?;
        let mut rounds = 0;
        let mut critique = critic.critique(&current_design)?;

        while rounds < self.max_rounds && critique.confidence < 0.95 {
            rounds += 1;
            info!(round = %rounds, "Refining plan through critique loop");
            current_design = format!(
                "{} (Refinement round {} resolving issue: {})",
                current_design, rounds, critique.issue
            );
            critique = critic.critique(&current_design)?;
        }

        let dag = synthesizer.synthesize(&current_design, &critique)?;
        Ok(dag)
    }
}

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::cognition::graph::{CognitiveExecutionGraph, CognitiveNodeType, EdgeCondition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostGovernorConfig {
    pub max_speculative_branches: usize,
    pub max_recovery_depth: usize,
    pub session_cost_ceiling_usd: f64,
    pub max_parallel_verifiers: usize,
    pub active_pruning_enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CognitiveBudgetClass {
    Minimal,
    Standard,
    Intensive,
    Forensic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionMetric {
    pub node_id: String,
    pub node_type: CognitiveNodeType,
    pub provider_name: String,
    pub duration_ms: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
    pub success: bool,
    pub verifier_roi: f64,
    pub semantic_contribution_score: f64,
    pub replay_relevance: f64,
    pub drift_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeExecutionProfile {
    pub edge_id: String,
    pub traversal_success_rate: f64,
    pub avg_cost_usd: f64,
    pub avg_latency_ms: u64,
    pub semantic_success_rate: f64,
    pub rollback_trigger_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMotif {
    pub motif_id: String,
    pub problem_archetype: String,
    pub optimal_nodes: Vec<CognitiveNodeType>,
    pub optimal_edges: Vec<(String, String)>,
    pub success_count: u64,
    pub avg_cost_usd: f64,
    pub avg_latency_ms: u64,
    pub semantic_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReplayManifest {
    pub original_graph_hash: String,
    pub optimized_graph_hash: String,
    pub pruned_nodes: Vec<String>,
    pub compressed_subgraphs: Vec<String>,
    pub edge_weight_mutations: Vec<String>,
    pub optimization_reasoning_hash: String,
}

// ----------------------------------------------------
// Adaptive Graph Pruner & Compressor Engine
// ----------------------------------------------------

pub struct GraphPruner;

impl GraphPruner {
    pub fn new() -> Self {
        Self
    }

    /// Prunes verifier nodes from the graph if their verifier ROI is 0.0
    pub fn prune_graph(
        &self,
        graph: &mut CognitiveExecutionGraph,
        metrics: &[NodeExecutionMetric],
    ) -> usize {
        let mut pruned_count = 0;
        
        // Collect node IDs that have 0.0 ROI
        let zero_roi_nodes: Vec<String> = metrics.iter()
            .filter(|m| m.node_type == CognitiveNodeType::Verify && m.verifier_roi == 0.0)
            .map(|m| m.node_id.clone())
            .collect();

        for node_id in zero_roi_nodes {
            if graph.nodes.contains_key(&node_id) {
                graph.nodes.remove(&node_id);
                // Remove adjacent edges
                graph.edges.retain(|e| e.from != node_id && e.to != node_id);
                pruned_count += 1;
            }
        }

        pruned_count
    }

    /// Mutates edge weights dynamically based on historical failure rates.
    /// If failure rates are high, rewrites conditions to direct towards rollback recovery.
    pub fn adjust_routing_weights(
        &self,
        graph: &mut CognitiveExecutionGraph,
        profiles: &[EdgeExecutionProfile],
    ) -> usize {
        let mut mutation_count = 0;

        for profile in profiles {
            if profile.rollback_trigger_rate > 0.5 {
                // If this edge causes rollbacks 50%+ of the time, mute it to prevent failure loops
                for edge in &mut graph.edges {
                    let current_edge_id = format!("{}_to_{}", edge.from, edge.to);
                    if current_edge_id == profile.edge_id {
                        edge.condition = EdgeCondition::OnProviderDrift; // Reroute condition
                        mutation_count += 1;
                    }
                }
            }
        }

        mutation_count
    }
}

pub struct GraphCompressionEngine;

impl GraphCompressionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Merges multiple sequential verification/repair steps into a single compressed Node
    pub fn compress_subgraph(
        &self,
        graph: &mut CognitiveExecutionGraph,
        target_nodes: &[String],
        compressed_node_id: &str,
    ) -> bool {
        if target_nodes.len() < 2 {
            return false;
        }

        // Verify all target nodes exist in the graph
        for node in target_nodes {
            if !graph.nodes.contains_key(node) {
                return false;
            }
        }

        // Remove the target nodes and create a new unified Compressed Node
        let mut base_node = graph.nodes.get(&target_nodes[0]).unwrap().clone();
        base_node.node_id = compressed_node_id.to_string();
        base_node.node_type = CognitiveNodeType::Verify;
        
        for node in target_nodes {
            graph.nodes.remove(node);
        }

        graph.nodes.insert(compressed_node_id.to_string(), base_node);

        // Adjust edges: any edge pointing to the first target node now points to the compressed node
        // Any edge pointing out from the last target node now points out from the compressed node
        let first_id = &target_nodes[0];
        let last_id = &target_nodes[target_nodes.len() - 1];

        for edge in &mut graph.edges {
            if &edge.to == first_id {
                edge.to = compressed_node_id.to_string();
            }
            if &edge.from == last_id {
                edge.from = compressed_node_id.to_string();
            }
        }

        // Retain only valid non-self edges
        graph.edges.retain(|e| e.from != e.to);

        true
    }
}

pub struct GraphMotifRegistry {
    motifs: HashMap<String, GraphMotif>,
}

impl GraphMotifRegistry {
    pub fn new() -> Self {
        let mut registry = Self { motifs: HashMap::new() };
        
        // Register default specialized motifs
        registry.register_motif(GraphMotif {
            motif_id: "motif_auth_regression".to_string(),
            problem_archetype: "AuthRegression".to_string(),
            optimal_nodes: vec![CognitiveNodeType::Inspect, CognitiveNodeType::Verify, CognitiveNodeType::SandboxReplay],
            optimal_edges: vec![("Inspect".to_string(), "Verify".to_string()), ("Verify".to_string(), "SandboxReplay".to_string())],
            success_count: 120,
            avg_cost_usd: 0.12,
            avg_latency_ms: 1200,
            semantic_success_rate: 0.98,
        });

        registry.register_motif(GraphMotif {
            motif_id: "motif_type_weakening".to_string(),
            problem_archetype: "TypeWeakening".to_string(),
            optimal_nodes: vec![CognitiveNodeType::Inspect, CognitiveNodeType::Repair, CognitiveNodeType::Verify],
            optimal_edges: vec![("Inspect".to_string(), "Repair".to_string()), ("Repair".to_string(), "Verify".to_string())],
            success_count: 85,
            avg_cost_usd: 0.08,
            avg_latency_ms: 950,
            semantic_success_rate: 0.95,
        });

        registry
    }

    pub fn register_motif(&mut self, motif: GraphMotif) {
        self.motifs.insert(motif.problem_archetype.clone(), motif);
    }

    pub fn get_motif(&self, archetype: &str) -> Option<&GraphMotif> {
        self.motifs.get(archetype)
    }
}

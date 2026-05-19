use crate::intelligence::graph::SemanticGraph;

pub struct CausalAnatomyMap;

impl CausalAnatomyMap {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates structural dependency blast radius and risk attenuation over multi-hop chains.
    /// Traverses functional dependency paths, compounding uncertainty and attenuating risk per hop.
    pub fn propagate_blast_radius(
        &self,
        semantic_graph: &SemanticGraph,
        start_node_id: &str,
        depth_limit: u32,
    ) -> (f64, f64) {
        if !semantic_graph.nodes.contains_key(start_node_id) {
            return (0.0, 0.0);
        }

        let mut cumulative_risk = 0.50; // Starting baseline node risk
        let mut compounded_uncertainty = 0.10; // Starting baseline node uncertainty
        let mut active_nodes = vec![start_node_id.to_string()];
        let mut visited = std::collections::HashSet::new();
        visited.insert(start_node_id.to_string());

        for depth in 1..=depth_limit {
            let mut next_layer = Vec::new();

            for node_id in &active_nodes {
                if let Some(edges) = semantic_graph.edges.get(node_id) {
                    for edge in edges.value() {
                        if !visited.contains(&edge.to_id) {
                            visited.insert(edge.to_id.clone());
                            next_layer.push(edge.to_id.clone());

                            // Attenuation factor: risk decays per hop (0.70^depth)
                            let attenuation = 0.70_f64.powi(depth as i32);
                            cumulative_risk += 0.30 * attenuation;

                             // Uncertainty increases per hop (compounds by 15% absolute per step)
                             compounded_uncertainty = (compounded_uncertainty + 0.15_f64).min(1.0_f64);
                        }
                    }
                }
            }

            if next_layer.is_empty() {
                break;
            }
            active_nodes = next_layer;
        }

        (cumulative_risk.min(1.0), compounded_uncertainty)
    }
}

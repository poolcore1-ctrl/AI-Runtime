use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use crate::intelligence::symbols::{Symbol, SymbolKind};
use tracing::{debug, instrument};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    File,
    Function,
    Struct,
    Class,
    Interface,
    Component,
    Hook,
    Route,
    Module,
}

impl From<SymbolKind> for NodeKind {
    fn from(kind: SymbolKind) -> Self {
        match kind {
            SymbolKind::Function => NodeKind::Function,
            SymbolKind::Struct => NodeKind::Struct,
            SymbolKind::Class => NodeKind::Class,
            SymbolKind::Interface => NodeKind::Interface,
            SymbolKind::Export => NodeKind::Module, // Or whatever makes sense contextually
            SymbolKind::Import => NodeKind::Module,
            SymbolKind::Hook => NodeKind::Hook,
            SymbolKind::Route => NodeKind::Route,
            SymbolKind::Component => NodeKind::Component,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeKind {
    Imports,
    Exports,
    Calls,
    Owns,
    Extends,
    Implements,
    Renders,
    UsesHook,
    DependsOn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String, // Stable ID: file_path:symbol_name:symbol_kind
    pub kind: NodeKind,
    pub name: String,
    pub file_path: String,
    pub original_symbol: Option<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from_id: String,
    pub to_id: String,
    pub kind: EdgeKind,
}

/// The Repository Cognition Graph
/// Models symbols, ownership, dependencies, and semantic relationships.
pub struct SemanticGraph {
    pub nodes: DashMap<String, GraphNode>,
    pub edges: DashMap<String, Vec<GraphEdge>>,
}

impl SemanticGraph {
    pub fn new() -> Self {
        Self {
            nodes: DashMap::new(),
            edges: DashMap::new(),
        }
    }

    /// Generates a stable, deterministic ID for a node
    pub fn generate_node_id(file_path: &str, name: &str, kind: &NodeKind) -> String {
        format!("{}:{}:{:?}", file_path, name, kind)
    }

    #[instrument(skip(self, symbol))]
    pub fn add_symbol_node(&self, symbol: Symbol) -> String {
        let kind = NodeKind::from(symbol.kind.clone());
        let id = Self::generate_node_id(&symbol.file_path, &symbol.name, &kind);
        
        let node = GraphNode {
            id: id.clone(),
            kind,
            name: symbol.name.clone(),
            file_path: symbol.file_path.clone(),
            original_symbol: Some(symbol),
        };
        
        self.nodes.insert(id.clone(), node);
        id
    }

    #[instrument(skip(self))]
    pub fn add_edge(&self, from_id: String, to_id: String, kind: EdgeKind) {
        debug!(from = %from_id, to = %to_id, edge_type = ?kind, "Adding semantic edge");
        let edge = GraphEdge {
            from_id: from_id.clone(),
            to_id,
            kind,
        };
        
        self.edges.entry(from_id).or_insert_with(Vec::new).push(edge);
    }
}

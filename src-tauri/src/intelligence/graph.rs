use std::collections::HashMap;
use crate::intelligence::symbols::Symbol;

pub struct DependencyGraph {
    pub nodes: HashMap<String, Symbol>,
    pub edges: Vec<(String, String)>, // "from" depends on "to"
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }
}

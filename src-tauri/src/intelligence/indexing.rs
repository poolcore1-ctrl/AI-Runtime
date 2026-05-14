use dashmap::DashMap;
use crate::intelligence::symbols::Symbol;

/// A specialized index for fast symbol lookup across the repository.
pub struct RepositoryIndex {
    // Maps symbol name -> List of symbols (name can be duplicated across files)
    pub symbols_by_name: DashMap<String, Vec<Symbol>>,
}

impl RepositoryIndex {
    pub fn new() -> Self {
        Self {
            symbols_by_name: DashMap::new(),
        }
    }

    /// Adds a symbol to the index.
    pub fn add_symbol(&self, symbol: Symbol) {
        self.symbols_by_name
            .entry(symbol.name.clone())
            .or_insert_with(Vec::new)
            .push(symbol);
    }

    /// Finds symbols by name.
    pub fn find_by_name(&self, name: &str) -> Vec<Symbol> {
        self.symbols_by_name
            .get(name)
            .map(|v| v.clone())
            .unwrap_or_default()
    }
}

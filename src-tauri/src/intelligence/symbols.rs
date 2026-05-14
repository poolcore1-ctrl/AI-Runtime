use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Class,
    Interface,
    Export,
    Import,
    Hook,
    Route,
    Component,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    // E.g., for functions, what does it take and return?
    pub signature: Option<String>,
}

/// Extractor to pull semantic symbols from a raw AST
pub struct SymbolExtractor;

impl SymbolExtractor {
    pub fn new() -> Self {
        Self
    }
    // We will add tree-sitter Query logic here
}

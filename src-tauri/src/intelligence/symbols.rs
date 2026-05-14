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

use tree_sitter::{Query, QueryCursor, Tree, Node};
use anyhow::{Result, anyhow};
use crate::intelligence::ast::SupportedLanguage;

/// Extractor to pull semantic symbols from a raw AST
pub struct SymbolExtractor {
    rust_query: Query,
    ts_query: Query,
    tsx_query: Query,
}

impl SymbolExtractor {
    pub fn new() -> Result<Self> {
        let rust_lang = SupportedLanguage::Rust.get_language();
        let ts_lang = SupportedLanguage::TypeScript.get_language();
        let tsx_lang = SupportedLanguage::TSX.get_language();
        
        let rust_query_str = r#"
            (function_item name: (identifier) @name) @function
            (struct_item name: (type_identifier) @name) @struct
        "#;

        let ts_query_str = r#"
            (function_declaration name: (identifier) @name) @function
            (class_declaration name: (type_identifier) @name) @class
            (interface_declaration name: (type_identifier) @name) @interface
            (export_statement declaration: (_) @export)
        "#;
        
        let rust_query = Query::new(&rust_lang, rust_query_str)
            .map_err(|e| anyhow!("Failed to compile Rust query: {}", e))?;
            
        let ts_query = Query::new(&ts_lang, ts_query_str)
            .map_err(|e| anyhow!("Failed to compile TypeScript query: {}", e))?;
            
        let tsx_query = Query::new(&tsx_lang, ts_query_str)
            .map_err(|e| anyhow!("Failed to compile TSX query: {}", e))?;

        Ok(Self { rust_query, ts_query, tsx_query })
    }

    pub fn extract_rust_symbols(&self, tree: &Tree, source_code: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(&self.rust_query, tree.root_node(), source_code.as_bytes());
        
        for m in matches {
            let mut name = String::new();
            let mut kind = SymbolKind::Function;
            let mut node_opt: Option<Node> = None;

            for capture in m.captures {
                let capture_name: &str = &self.rust_query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        if let Ok(text) = capture.node.utf8_text(source_code.as_bytes()) {
                            name = text.to_string();
                        }
                    }
                    "function" => {
                        kind = SymbolKind::Function;
                        node_opt = Some(capture.node);
                    }
                    "struct" => {
                        kind = SymbolKind::Struct;
                        node_opt = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let Some(node) = node_opt {
                if !name.is_empty() {
                    symbols.push(Symbol {
                        name,
                        kind,
                        file_path: file_path.to_string(),
                        start_line: node.start_position().row + 1, // 1-indexed
                        end_line: node.end_position().row + 1,
                        signature: None, // Can be refined later
                    });
                }
            }
        }
        
        symbols
    }

    pub fn extract_ts_symbols(&self, tree: &Tree, source_code: &str, file_path: &str, is_tsx: bool) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut cursor = QueryCursor::new();
        
        let query = if is_tsx { &self.tsx_query } else { &self.ts_query };
        let matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());
        
        for m in matches {
            let mut name = String::new();
            let mut kind = SymbolKind::Function;
            let mut node_opt: Option<Node> = None;

            for capture in m.captures {
                let capture_name: &str = &query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        if let Ok(text) = capture.node.utf8_text(source_code.as_bytes()) {
                            name = text.to_string();
                        }
                    }
                    "function" => {
                        kind = SymbolKind::Function;
                        node_opt = Some(capture.node);
                    }
                    "class" => {
                        kind = SymbolKind::Class;
                        node_opt = Some(capture.node);
                    }
                    "interface" => {
                        kind = SymbolKind::Interface;
                        node_opt = Some(capture.node);
                    }
                    "export" => {
                        kind = SymbolKind::Export;
                        node_opt = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let Some(node) = node_opt {
                if !name.is_empty() || matches!(kind, SymbolKind::Export) {
                    // For exports, we might not always have a generic "name" easily extracted 
                    // depending on the syntax (e.g., export default function() {}), 
                    // but we capture the semantic intent.
                    let symbol_name = if name.is_empty() { "anonymous_export".to_string() } else { name };
                    
                    symbols.push(Symbol {
                        name: symbol_name,
                        kind,
                        file_path: file_path.to_string(),
                        start_line: node.start_position().row + 1,
                        end_line: node.end_position().row + 1,
                        signature: None,
                    });
                }
            }
        }
        
        symbols
    }
}

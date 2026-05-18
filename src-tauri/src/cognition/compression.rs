use crate::intelligence::graph::SemanticGraph;
use anyhow::Result;
use std::fs;
use std::collections::HashSet;
use tracing::{info, debug, instrument};

pub struct ContextCompressor;

impl ContextCompressor {
    pub fn new() -> Self { Self }

    #[instrument(skip(self, graph))]
    pub fn compress_context(&self, query: &str, graph: &SemanticGraph, max_chars: usize) -> Result<String> {
        info!(query = %query, max_chars = max_chars, "Compressing repository context");

        let query_terms: Vec<String> = query.to_lowercase()
            .split_whitespace()
            .map(|s| s.replace(|c: char| !c.is_alphanumeric(), ""))
            .filter(|s| !s.is_empty())
            .collect();

        let mut matched_symbols = Vec::new();
        let mut seen_keys = HashSet::new();

        // 1. Identify relevant symbol nodes in semantic graph
        for entry in graph.nodes.iter() {
            let node = entry.value();
            let name_lower = node.name.to_lowercase();
            let file_lower = node.file_path.to_lowercase();

            // Match if query terms intersect symbol name or file path
            let is_match = query_terms.iter().any(|term| {
                name_lower.contains(term) || file_lower.contains(term)
            }) || query_terms.is_empty();

            if is_match {
                if let Some(ref sym) = node.original_symbol {
                    let key = format!("{}:{}:{}", sym.file_path, sym.start_line, sym.end_line);
                    if !seen_keys.contains(&key) {
                        seen_keys.insert(key);
                        matched_symbols.push(sym.clone());
                    }
                }
            }
        }

        // Limit matches to avoid over-indexing
        matched_symbols.truncate(15);

        // 2. Extract code blocks for matched symbols
        let mut compressed = String::new();
        compressed.push_str("# Compressed Cognitive Context\n\n");
        compressed.push_str("The following high-value symbol blocks were semantically matched and compressed:\n\n");

        for sym in matched_symbols {
            if compressed.len() >= max_chars {
                debug!("Reached maximum character limit during context compression.");
                break;
            }

            // Extract file name
            let file_name = std::path::Path::new(&sym.file_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&sym.file_path);

            match fs::read_to_string(&sym.file_path) {
                Ok(content) => {
                    let lines: Vec<&str> = content.lines().collect();
                    if sym.start_line > 0 && sym.start_line <= lines.len() {
                        let end_idx = sym.end_line.min(lines.len());
                        let start_idx = sym.start_line - 1; // 1-indexed conversion
                        
                        let sliced_lines = &lines[start_idx..end_idx];
                        let code_block = sliced_lines.join("\n");

                        let block_desc = format!(
                            "## Symbol outline: `{}` in [{}]({})\n\n```typescript\n{}\n```\n\n",
                            sym.name, file_name, sym.file_path, code_block
                        );

                        if compressed.len() + block_desc.len() <= max_chars {
                            compressed.push_str(&block_desc);
                        }
                    }
                }
                Err(err) => {
                    debug!(path = %sym.file_path, err = %err, "Failed to read file for symbol context extraction");
                }
            }
        }

        if compressed.len() <= 100 {
            compressed.push_str("No highly-relevant semantic symbol blocks found in active workspaces.\n");
        }

        Ok(compressed)
    }
}

use tree_sitter::{Parser, Language, Tree};
use anyhow::{Result, anyhow};

/// Supported parsing languages
pub enum SupportedLanguage {
    Rust,
    TypeScript,
    TSX,
}

impl SupportedLanguage {
    pub fn get_language(&self) -> Language {
        match self {
            SupportedLanguage::Rust => tree_sitter_rust::language(),
            SupportedLanguage::TypeScript => tree_sitter_typescript::language_typescript(),
            SupportedLanguage::TSX => tree_sitter_typescript::language_tsx(),
        }
    }
}

/// A wrapper around Tree-sitter to parse code into raw ASTs.
/// We do NOT persist these raw trees long-term, they are only used to extract semantic symbols.
pub struct AstParser {
    parser: Parser,
}

impl AstParser {
    pub fn new(lang: SupportedLanguage) -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&lang.get_language())
            .map_err(|e| anyhow!("Failed to set language: {}", e))?;
        Ok(Self { parser })
    }

    /// Parse a source file into a Tree.
    /// In the future, this will accept an `old_tree` for incremental parsing.
    pub fn parse(&mut self, source_code: &str, old_tree: Option<&Tree>) -> Option<Tree> {
        self.parser.parse(source_code, old_tree)
    }
}

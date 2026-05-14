use crate::intelligence::indexing::RepositoryIndex;
use crate::intelligence::symbols::Symbol;
use std::sync::Arc;

/// The search and reasoning engine for the Repository Intelligence Layer.
/// Answers the question: "What parts of the repository are relevant to this task?"
pub struct CognitiveRetrieval {
    pub index: Arc<RepositoryIndex>,
}

impl CognitiveRetrieval {
    pub fn new(index: Arc<RepositoryIndex>) -> Self {
        Self { index }
    }

    /// Searches for symbols by name across the entire repository index.
    pub fn find_relevant_symbols(&self, query: &str) -> Vec<Symbol> {
        // For now, this is a simple exact name match.
        // In the future, this will use vector search or fuzzy matching.
        self.index.find_by_name(query)
    }
}

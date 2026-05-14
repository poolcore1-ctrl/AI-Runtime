pub mod ast;
pub mod symbols;
pub mod graph;
pub mod indexing;
pub mod retrieval;
pub mod incremental;

use tracing::{info, debug, instrument};
use anyhow::Result;

use crate::intelligence::ast::{AstParser, SupportedLanguage};
use crate::intelligence::symbols::SymbolExtractor;
use crate::intelligence::graph::SemanticGraph;
use crate::intelligence::incremental::IncrementalTracker;
use crate::intelligence::indexing::RepositoryIndex;
use crate::intelligence::retrieval::CognitiveRetrieval;
use std::sync::Arc;

/// The Repository Intelligence Engine
/// This acts as the perception system for the ASOS,
/// converting raw code into semantic, structural understanding.
pub struct IntelligenceEngine {
    pub graph: Arc<SemanticGraph>,
    pub extractor: Arc<SymbolExtractor>,
    pub tracker: Arc<IncrementalTracker>,
    pub index: Arc<RepositoryIndex>,
    pub retrieval: Arc<CognitiveRetrieval>,
}

impl IntelligenceEngine {
    pub fn new() -> Result<Self> {
        let index = Arc::new(RepositoryIndex::new());
        Ok(Self {
            graph: Arc::new(SemanticGraph::new()),
            extractor: Arc::new(SymbolExtractor::new()?),
            tracker: Arc::new(IncrementalTracker::new()),
            index: index.clone(),
            retrieval: Arc::new(CognitiveRetrieval::new(index)),
        })
    }

    /// The main entry point for the incremental pipeline:
    /// Filesystem Watcher -> Incremental Parser -> Symbol Extractor -> Semantic Graph Builder -> Repository Index
    #[instrument(skip(self))]
    pub async fn process_repository(&self, project_path: &str) -> Result<()> {
        info!(project_path = %project_path, "Starting repository intelligence processing");
        // Pipeline integration will go here
        Ok(())
    }

    #[instrument(skip(self, source_code))]
    pub fn process_file(&self, file_path: &str, source_code: &str, lang: SupportedLanguage, is_tsx: bool) -> Result<()> {
        if !self.tracker.has_changed(file_path, source_code) {
            debug!(file_path = %file_path, "File unchanged, skipping cognitive pipeline");
            return Ok(());
        }

        debug!(file_path = %file_path, "Processing file through cognitive pipeline");
        
        // 1. Parsing
        let mut parser = AstParser::new(lang)?;
        let tree = parser.parse(source_code, None).ok_or_else(|| anyhow::anyhow!("Failed to parse file"))?;

        // 2. Symbol Extraction
        let symbols = if file_path.ends_with(".rs") {
            self.extractor.extract_rust_symbols(&tree, source_code, file_path)
        } else {
            self.extractor.extract_ts_symbols(&tree, source_code, file_path, is_tsx)
        };

        debug!(symbol_count = symbols.len(), "Extracted symbols");

        // 3. Graph Construction & Indexing
        for symbol in symbols {
            self.graph.add_symbol_node(symbol.clone());
            self.index.add_symbol(symbol);
        }

        // Future: 4. Dependency Extraction (Edges)
        
        Ok(())
    }
}

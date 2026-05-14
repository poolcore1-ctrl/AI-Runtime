use dashmap::DashMap;
use sha2::{Sha256, Digest};

/// Tracks file modifications to prevent redundant parsing and extraction.
pub struct IncrementalTracker {
    // Maps file_path -> SHA256 hash of the content
    pub file_hashes: DashMap<String, String>,
}

impl IncrementalTracker {
    pub fn new() -> Self {
        Self {
            file_hashes: DashMap::new(),
        }
    }

    /// Computes the SHA256 hash of the given source code.
    pub fn compute_hash(source_code: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source_code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Checks if a file has changed based on its previous hash.
    /// Returns true if the file is new or its content has changed.
    /// If it has changed, it updates the stored hash.
    pub fn has_changed(&self, file_path: &str, source_code: &str) -> bool {
        let new_hash = Self::compute_hash(source_code);
        
        if let Some(old_hash) = self.file_hashes.get(file_path) {
            if *old_hash.value() == new_hash {
                return false; // No change
            }
        }
        
        self.file_hashes.insert(file_path.to_string(), new_hash);
        true
    }
}

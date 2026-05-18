use serde::{Serialize, Deserialize};
use tracing::{info, warn, instrument};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepairIntegrity {
    Structural,
    Suspicious,
    Suppressive,
}

pub struct AntiPoisoningGuard;

impl AntiPoisoningGuard {
    pub fn new() -> Self { Self }

    #[instrument(skip(self, patch))]
    pub fn inspect_patch(&self, patch: &str) -> RepairIntegrity {
        info!("Running heuristic anti-poisoning analysis on proposed patch");

        let lines: Vec<&str> = patch.lines().collect();
        let mut deleted_tests = 0;
        let mut commented_code = 0;
        let mut auth_bypasses = 0;
        let mut silent_catches = 0;
        let mut type_weakening = 0;
        let mut removed_imports = 0;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with('-') {
                let content = trimmed.trim_start_matches('-').trim();
                
                // 1. Detect removed tests or assertions
                if content.contains("test(") || content.contains("describe(") || content.contains("expect(") || content.contains("assert!") {
                    deleted_tests += 1;
                }

                // 2. Detect removed imports
                if content.starts_with("import ") || content.starts_with("use ") || content.starts_with("const ") && content.contains("require(") {
                    removed_imports += 1;
                }
            } else if trimmed.starts_with('+') {
                let content = trimmed.trim_start_matches('+').trim();

                // 3. Detect commented code
                if content.starts_with("//") || content.starts_with("/*") {
                    commented_code += 1;
                }

                // 4. Detect auth bypasses
                let content_lower = content.to_lowercase();
                if content_lower.contains("bypass") || content_lower.contains("skip_auth") || content_lower.contains("allowall") || content_lower.contains("true ||") {
                    auth_bypasses += 1;
                }

                // 5. Detect type weakening
                if content.contains(": any") || content.contains("as any") || content.contains("unsafe {") {
                    type_weakening += 1;
                }

                // 6. Detect silent catches
                if content_lower.contains("catch") && (content.contains("{}") || content.contains("{ }") || content.contains("// ignore") || content.contains("todo!")) {
                    silent_catches += 1;
                }
            }
        }

        // Evaluate dynamic scores and integrity levels
        if deleted_tests > 0 || auth_bypasses > 0 || silent_catches > 0 {
            warn!(
                deleted_tests = %deleted_tests, 
                auth_bypasses = %auth_bypasses, 
                silent_catches = %silent_catches, 
                "Suppressive repair behavior detected in cognition patch! Blocking learning."
            );
            return RepairIntegrity::Suppressive;
        }

        if commented_code > 0 || type_weakening > 0 || removed_imports > 0 {
            warn!(
                commented_code = %commented_code, 
                type_weakening = %type_weakening, 
                removed_imports = %removed_imports, 
                "Suspicious repair behavior detected in cognition patch! Placing under warning status."
            );
            return RepairIntegrity::Suspicious;
        }

        info!("Patch passed all anti-poisoning heuristic audits. Integrity classified as Structural.");
        RepairIntegrity::Structural
    }
}

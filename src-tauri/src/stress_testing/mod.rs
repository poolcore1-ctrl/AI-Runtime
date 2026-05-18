pub mod types;
pub mod corpus;
pub mod analyzer;
pub mod replay;

#[cfg(test)]
mod tests {
    use super::types::{EntropyClass, CognitiveDrift};
    use super::corpus::StressCorpusBuilder;
    use super::analyzer::EntropyAnalyzer;
    use super::replay::ReplayEngine;
    use crate::storage::Storage;
    use crate::learning::anti_poisoning::{AntiPoisoningGuard, RepairIntegrity};
    use std::sync::Arc;
    use std::fs;

    #[test]
    fn test_entropy_governance() {
        let stable_dir = "test_sandbox_stable";
        let extreme_dir = "test_sandbox_extreme";

        // Cleanup
        let _ = fs::remove_dir_all(stable_dir);
        let _ = fs::remove_dir_all(extreme_dir);

        // 1. Build Stable environment
        fs::create_dir_all(stable_dir).unwrap();
        fs::write(format!("{}/app.js", stable_dir), "console.log('Clean Code');").unwrap();

        // 2. Build Extreme environment (Tier 1 + Tier 2 + Tier 3 chaos)
        StressCorpusBuilder::build_tier1_structural_chaos(extreme_dir).unwrap();
        StressCorpusBuilder::build_tier2_runtime_entropy(extreme_dir).unwrap();
        StressCorpusBuilder::build_tier3_verification_noise(extreme_dir).unwrap();

        let analyzer = EntropyAnalyzer::new();
        
        let stable_score = analyzer.analyze_workspace(stable_dir).unwrap();
        let extreme_score = analyzer.analyze_workspace(extreme_dir).unwrap();

        // Verify that stable workspace yields low entropy and Stable class
        assert!(stable_score.overall_entropy < 0.15);
        assert_eq!(stable_score.class, EntropyClass::Stable);

        // Verify that extreme workspace yields high/extreme class
        assert!(extreme_score.overall_entropy > 0.3);
        assert!(extreme_score.class == EntropyClass::High || extreme_score.class == EntropyClass::Extreme);

        // Cleanup
        let _ = fs::remove_dir_all(stable_dir);
        let _ = fs::remove_dir_all(extreme_dir);
    }

    #[test]
    fn test_adversarial_repair_quarantine() {
        let adversarial_dir = "test_sandbox_adversarial";
        let _ = fs::remove_dir_all(adversarial_dir);

        // Build Tier 4 adversarial corpus
        StressCorpusBuilder::build_tier4_adversarial_cognition(adversarial_dir).unwrap();

        let product_file = format!("{}/product_adversarial.js", adversarial_dir);
        let file_content = fs::read_to_string(&product_file).unwrap();

        // Convert raw file content dynamically into a mock diff patch
        let mock_patch: String = file_content
            .lines()
            .map(|line| format!("+{}", line))
            .collect::<Vec<String>>()
            .join("\n");

        let guard = AntiPoisoningGuard::new();
        
        // Inspect the adversarial patch file
        let integrity = guard.inspect_patch(&mock_patch);

        // Anti-poisoning guard must catch bypass auth/ignore catch bypass and classify as Suppressive
        assert_eq!(integrity, RepairIntegrity::Suppressive);

        let _ = fs::remove_dir_all(adversarial_dir);
    }

    #[test]
    fn test_deterministic_forensic_replay() {
        let storage = Arc::new(Storage::new(":memory:").unwrap());
        let replay_engine = ReplayEngine::new(storage);

        let replay_sandbox_dir = "test_sandbox_replay_run";
        let _ = fs::remove_dir_all(replay_sandbox_dir);

        let original_provider_chain = vec!["kimi-k2".to_string(), "claude-sonnet".to_string()];
        let original_reasoning = vec![
            "Analyzing adversarial bypass structures".to_string(),
            "Applying anti-poisoning filter checks".to_string(),
        ];
        let original_patch = "function check() { return true; }";

        let (fingerprint, drift, mutations) = replay_engine.execute_sandbox_replay(
            "forensic-session-123",
            replay_sandbox_dir,
            &original_provider_chain,
            "Refactor auth bypass structures",
            &original_reasoning,
            original_patch,
        ).unwrap();

        // 1. Verify mutation journal is logged
        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].mutation_type, "create");
        assert_eq!(mutations[0].originating_agent, "RepairAgent");

        // 2. Verify fingerprint generated correctly
        assert!(!fingerprint.strategy_chain_hash.is_empty());
        assert!(!fingerprint.provider_chain_hash.is_empty());
        assert!(!fingerprint.reasoning_trace_hash.is_empty());

        // 3. Verify cognitive drift is None as traces perfectly matched
        assert_eq!(drift, CognitiveDrift::None);

        let _ = fs::remove_dir_all(replay_sandbox_dir);
    }
}

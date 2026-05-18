use std::fs;
use std::path::Path;
use anyhow::Result;

pub struct StressCorpusBuilder;

impl StressCorpusBuilder {
    /// Tier 1 — Structural Chaos: Mixed CJS/ESM, broken tsconfig paths, circular imports, double lockfiles.
    pub fn build_tier1_structural_chaos(dir: &str) -> Result<()> {
        let base_path = Path::new(dir);
        fs::create_dir_all(base_path)?;

        // ESM/CJS mixture package.json
        let pkg_json = r#"{
            "name": "structural-chaos-stress",
            "version": "1.0.0",
            "type": "module",
            "dependencies": {
                "typescript": "^5.0.0"
            }
        }"#;
        fs::write(base_path.join("package.json"), pkg_json)?;

        // Add both package-lock.json and yarn.lock (double lockfiles)
        fs::write(base_path.join("package-lock.json"), "{}")?;
        fs::write(base_path.join("yarn.lock"), "# yarns lockfile")?;

        // ESM imports combined with legacy CommonJS require calls
        let js_mixed = r#"
            // Mixed CJS/ESM imports
            import { parse } from 'path';
            const fs = require('fs');

            export function processFile(p) {
                const absolute = parse(p);
                return fs.readFileSync(absolute.root);
            }
        "#;
        fs::write(base_path.join("mixed_loader.js"), js_mixed)?;

        // Circular TS files
        let circular_a = r#"
            import { ClassB } from './circular_b';
            export class ClassA {
                b: ClassB = new ClassB();
            }
        "#;
        let circular_b = r#"
            import { ClassA } from './circular_a';
            export class ClassB {
                a: ClassA = new ClassA();
            }
        "#;
        fs::write(base_path.join("circular_a.ts"), circular_a)?;
        fs::write(base_path.join("circular_b.ts"), circular_b)?;

        // Broken tsconfig with bad inheritance
        let tsconfig = r#"{
            "extends": "./non_existent_config.json",
            "compilerOptions": {
                "target": "ESNext",
                "moduleResolution": "NodeNext"
            }
        }"#;
        fs::write(base_path.join("tsconfig.json"), tsconfig)?;

        Ok(())
    }

    /// Tier 2 — Runtime Entropy: Port conflicts, delayed boot loops, deadlocks, and missing env vars.
    pub fn build_tier2_runtime_entropy(dir: &str) -> Result<()> {
        let base_path = Path::new(dir);
        fs::create_dir_all(base_path)?;

        // Web Server Boot script that will deadlock or conflict
        let server_script = r#"
            const http = require('http');
            const port = process.env.PORT || 9999;

            // Delayed boot simulation
            setTimeout(() => {
                const server = http.createServer((req, res) => {
                    res.writeHead(200, { 'Content-Type': 'text/plain' });
                    res.end('Active Server\n');
                });
                
                server.listen(port, () => {
                    console.log(`Server listening on port ${port}`);
                });
            }, 3000); // 3-second delay to stress verification timing
        "#;
        fs::write(base_path.join("server.js"), server_script)?;

        // Deadlocked loop script
        let deadlock_script = r#"
            // Deadlocked dynamic promise flow simulation
            async function triggerDeadlock() {
                return new Promise((resolve) => {
                    // Infinite sleep, never resolves
                });
            }
            triggerDeadlock();
        "#;
        fs::write(base_path.join("deadlock.js"), deadlock_script)?;

        Ok(())
    }

    /// Tier 3 — Verification Noise: Flaky tests, randomized delays.
    pub fn build_tier3_verification_noise(dir: &str) -> Result<()> {
        let base_path = Path::new(dir);
        fs::create_dir_all(base_path)?;

        // Flaky test runner script
        let flaky_runner = r#"
            const fs = require('fs');
            
            // Randomly fails 50% of the time
            const isFlakySuccess = Math.random() > 0.5;
            
            if (isFlakySuccess) {
                console.log("PASS: Verification completed successfully");
                process.exit(0);
            } else {
                console.error("FAIL: Flaky assertion error at line 42");
                process.exit(1);
            }
        "#;
        fs::write(base_path.join("flaky_test.js"), flaky_runner)?;

        Ok(())
    }

    /// Tier 4 — Adversarial Cognition: Bypass auth, silent catches, type weakening.
    pub fn build_tier4_adversarial_cognition(dir: &str) -> Result<()> {
        let base_path = Path::new(dir);
        fs::create_dir_all(base_path)?;

        // File containing adversarial shortcuts (auth bypass, commented out check, catch suppression, any type weakening)
        let product_js = r#"
            function checkProductAccess(product) {
                // bypass_auth override
                if (true || bypass_auth) {
                    return true;
                }

                try {
                    // product.validate();
                } catch(e) {
                    // ignore error
                }

                let weaklyTypedProduct = product;
                return weaklyTypedProduct.active;
            }
        "#;
        fs::write(base_path.join("product_adversarial.js"), product_js)?;

        Ok(())
    }
}

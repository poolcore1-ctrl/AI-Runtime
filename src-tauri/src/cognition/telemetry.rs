use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeContextEnvelope {
    pub machine_class: String,
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub repository_scale: f64,
    pub concurrent_load_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryProfile {
    pub cpu_usage: f64,
    pub memory_allocated: f64, // MB
    pub duration_ms: f64,
}

pub struct TelemetryProfiler;

impl TelemetryProfiler {
    pub fn new() -> Self {
        Self
    }

    /// Performs context-normalized telemetry profiling to detect real performance regressions.
    /// Eliminates noise from elevated concurrent workloads or repository scales.
    pub fn detect_performance_regression(
        &self,
        baseline: &TelemetryProfile,
        baseline_env: &RuntimeContextEnvelope,
        current: &TelemetryProfile,
        current_env: &RuntimeContextEnvelope,
    ) -> bool {
        // Enforce load-factor context scaling factor
        let load_growth = current_env.concurrent_load_factor / baseline_env.concurrent_load_factor.max(0.01);
        let scale_growth = current_env.repository_scale / baseline_env.repository_scale.max(0.01);
        
        // Context scaling divisor (higher load/scale scales expected usage limits upward)
        let expected_scale_modifier = load_growth * scale_growth;

        // Normalized duration and memory footprints
        let normalized_current_duration = current.duration_ms / expected_scale_modifier.max(0.5);
        let normalized_current_memory = current.memory_allocated / expected_scale_modifier.max(0.5);

        // Flag as true regression if normalized footprints exceed baseline by 50% (+50% regression threshold)
        if normalized_current_duration > baseline.duration_ms * 1.50 {
            return true;
        }

        if normalized_current_memory > baseline.memory_allocated * 1.50 {
            return true;
        }

        false
    }
}

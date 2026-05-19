use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConcurrencyField {
    pub queue_arrival_rate: f64,
    pub processing_rate: f64,
    pub lock_wait_time_ms: u64,
    pub thread_exhaustion_factor: f64,
    pub memory_pressure_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatentPressureGradient {
    pub pressure_gradient: f64, // (queue_arrival_rate - processing_rate)
    pub instability_index: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchedulerInstabilityIndex {
    pub starvation_likelihood: f64,
    pub priority_inversion_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistributedResonancePattern {
    pub synchronization_drift_ms: u64,
    pub jitter_cascade_risk: f64,
}

impl ConcurrencyField {
    pub fn new(
        queue_arrival_rate: f64,
        processing_rate: f64,
        lock_wait_time_ms: u64,
        thread_exhaustion_factor: f64,
        memory_pressure_factor: f64,
    ) -> Self {
        Self {
            queue_arrival_rate,
            processing_rate,
            lock_wait_time_ms,
            thread_exhaustion_factor,
            memory_pressure_factor,
        }
    }

    /// Pressure Gradient = Queue Arrival Rate - Processing Rate
    pub fn calculate_latent_gradient(&self) -> LatentPressureGradient {
        let pressure_gradient = self.queue_arrival_rate - self.processing_rate;
        // Instability climbs as the gradient increases and threads are exhausted
        let instability_index = if pressure_gradient > 0.0 {
            ((pressure_gradient * 0.40) + (self.thread_exhaustion_factor * 0.60)).min(1.0)
        } else {
            (self.thread_exhaustion_factor * 0.30).min(1.0)
        };

        LatentPressureGradient {
            pressure_gradient,
            instability_index,
        }
    }

    /// Scheduler starvation maps directly to CPU load, lock contention times, and thread exhaustion.
    pub fn estimate_scheduler_instability(&self) -> SchedulerInstabilityIndex {
        let starvation_likelihood = if self.thread_exhaustion_factor > 0.80 {
            (self.thread_exhaustion_factor * 0.90).min(1.0)
        } else {
            self.thread_exhaustion_factor * 0.40
        };

        let priority_inversion_risk = if self.lock_wait_time_ms > 150 {
            (self.lock_wait_time_ms as f64 / 300.0).min(1.0)
        } else {
            (self.lock_wait_time_ms as f64 / 1000.0).min(1.0)
        };

        SchedulerInstabilityIndex {
            starvation_likelihood,
            priority_inversion_risk,
        }
    }

    /// Tracks distributed sync drifts and network/worker scheduling wake queues.
    pub fn estimate_resonance_pattern(&self) -> DistributedResonancePattern {
        let synchronization_drift_ms = (self.lock_wait_time_ms as f64 * self.memory_pressure_factor) as u64;
        let jitter_cascade_risk = if synchronization_drift_ms > 100 {
            ((synchronization_drift_ms as f64 / 250.0) * 0.80).min(1.0)
        } else {
            0.10
        };

        DistributedResonancePattern {
            synchronization_drift_ms,
            jitter_cascade_risk,
        }
    }
}

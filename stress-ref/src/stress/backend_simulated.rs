use super::backend::*;

/// Simulated backend — no-ops. All stress is handled by the Poisson/sinusoidal models.
pub struct SimulatedBackend;

impl SimulatedBackend {
    pub fn new() -> Self { Self }
}

impl StressBackend for SimulatedBackend {
    fn apply_network_degradation(&mut self, _config: &NetworkDegradationConfig) -> Result<(), String> {
        Ok(())  // Simulated stressors handle this internally
    }
    fn remove_network_degradation(&mut self, _interface: &str) -> Result<(), String> {
        Ok(())
    }
    fn apply_resource_pressure(&mut self, _config: &ResourcePressureConfig) -> Result<(), String> {
        Ok(())
    }
    fn remove_resource_pressure(&mut self, _cgroup_path: &str) -> Result<(), String> {
        Ok(())
    }
    fn inject_memory_stress(&mut self, _config: &MemoryStressConfig) -> Result<(), String> {
        Ok(())
    }
    fn remove_memory_stress(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn pause_workload(&mut self, _pid: u32) -> Result<(), String> {
        Ok(())
    }
    fn resume_workload(&mut self, _pid: u32) -> Result<(), String> {
        Ok(())
    }
    fn apply_network_partition(&mut self, _interface: &str) -> Result<(), String> {
        Ok(())
    }
    fn remove_network_partition(&mut self, _interface: &str) -> Result<(), String> {
        Ok(())
    }
    fn name(&self) -> &str { "simulated" }
}

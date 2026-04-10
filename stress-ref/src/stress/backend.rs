use serde::{Deserialize, Serialize};

/// Configuration for network degradation via Linux tc/netem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDegradationConfig {
    pub interface: String,
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub loss_percent: f64,
}

/// Configuration for resource pressure via Linux cgroups v2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePressureConfig {
    pub cgroup_path: String,
    pub memory_limit_bytes: Option<u64>,
    pub cpu_quota_us: Option<u64>,
    pub cpu_period_us: Option<u64>,
}

pub struct MemoryStressConfig {
    pub vm_bytes: u64,
    pub vm_method: String,
}

impl Default for MemoryStressConfig {
    fn default() -> Self {
        Self { vm_bytes: 268_435_456, vm_method: "flip".to_string() }
    }
}

/// Abstraction over simulated vs real stress injection.
pub trait StressBackend: Send {
    fn apply_network_degradation(&mut self, config: &NetworkDegradationConfig) -> Result<(), String>;
    fn remove_network_degradation(&mut self, interface: &str) -> Result<(), String>;
    fn apply_resource_pressure(&mut self, config: &ResourcePressureConfig) -> Result<(), String>;
    fn remove_resource_pressure(&mut self, cgroup_path: &str) -> Result<(), String>;
    fn inject_memory_stress(&mut self, config: &MemoryStressConfig) -> Result<(), String>;
    fn remove_memory_stress(&mut self) -> Result<(), String>;
    fn pause_workload(&mut self, pid: u32) -> Result<(), String>;
    fn resume_workload(&mut self, pid: u32) -> Result<(), String>;
    fn apply_network_partition(&mut self, interface: &str) -> Result<(), String>;
    fn remove_network_partition(&mut self, interface: &str) -> Result<(), String>;
    fn name(&self) -> &str;
}

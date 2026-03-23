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

/// Abstraction over simulated vs real stress injection.
pub trait StressBackend: Send {
    fn apply_network_degradation(&mut self, config: &NetworkDegradationConfig) -> Result<(), String>;
    fn remove_network_degradation(&mut self, interface: &str) -> Result<(), String>;
    fn apply_resource_pressure(&mut self, config: &ResourcePressureConfig) -> Result<(), String>;
    fn remove_resource_pressure(&mut self, cgroup_path: &str) -> Result<(), String>;
    fn name(&self) -> &str;
}

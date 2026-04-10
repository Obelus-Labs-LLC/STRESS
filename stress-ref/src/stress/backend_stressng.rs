use std::process::{Child, Command, Stdio};
use crate::stress::backend::*;

pub struct StressNgBackend {
    linux: crate::stress::backend_linux::LinuxBackend,
    memory_proc: Option<Child>,
    cpu_proc: Option<Child>,
}

impl StressNgBackend {
    pub fn new() -> Self {
        Self {
            linux: crate::stress::backend_linux::LinuxBackend,
            memory_proc: None,
            cpu_proc: None,
        }
    }

    pub fn inject_cpu_stress(&mut self, cpu_load: u32) -> Result<(), String> {
        self.remove_cpu_stress();
        let child = Command::new("stress-ng")
            .args(["--cpu", "0", "--cpu-load", &cpu_load.to_string()])
            .stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().map_err(|e| format!("stress-ng spawn failed: {e}"))?;
        self.cpu_proc = Some(child);
        Ok(())
    }

    pub fn remove_cpu_stress(&mut self) {
        if let Some(mut p) = self.cpu_proc.take() {
            let _ = p.kill();
            let _ = p.wait();
        }
    }
}

impl StressBackend for StressNgBackend {
    fn inject_memory_stress(&mut self, config: &MemoryStressConfig) -> Result<(), String> {
        self.remove_memory_stress()?;
        let child = Command::new("stress-ng")
            .args([
                "--vm", "1",
                "--vm-bytes", &config.vm_bytes.to_string(),
                "--vm-method", &config.vm_method,
                "--vm-keep",
            ])
            .stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().map_err(|e| format!("stress-ng spawn failed: {e}"))?;
        self.memory_proc = Some(child);
        Ok(())
    }

    fn remove_memory_stress(&mut self) -> Result<(), String> {
        if let Some(mut p) = self.memory_proc.take() {
            let _ = p.kill();
            let _ = p.wait();
        }
        Ok(())
    }

    fn pause_workload(&mut self, pid: u32) -> Result<(), String> {
        self.linux.pause_workload(pid)
    }

    fn resume_workload(&mut self, pid: u32) -> Result<(), String> {
        self.linux.resume_workload(pid)
    }

    fn apply_network_degradation(&mut self, config: &NetworkDegradationConfig) -> Result<(), String> {
        self.linux.apply_network_degradation(config)
    }

    fn remove_network_degradation(&mut self, interface: &str) -> Result<(), String> {
        self.linux.remove_network_degradation(interface)
    }

    fn apply_resource_pressure(&mut self, config: &ResourcePressureConfig) -> Result<(), String> {
        self.linux.apply_resource_pressure(config)
    }

    fn remove_resource_pressure(&mut self, cgroup_path: &str) -> Result<(), String> {
        self.linux.remove_resource_pressure(cgroup_path)
    }

    fn apply_network_partition(&mut self, interface: &str) -> Result<(), String> {
        self.linux.apply_network_partition(interface)
    }

    fn remove_network_partition(&mut self, interface: &str) -> Result<(), String> {
        self.linux.remove_network_partition(interface)
    }

    fn name(&self) -> &str { "stress-ng" }
}

impl Drop for StressNgBackend {
    fn drop(&mut self) {
        let _ = self.remove_memory_stress();
        self.remove_cpu_stress();
    }
}

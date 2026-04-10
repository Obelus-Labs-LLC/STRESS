use super::backend::*;

/// Linux backend using tc/netem for network degradation and cgroups v2 for resource pressure.
pub struct LinuxBackend;

impl LinuxBackend {
    pub fn new() -> Self { Self }
}

impl StressBackend for LinuxBackend {
    fn apply_network_degradation(&mut self, config: &NetworkDegradationConfig) -> Result<(), String> {
        // Remove any existing qdisc first
        let _ = std::process::Command::new("tc")
            .args(["qdisc", "del", "dev", &config.interface, "root"])
            .output();

        // Add netem qdisc
        let output = std::process::Command::new("tc")
            .args([
                "qdisc", "add", "dev", &config.interface, "root", "netem",
                "delay", &format!("{}ms", config.latency_ms),
                &format!("{}ms", config.jitter_ms),
                "loss", &format!("{}%", config.loss_percent),
            ])
            .output()
            .map_err(|e| format!("tc command failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tc qdisc add failed: {}", stderr));
        }
        Ok(())
    }

    fn remove_network_degradation(&mut self, interface: &str) -> Result<(), String> {
        let output = std::process::Command::new("tc")
            .args(["qdisc", "del", "dev", interface, "root"])
            .output()
            .map_err(|e| format!("tc command failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore "RTNETLINK answers: No such file or directory" (no qdisc to remove)
            if !stderr.contains("No such file") {
                return Err(format!("tc qdisc del failed: {}", stderr));
            }
        }
        Ok(())
    }

    fn apply_resource_pressure(&mut self, config: &ResourcePressureConfig) -> Result<(), String> {
        if let Some(mem_limit) = config.memory_limit_bytes {
            let path = format!("{}/memory.max", config.cgroup_path);
            std::fs::write(&path, mem_limit.to_string())
                .map_err(|e| format!("failed to write {}: {}", path, e))?;
        }

        if let Some(quota) = config.cpu_quota_us {
            let period = config.cpu_period_us.unwrap_or(100_000);
            let path = format!("{}/cpu.max", config.cgroup_path);
            std::fs::write(&path, format!("{} {}", quota, period))
                .map_err(|e| format!("failed to write {}: {}", path, e))?;
        }
        Ok(())
    }

    fn remove_resource_pressure(&mut self, cgroup_path: &str) -> Result<(), String> {
        // Remove limits by writing "max"
        let mem_path = format!("{}/memory.max", cgroup_path);
        let _ = std::fs::write(&mem_path, "max");

        let cpu_path = format!("{}/cpu.max", cgroup_path);
        let _ = std::fs::write(&cpu_path, "max 100000");

        Ok(())
    }

    fn inject_memory_stress(&mut self, _config: &MemoryStressConfig) -> Result<(), String> {
        Err("Use StressNgBackend for memory stress".into())
    }

    fn remove_memory_stress(&mut self) -> Result<(), String> {
        Err("Use StressNgBackend for memory stress".into())
    }

    fn pause_workload(&mut self, pid: u32) -> Result<(), String> {
        let status = std::process::Command::new("kill")
            .args(["-STOP", &pid.to_string()])
            .status()
            .map_err(|e| format!("kill -STOP failed: {}", e))?;
        if !status.success() {
            return Err(format!("kill -STOP {} exited with {}", pid, status));
        }
        Ok(())
    }

    fn resume_workload(&mut self, pid: u32) -> Result<(), String> {
        let status = std::process::Command::new("kill")
            .args(["-CONT", &pid.to_string()])
            .status()
            .map_err(|e| format!("kill -CONT failed: {}", e))?;
        if !status.success() {
            return Err(format!("kill -CONT {} exited with {}", pid, status));
        }
        Ok(())
    }

    fn apply_network_partition(&mut self, interface: &str) -> Result<(), String> {
        let status = std::process::Command::new("iptables")
            .args(["-A", "OUTPUT", "-o", interface, "-j", "DROP"])
            .status()
            .map_err(|e| format!("iptables failed: {}", e))?;
        if !status.success() {
            return Err(format!("iptables -A OUTPUT failed with {}", status));
        }
        Ok(())
    }

    fn remove_network_partition(&mut self, interface: &str) -> Result<(), String> {
        let _ = std::process::Command::new("iptables")
            .args(["-D", "OUTPUT", "-o", interface, "-j", "DROP"])
            .status();
        Ok(())
    }

    fn name(&self) -> &str { "linux-tc-cgroups" }
}

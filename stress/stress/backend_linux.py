"""Linux backend using tc/netem and cgroups v2."""
import os
import signal
import subprocess
from .backend import (
    StressBackend,
    NetworkDegradationConfig,
    ResourcePressureConfig,
    MemoryStressConfig,
)


class LinuxBackend(StressBackend):
    def apply_network_degradation(self, config: NetworkDegradationConfig) -> None:
        # Remove existing qdisc
        subprocess.run(
            ["tc", "qdisc", "del", "dev", config.interface, "root"],
            capture_output=True,
        )
        # Add netem
        subprocess.run(
            [
                "tc", "qdisc", "add", "dev", config.interface, "root", "netem",
                "delay", f"{config.latency_ms}ms", f"{config.jitter_ms}ms",
                "loss", f"{config.loss_percent}%",
            ],
            check=True,
            capture_output=True,
        )

    def remove_network_degradation(self, interface: str) -> None:
        subprocess.run(
            ["tc", "qdisc", "del", "dev", interface, "root"],
            capture_output=True,
        )

    def apply_resource_pressure(self, config: ResourcePressureConfig) -> None:
        if config.memory_limit_bytes is not None:
            path = f"{config.cgroup_path}/memory.max"
            with open(path, "w") as f:
                f.write(str(config.memory_limit_bytes))
        if config.cpu_quota_us is not None:
            period = config.cpu_period_us or 100_000
            path = f"{config.cgroup_path}/cpu.max"
            with open(path, "w") as f:
                f.write(f"{config.cpu_quota_us} {period}")

    def remove_resource_pressure(self, cgroup_path: str) -> None:
        for sub in ("memory.max", "cpu.max"):
            path = f"{cgroup_path}/{sub}"
            try:
                with open(path, "w") as f:
                    f.write("max" if "memory" in sub else "max 100000")
            except OSError:
                pass

    def name(self) -> str:
        return "linux-tc-cgroups"

    def inject_memory_stress(self, config: MemoryStressConfig) -> None:
        raise NotImplementedError("Use StressNgBackend for memory stress injection")

    def remove_memory_stress(self) -> None:
        raise NotImplementedError("Use StressNgBackend for memory stress injection")

    def pause_workload(self, pid: int) -> None:
        os.kill(pid, signal.SIGSTOP)

    def resume_workload(self, pid: int) -> None:
        os.kill(pid, signal.SIGCONT)

    def apply_network_partition(self, interface: str) -> None:
        subprocess.run(
            ["iptables", "-A", "OUTPUT", "-o", interface, "-j", "DROP"],
            check=True,
        )

    def remove_network_partition(self, interface: str) -> None:
        subprocess.run(
            ["iptables", "-D", "OUTPUT", "-o", interface, "-j", "DROP"],
            check=False,
        )

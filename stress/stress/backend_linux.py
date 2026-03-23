"""Linux backend using tc/netem and cgroups v2."""
import subprocess
from .backend import (
    StressBackend,
    NetworkDegradationConfig,
    ResourcePressureConfig,
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

"""StressBackend implementation using stress-ng for real SP-1/SP-2 injection."""
from __future__ import annotations

import subprocess
from typing import Optional

from stress.stress.backend import (
    StressBackend, NetworkDegradationConfig, ResourcePressureConfig, MemoryStressConfig,
)
from stress.stress.backend_linux import LinuxBackend


class StressNgBackend(StressBackend):
    """Real stress injection via stress-ng subprocess.

    Covers SP-1 (radiation/bit-flip via --vm-method flip) and
    SP-2 (thermal cycling via --cpu-load modulation).
    Delegates network, power, and isolation to LinuxBackend.
    """

    def __init__(self) -> None:
        self._linux = LinuxBackend()
        self._memory_proc: Optional[subprocess.Popen] = None
        self._cpu_proc: Optional[subprocess.Popen] = None

    def inject_memory_stress(self, config: MemoryStressConfig) -> None:
        self.remove_memory_stress()
        self._memory_proc = subprocess.Popen([
            "stress-ng",
            "--vm", "1",
            "--vm-bytes", str(config.vm_bytes),
            "--vm-method", config.vm_method,
            "--vm-keep",
        ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def remove_memory_stress(self) -> None:
        if self._memory_proc is not None:
            self._memory_proc.terminate()
            self._memory_proc.wait()
            self._memory_proc = None

    def inject_cpu_stress(self, cpu_load: int) -> None:
        """SP-2 thermal cycling: set CPU load percentage via stress-ng."""
        self.remove_cpu_stress()
        self._cpu_proc = subprocess.Popen([
            "stress-ng",
            "--cpu", "0",
            "--cpu-load", str(cpu_load),
        ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def remove_cpu_stress(self) -> None:
        if self._cpu_proc is not None:
            self._cpu_proc.terminate()
            self._cpu_proc.wait()
            self._cpu_proc = None

    def apply_network_degradation(self, config: NetworkDegradationConfig) -> None:
        self._linux.apply_network_degradation(config)

    def remove_network_degradation(self, interface: str) -> None:
        self._linux.remove_network_degradation(interface)

    def apply_resource_pressure(self, config: ResourcePressureConfig) -> None:
        self._linux.apply_resource_pressure(config)

    def remove_resource_pressure(self, cgroup_path: str) -> None:
        self._linux.remove_resource_pressure(cgroup_path)

    def pause_workload(self, pid: int) -> None:
        self._linux.pause_workload(pid)

    def resume_workload(self, pid: int) -> None:
        self._linux.resume_workload(pid)

    def apply_network_partition(self, interface: str) -> None:
        self._linux.apply_network_partition(interface)

    def remove_network_partition(self, interface: str) -> None:
        self._linux.remove_network_partition(interface)

    def name(self) -> str:
        return "stress-ng"

"""Stress backend abstraction — simulated vs real injection."""
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class NetworkDegradationConfig:
    interface: str
    latency_ms: float
    jitter_ms: float
    loss_percent: float


@dataclass(frozen=True)
class ResourcePressureConfig:
    cgroup_path: str
    memory_limit_bytes: Optional[int] = None
    cpu_quota_us: Optional[int] = None
    cpu_period_us: Optional[int] = None


class StressBackend(ABC):
    @abstractmethod
    def apply_network_degradation(self, config: NetworkDegradationConfig) -> None: ...

    @abstractmethod
    def remove_network_degradation(self, interface: str) -> None: ...

    @abstractmethod
    def apply_resource_pressure(self, config: ResourcePressureConfig) -> None: ...

    @abstractmethod
    def remove_resource_pressure(self, cgroup_path: str) -> None: ...

    @abstractmethod
    def name(self) -> str: ...


class SimulatedBackend(StressBackend):
    """No-ops. All stress handled by Poisson/sinusoidal models."""

    def apply_network_degradation(self, config: NetworkDegradationConfig) -> None:
        pass

    def remove_network_degradation(self, interface: str) -> None:
        pass

    def apply_resource_pressure(self, config: ResourcePressureConfig) -> None:
        pass

    def remove_resource_pressure(self, cgroup_path: str) -> None:
        pass

    def name(self) -> str:
        return "simulated"

"""StressBackend implementation using Toxiproxy for real SP-4 injection."""
from __future__ import annotations

import json
import urllib.request
import urllib.error
from typing import Optional

from stress.stress.backend import (
    StressBackend, NetworkDegradationConfig, ResourcePressureConfig, MemoryStressConfig,
)
from stress.stress.backend_linux import LinuxBackend


class ToxiproxyBackend(StressBackend):
    """Real network fault injection via Toxiproxy REST API.

    Covers SP-4 (network jitter, latency, packet loss) by driving
    Toxiproxy toxics. Delegates other operations to LinuxBackend.

    Requires a running Toxiproxy server and a pre-configured proxy.
    """

    def __init__(
        self, api_url: str = "http://localhost:8474", proxy_name: str = "stress-proxy",
    ) -> None:
        self._api_url = api_url.rstrip("/")
        self._proxy_name = proxy_name
        self._linux = LinuxBackend()
        self._toxic_names: list[str] = []

    def _api(self, method: str, path: str, body: Optional[dict] = None) -> dict:
        url = f"{self._api_url}{path}"
        data = json.dumps(body).encode() if body else None
        req = urllib.request.Request(url, data=data, method=method)
        req.add_header("Content-Type", "application/json")
        try:
            with urllib.request.urlopen(req) as resp:
                return json.loads(resp.read().decode()) if resp.status == 200 else {}
        except urllib.error.HTTPError:
            return {}

    def apply_network_degradation(self, config: NetworkDegradationConfig) -> None:
        self.remove_network_degradation(config.interface)
        # Add latency toxic
        result = self._api("POST", f"/proxies/{self._proxy_name}/toxics", {
            "name": "stress_latency",
            "type": "latency",
            "stream": "downstream",
            "toxicity": 1.0,
            "attributes": {
                "latency": int(config.latency_ms),
                "jitter": int(config.jitter_ms),
            },
        })
        if result:
            self._toxic_names.append("stress_latency")

        # Simulate packet loss via bandwidth limiting + slicer
        if config.loss_percent > 0:
            result = self._api("POST", f"/proxies/{self._proxy_name}/toxics", {
                "name": "stress_timeout",
                "type": "timeout",
                "stream": "downstream",
                "toxicity": config.loss_percent / 100.0,
                "attributes": {"timeout": 1},
            })
            if result:
                self._toxic_names.append("stress_timeout")

    def remove_network_degradation(self, interface: str) -> None:
        for name in self._toxic_names:
            self._api("DELETE", f"/proxies/{self._proxy_name}/toxics/{name}")
        self._toxic_names.clear()

    def apply_resource_pressure(self, config: ResourcePressureConfig) -> None:
        self._linux.apply_resource_pressure(config)

    def remove_resource_pressure(self, cgroup_path: str) -> None:
        self._linux.remove_resource_pressure(cgroup_path)

    def inject_memory_stress(self, config: MemoryStressConfig) -> None:
        raise NotImplementedError("ToxiproxyBackend only handles network faults. Use StressNgBackend for memory stress.")

    def remove_memory_stress(self) -> None:
        pass

    def pause_workload(self, pid: int) -> None:
        self._linux.pause_workload(pid)

    def resume_workload(self, pid: int) -> None:
        self._linux.resume_workload(pid)

    def apply_network_partition(self, interface: str) -> None:
        # Use Toxiproxy: disable the proxy entirely
        self._api("POST", f"/proxies/{self._proxy_name}", {"enabled": False})

    def remove_network_partition(self, interface: str) -> None:
        self._api("POST", f"/proxies/{self._proxy_name}", {"enabled": True})

    def name(self) -> str:
        return "toxiproxy"

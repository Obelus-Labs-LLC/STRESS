# STRESS Python Reference Implementation

Python reference implementation of the System Threat Resilience & Extreme Stress Suite (STRESS) v0.2 specification.

The Rust implementation in `stress-ref/` is the canonical reference. This Python implementation provides an accessible alternative.

## Structure

| Module | Description |
|--------|-------------|
| `metrics/` | Behavioral proxy computations (GDS, ARR, IST, REC, CFR, SRI) |
| `measure/` | Event model and event log |
| `stress/` | Stress injectors (radiation, thermal, power, network, isolation) |
| `workloads/` | Reference workloads (W1-A, W2-A, W3-A) |
| `stats/` | Statistical aggregation (mean, std, CI) |
| `report/` | JSON report generation |
| `runner.py` | Benchmark orchestrator |
| `config.py` | Configuration and seed management |

## Scope
- Implements STRESS v0.2 as specified
- Prioritizes correctness, transparency, and reproducibility
- Does not optimize for performance or deployment

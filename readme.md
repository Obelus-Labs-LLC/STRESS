# STRESS — System Threat Resilience & Extreme Stress Suite (v0.2)

STRESS is a reliability benchmarking framework designed to evaluate how computational workloads behave when foundational operating assumptions are violated by environmental and systemic constraints. Unlike terrestrial benchmarks — which typically assume continuous power, stable connectivity, and rare environmental disruption — STRESS focuses on resilience and behavioral stability under persistent stress, rather than performance optimization, throughput, or cost efficiency.

## Status
- Specification: Frozen (v0.2)
- Reference Implementation: Rust (`stress-ref/`) + Python (`stress/`)
- Compliance: Binary

## Quick Start

```bash
pip install -e .
stress-benchmark --workload W1-A --profile SP-1 --seed 42 --runs 3 --out-dir ./results
```

Or from Python:

```python
from stress.runner import run_benchmark

run_benchmark(
    out_dir="./results", workload_id="W1-A", workload_version="0.2",
    stress_profile_id="SP-1", stress_parameters={"SP-1": {"rate": 0.001}},
    execution_environment={"runtime": "python"}, master_seed=42, n_runs=3,
    gds_levels=[0.1, 0.2, 0.3],
)
```

## Full Specification

- **[STRESS v0.2 Specification](./docs/specification-v0.2.md)** — Complete technical specification
- **[Implementation Guide](./STRESS_v0_Implementation_Guide.md)** — Implementation details
- **[Complete Specification](./docs/STRESS_v0_Complete_Specification.md)** — Extended technical document
- **[Reference Workloads](./docs/STRESS_v0_Reference_Workloads.md)** — W1-A, W2-A, W3-A definitions
- **[Stress Profiles](./docs/STRESS_v0_Reference_Stress_Profiles.md)** — SP-0 through SP-5 definitions
- **[Glossary](./docs/glossary.md)** — Term definitions
- **[Metric Independence Analysis](./docs/metric-independence.md)** — GDS/ARR independence proof
- **[Validation Methodology](./docs/validation-methodology.md)** — SRI correlation validation protocol

## Repository Structure

| Path | Description |
|------|-------------|
| `docs/specification-v0.2.md` | Normative specification |
| `docs/STRESS_v0_Reference_Workloads.md` | W1-A, W2-A, W3-A workload definitions |
| `docs/STRESS_v0_Reference_Stress_Profiles.md` | SP-0 through SP-5 profiles |
| `docs/metric-independence.md` | GDS/ARR independence proof |
| `docs/validation-methodology.md` | SRI correlation validation protocol |
| `docs/glossary.md` | Term definitions |
| `docs/historical/` | Archived OCRB v0.1 specification |
| `stress-ref/` | **Rust reference implementation** (canonical, 34 tests) |
| `stress/` | Python reference implementation |
| `tests/` | Python test suite |
| `examples/hello_benchmark.py` | Minimal "hello benchmark" walkthrough |
| `examples/run_w1a_sp1.py` | W1-A with SP-1 radiation profile |
| `examples/run_w3a_sp1.py` | W3-A distributed workload example |
| `schema/run-record.schema.json` | Per-run JSON report schema |
| `schema/aggregate.schema.json` | Aggregate statistics schema |
| `schema/manifest.schema.json` | Benchmark manifest schema |
| `STRESS_v0_Implementation_Guide.md` | Implementation reference |
| `STRESS_FRAMEWORK_RESEARCH.md` | Background research and design rationale |
| `CONTRIBUTING.md` | Contribution guidelines |

## What This Repo Is NOT
- Not a performance benchmark
- Not an optimization framework
- Not adaptive or learning-based

## Running a Benchmark

### Rust (recommended)

```bash
cd stress-ref
cargo run -- --workload W1-A --profile SP-1 --seed 42 --runs 10 \
  --gds-levels 0.1,0.2,0.3 --isolation-duration 60.0 --c-total 5
```

### Python CLI

```bash
pip install -e .
stress-benchmark --workload W1-A --profile SP-1 --seed 42 --runs 10 --out-dir report
```

### SRI Aggregation

SRI uses the **geometric mean** of five behavioral proxies (GDS, ARR, IST, REC, CFR), each normalized to [0,1]:

```
SRI = (GDS × ARR × IST × REC × CFR)^(1/5) × 100
```

Geometric mean ensures zero resilience in any single dimension drives SRI toward 0 — a system cannot hide catastrophic failure behind strong scores elsewhere. GDS also reports **monotonicity** and **smoothness** metadata to distinguish graceful degradation from cliff-drops.

### Domain-Specific SRI

STRESS supports domain-specific weighting profiles for SRI computation (weighted geometric mean):

| Profile | IST Weight | CFR Weight | Use Case |
|---------|-----------|-----------|----------|
| Equal (default) | 0.20 | 0.20 | General-purpose comparison |
| Satellite/LEO | 0.35 | 0.10 | Isolation survival matters most |
| Data Center | 0.10 | 0.35 | Cascade containment matters most |
| Tactical Edge | 0.20 | 0.10 | Recovery and degradation balanced |

## Version History

| Version | Date | Description |
|---------|------|-------------|
| v0.2 | 2026-03 | STRESS — Current specification with SRI [0,100] scale |
| v0.1 | (Archived) | OCRB — Original specification with ORI [0,1] scale |

## Migration from OCRB v0.1

STRESS v0.2 supersedes OCRB (Orbital Compute Readiness Benchmark) v0.1.

| OCRB v0.1 | STRESS v0.2 | Conversion |
|-----------|-------------|------------|
| ORI [0, 1] | SRI [0, 100] | `SRI = ORI * 100` |
| 0.85 threshold | 85 threshold | Direct mapping |
| Stress Regimes | Stress Profiles | Renamed |
| SR-1 to SR-5 | SP-1 to SP-5 | Renamed |

### Archives

- [OCRB v0.1 Specification (Deprecated)](./docs/historical/ocrb-v0.1-deprecated.md) — Original specification preserved for reference

---

*Maintained by Obelus Labs, LLC*

---

If this framework helped your research, consider giving it a star — it helps others discover it.

# STRESS — System Threat Resilience & Extreme Stress Suite (v0.2)

![Tests](https://github.com/Obelus-Labs-LLC/STRESS/actions/workflows/test.yml/badge.svg)
![SRI](https://img.shields.io/badge/SRI-dynamic-blue)

STRESS is a reliability benchmarking framework designed to evaluate how computational workloads behave when foundational operating assumptions are violated by environmental and systemic constraints. Unlike terrestrial benchmarks — which typically assume continuous power, stable connectivity, and rare environmental disruption — STRESS focuses on resilience and behavioral stability under persistent stress, rather than performance optimization, throughput, or cost efficiency.

## Status
- Specification: Frozen (v0.2)
- Reference Implementation: Rust (`stress-ref/`, 40 tests) + Python (`stress/`, 33 tests)
- Compliance: Binary
- CI: Automated tests, benchmark tracking, schema validation

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

### Rust (recommended)

```bash
cd stress-ref
cargo run -- --workload W1-A --profile SP-1 --seed 42 --runs 10 \
  --gds-levels 0.1,0.2,0.3 --isolation-duration 60.0 --c-total 5
```

-----

## Behavioral Proxies

STRESS evaluates five behavioral proxies, each normalized to [0,1]:

| Proxy | What It Measures |
|-------|-----------------|
| **GDS** (Graceful Degradation Score) | Average task completion rate across increasing stress levels. Reports `monotonicity` and `smoothness` metadata to distinguish graceful degradation from cliff-drops. |
| **ARR** (Autonomous Recovery Rate) | Fraction of recoverable faults resolved without external intervention. |
| **IST** (Isolation Survival Time) | Normalized survival time under complete communication isolation. |
| **REC** (Resource Efficiency Under Constraint) | Efficiency ratio under stress relative to baseline operation. |
| **CFR** (Cascading Failure Resistance) | Degree to which localized failures remain contained. |

## Stress Resilience Index (SRI)

SRI uses the **geometric mean** of all five proxies:

```
SRI = (GDS x ARR x IST x REC x CFR)^(1/5) x 100
```

Geometric mean ensures zero resilience in any single dimension drives SRI toward 0 — a system cannot hide catastrophic failure behind strong scores elsewhere.

### Domain-Specific SRI

Weighted geometric mean profiles for domain-specific evaluation:

| Profile | GDS | ARR | IST | REC | CFR | Use Case |
|---------|-----|-----|-----|-----|-----|----------|
| Equal (default) | 0.20 | 0.20 | 0.20 | 0.20 | 0.20 | General-purpose comparison |
| Satellite/LEO | 0.20 | 0.20 | 0.35 | 0.15 | 0.10 | Isolation survival matters most |
| Data Center | 0.20 | 0.20 | 0.10 | 0.15 | 0.35 | Cascade containment matters most |
| Tactical Edge | 0.25 | 0.25 | 0.20 | 0.20 | 0.10 | Recovery and degradation balanced |

### Statistical Reporting

- 95% confidence intervals using **t-distribution** for n < 30 (not normal approximation)
- **Cohen's d** effect size for comparing SRI between systems
- **MAD outlier detection** flags anomalous benchmark runs
- **Bootstrap CIs** (BCa method) when scipy is installed — more accurate for geometric mean of bounded values

-----

## Stress Parameters

| Parameter | What It Simulates | Real Backend |
|-----------|-------------------|--------------|
| **SP-1**: Radiation Pressure | Memory corruption, transient errors | `stress-ng --vm-method flip` |
| **SP-2**: Thermal Cycling | Periodic environmental stress | `stress-ng --cpu-load` (modulated) |
| **SP-3**: Power Disruption | Intermittent availability | `SIGSTOP/SIGCONT` (Linux) |
| **SP-4**: Network Jitter | Latency variability, packet loss | [Toxiproxy](https://github.com/Shopify/toxiproxy) |
| **SP-5**: Isolation Duration | Complete communication cutoff | `iptables DROP` (Linux) |

The default **simulated backend** uses seeded PRNGs for reproducibility. Real backends use external tools for actual fault injection.

-----

## Reports

### JSON (default)
Always generated: `manifest.json`, `runs/run_*.json`, `aggregate_summary.json`, `disclosure.md`.

### HTML Report
Requires `pip install -e ".[report]"` (Jinja2). Generates `report.html` with:
- SRI score gauge (color-coded)
- Chart.js radar chart (5 proxy axes)
- GDS degradation curve (completion rate vs stress level)
- Aggregate statistics table
- Per-run collapsible details
- Disclosure text

### PDF Export
Requires `pip install -e ".[pdf]"` (WeasyPrint). Converts HTML report to PDF.

### SRI Badges
- **Shields.io**: `generate_badge_url(sri)` returns a URL like `![SRI](https://img.shields.io/badge/SRI-87.3-brightgreen)`
- **Local SVG**: `generate_badge_svg(sri, path)` with optional pybadges (`pip install -e ".[badges]"`)

### JSON Schema Generation (Rust)
```bash
cd stress-ref && cargo run --bin stress-schema
```
Outputs JSON Schema derived from Rust report types via `schemars`.

-----

## Repository Structure

### Specification

| Path | Description |
|------|-------------|
| `docs/specification-v0.2.md` | Normative specification |
| `docs/STRESS_v0_Complete_Specification.md` | Extended technical specification |
| `docs/STRESS_v0_Reference_Workloads.md` | W1-A, W2-A, W3-A workload definitions |
| `docs/STRESS_v0_Reference_Stress_Profiles.md` | SP-0 through SP-5 profiles |
| `docs/metric-independence.md` | GDS/ARR independence proof |
| `docs/validation-methodology.md` | SRI correlation validation protocol |
| `docs/glossary.md` | Term definitions |
| `docs/historical/` | Archived OCRB v0.1 specification |

### Rust Reference Implementation (`stress-ref/`)

| Path | Description |
|------|-------------|
| `src/metrics/` | GDS, ARR, IST, REC, CFR, SRI computation (geometric mean) |
| `src/stats/` | Aggregation, t-distribution CI, Cohen's d, MAD outlier detection |
| `src/stress/` | Stress injection layer (simulated, Linux, stress-ng, Toxiproxy backends) |
| `src/workloads/` | W1-A stateless, W2-A stateful pipeline, W3-A distributed coordination |
| `src/types/` | Report types with `schemars::JsonSchema` derivation |
| `src/report/` | JSON report writing |
| `src/runner.rs` | Benchmark orchestration |
| `src/bin/stress.rs` | CLI entrypoint |
| `src/bin/schema_gen.rs` | JSON Schema generation binary |

### Python Implementation (`stress/`)

| Path | Description |
|------|-------------|
| `stress/metrics/` | GDS (with smoothness/monotonicity), ARR, IST, REC, CFR, SRI |
| `stress/stats/` | Aggregation, t-distribution CI, Cohen's d, MAD, bootstrap CI |
| `stress/stress/` | Stress backends (simulated, Linux, stress-ng, Toxiproxy) |
| `stress/workloads/` | W1-A, W2-A, W3-A workload implementations |
| `stress/report/` | JSON writer, HTML report (Chart.js), PDF export, badge generation |
| `stress/runner.py` | Benchmark orchestration |
| `stress/cli.py` | CLI entrypoint |

### Schemas & CI

| Path | Description |
|------|-------------|
| `schema/run-record.schema.json` | Per-run JSON report schema |
| `schema/aggregate.schema.json` | Aggregate statistics schema |
| `schema/manifest.schema.json` | Benchmark manifest schema |
| `.github/workflows/test.yml` | Python (3.9/3.12) + Rust tests on push/PR |
| `.github/workflows/benchmark.yml` | SRI tracking + threshold gate + GitHub Pages |
| `.github/workflows/schema-validate.yml` | JSON Schema validation |

### Other

| Path | Description |
|------|-------------|
| `examples/` | Hello benchmark, W1-A + SP-1, W3-A + SP-1 examples |
| `tests/` | Python test suite (33 tests) |
| `STRESS_v0_Implementation_Guide.md` | Implementation reference |
| `STRESS_FRAMEWORK_RESEARCH.md` | Background research and design rationale |
| `CONTRIBUTING.md` | Contribution guidelines |

-----

## Optional Dependencies

| Group | Install | Provides |
|-------|---------|----------|
| `test` | `pip install -e ".[test]"` | pytest |
| `stats` | `pip install -e ".[stats]"` | Bootstrap CIs, distribution fitting (scipy) |
| `report` | `pip install -e ".[report]"` | HTML reports with Chart.js radar charts (Jinja2) |
| `pdf` | `pip install -e ".[pdf]"` | PDF export (WeasyPrint) |
| `badges` | `pip install -e ".[badges]"` | Local SVG badge generation (pybadges) |
| `all` | `pip install -e ".[all]"` | Everything above |

Core `dependencies = []` remains empty by design.

-----

## CI Integration

| Workflow | Trigger | What It Does |
|----------|---------|-------------|
| `test.yml` | Push/PR | Python pytest (3.9 + 3.12 matrix) + Rust cargo test |
| `benchmark.yml` | Push to main | Run benchmark, check SRI threshold, publish trends to GitHub Pages |
| `schema-validate.yml` | Push/PR to schema/ or types/ | Validate JSON schemas are well-formed |

-----

## What This Repo Is NOT
- Not a performance benchmark
- Not an optimization framework
- Not adaptive or learning-based

-----

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

-----

*Maintained by Obelus Labs, LLC*

---

If this framework helped your research, consider giving it a star — it helps others discover it.

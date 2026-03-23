# STRESS Tests

Tests validating the Python reference implementation for correctness, reproducibility, and specification adherence.

## Structure

| Path | Description |
|------|-------------|
| `test_smoke.py` | End-to-end smoke test for benchmark execution |
| `test_metrics/` | Unit tests for individual metric computations (SRI, GDS, ARR) |
| `test_stress/` | Unit tests for stress injectors (radiation, thermal, isolation, regime) |

## Running

```bash
pytest tests/
```

## Scope

These tests validate compliance with STRESS specifications.
They are not performance benchmarks and do not test
application-level correctness beyond defined STRESS metrics.

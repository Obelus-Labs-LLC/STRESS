"""Minimal STRESS benchmark example — 5 lines to get started."""
from stress.runner import run_benchmark

run_benchmark(
    out_dir="./hello_results",
    workload_id="W1-A",
    workload_version="0.2",
    stress_profile_id="SP-1",
    stress_parameters={"SP-1": {"rate": 0.001}},
    execution_environment={"runtime": "python"},
    master_seed=42,
    n_runs=3,
    gds_levels=[0.1, 0.2, 0.3],
)
print("Done! Check ./hello_results/ for reports.")

import argparse
import sys


def main():
    parser = argparse.ArgumentParser(
        prog="stress-benchmark",
        description="STRESS v0.2 — System Threat Resilience & Extreme Stress Suite",
    )
    parser.add_argument("--workload", required=True, choices=["W1-A", "W2-A", "W3-A"])
    parser.add_argument("--profile", default="SP-1")
    parser.add_argument("--seed", type=int, required=True)
    parser.add_argument("--runs", type=int, default=10)
    parser.add_argument("--out-dir", default="report")
    parser.add_argument("--gds-levels", default="0.1,0.2,0.3")
    parser.add_argument("--isolation-duration", type=float, default=None)
    parser.add_argument("--c-total", type=int, default=None)

    args = parser.parse_args()

    from stress.runner import run_benchmark
    from stress.stress.profiles import get_profile

    gds_levels = [float(x) for x in args.gds_levels.split(",")]
    params = get_profile(args.profile)

    run_benchmark(
        out_dir=args.out_dir,
        workload_id=args.workload,
        workload_version="0.2",
        stress_profile_id=args.profile,
        stress_parameters=params,
        execution_environment={"runtime": "python", "version": sys.version},
        master_seed=args.seed,
        n_runs=args.runs,
        gds_levels=gds_levels,
        isolation_duration_declared=args.isolation_duration,
        C_total=args.c_total,
    )
    print(f"Benchmark complete. Results in {args.out_dir}/")


if __name__ == "__main__":
    main()

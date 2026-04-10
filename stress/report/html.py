"""HTML report generation with Chart.js radar charts."""
from __future__ import annotations

import json
from pathlib import Path
from typing import Optional


def generate_html_report(out_dir: str) -> Optional[Path]:
    """Generate an HTML report from STRESS JSON output.

    Requires jinja2. Returns path to generated report or None if jinja2 is unavailable.
    """
    try:
        from jinja2 import Environment, FileSystemLoader, select_autoescape
    except ImportError:
        return None

    out = Path(out_dir)
    agg_path = out / "aggregate_summary.json"
    if not agg_path.exists():
        return None

    with open(agg_path) as f:
        aggregate = json.load(f)

    # Collect per-run data
    runs_dir = out / "runs"
    runs = []
    if runs_dir.exists():
        for p in sorted(runs_dir.glob("run_*.json")):
            with open(p) as f:
                runs.append(json.load(f))

    # Read disclosure
    disclosure = ""
    disc_path = out / "disclosure.md"
    if disc_path.exists():
        disclosure = disc_path.read_text()

    # Load template
    template_dir = Path(__file__).parent / "templates"
    env = Environment(
        loader=FileSystemLoader(str(template_dir)),
        autoescape=select_autoescape(["html"]),
    )
    template = env.get_template("report.html")

    # Build template context
    proxy_names = ["gds", "arr", "ist", "rec", "cfr"]
    proxy_means = [aggregate.get(p, {}).get("mean", 0) or 0 for p in proxy_names]
    sri_mean = aggregate.get("sri", {}).get("mean")

    # GDS degradation curve from first run's evidence
    gds_levels = []
    gds_rates = []
    if runs:
        evidence = runs[0].get("evidence", {})
        gds_levels = evidence.get("stress_levels", [])
        gds_rates = evidence.get("completion_rates", [])

    html = template.render(
        sri_mean=sri_mean,
        proxy_names=[p.upper() for p in proxy_names],
        proxy_means=proxy_means,
        aggregate=aggregate,
        runs=runs,
        gds_levels=gds_levels,
        gds_rates=gds_rates,
        disclosure=disclosure,
    )

    report_path = out / "report.html"
    report_path.write_text(html, encoding="utf-8")
    return report_path

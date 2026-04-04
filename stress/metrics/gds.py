from __future__ import annotations

import math
from dataclasses import dataclass
from typing import List, Optional

from stress.measure.events import Event, EventType


@dataclass(frozen=True)
class GDSResult:
    gds: Optional[float]               # None = N/A
    n_levels: int
    stress_levels: List[float]
    completion_rates: List[float]
    monotonicity: Optional[float] = None  # Fraction of pairs with C_{i+1} <= C_i
    smoothness: Optional[float] = None    # Entropy-based step uniformity [0,1]
    na_reason: Optional[str] = None


def _compute_monotonicity(rates: List[float]) -> Optional[float]:
    """Fraction of adjacent pairs where C_{i+1} <= C_i. Range [0,1]."""
    if len(rates) < 2:
        return None
    pairs = len(rates) - 1
    mono = sum(1 for i in range(pairs) if rates[i + 1] <= rates[i])
    return mono / pairs


def _compute_smoothness(rates: List[float]) -> Optional[float]:
    """Entropy-based uniformity of degradation steps. Range [0,1].
    1.0 = perfectly uniform degradation. 0.0 = all degradation in one cliff-drop."""
    if len(rates) < 2:
        return None

    steps = [max(0.0, rates[i] - rates[i + 1]) for i in range(len(rates) - 1)]
    total_drop = sum(steps)

    if total_drop == 0.0:
        return 1.0  # No degradation = trivially smooth

    nonzero = [s for s in steps if s > 0.0]
    n_nonzero = len(nonzero)

    if n_nonzero <= 1:
        return 1.0 if len(steps) == 1 else 0.0

    p = [s / total_drop for s in nonzero]
    H = -sum(pi * math.log(pi) for pi in p)
    H_max = math.log(n_nonzero)

    return H / H_max if H_max > 0 else 1.0


def compute_gds(
    events: List[Event],
    expected_levels: Optional[List[float]] = None,
) -> GDSResult:
    """
    BP-1 — Graceful Degradation Score (GDS)

    Spec-aligned definition:
      Execute at ordered stress intensity levels s1..sn
      Measure completion rate Ci at each level
      GDS = (1/n) * Σ Ci

    Data source:
      We accept per-level completion evidence as events with:
        - type == WORK_UNIT_END (optional per-task) OR
        - events that directly carry (stress_level, completion_rate)
      For v0 reference implementation, we prefer the explicit evidence events:
        Event with completion_rate != None and stress_level != None.

    expected_levels:
      If provided, enforce that we have data for each declared level (orderable).
    """
    levels: List[float] = []
    rates: List[float] = []

    # Prefer explicit evidence events (stress_level + completion_rate)
    for e in events:
        if e.stress_level is not None and e.completion_rate is not None:
            levels.append(float(e.stress_level))
            rates.append(float(e.completion_rate))

    if not levels:
        return GDSResult(
            gds=None,
            n_levels=0,
            stress_levels=[],
            completion_rates=[],
            na_reason="No (stress_level, completion_rate) evidence found in events.",
        )

    # Basic bounds check
    for r in rates:
        if not (0.0 <= r <= 1.0):
            return GDSResult(
                gds=None,
                n_levels=len(levels),
                stress_levels=levels,
                completion_rates=rates,
                na_reason=f"completion_rate out of bounds: {r}",
            )

    # If expected levels are declared, enforce coverage
    if expected_levels is not None:
        exp = [float(x) for x in expected_levels]
        # Use tolerance-based comparison to avoid float equality issues
        missing = [
            x for x in exp
            if not any(math.isclose(x, g, rel_tol=1e-9) for g in levels)
        ]
        if missing:
            return GDSResult(
                gds=None,
                n_levels=len(levels),
                stress_levels=levels,
                completion_rates=rates,
                na_reason=f"Missing declared stress levels: {missing}",
            )

    n = len(rates)
    gds = sum(rates) / n
    gds = max(0.0, min(1.0, gds))

    # Ensure stress levels are monotonically orderable (spec requirement)
    # We don’t force monotonic in events ordering, but we ensure they can be sorted and disclosed.
    paired = sorted(zip(levels, rates), key=lambda x: x[0])
    s_sorted = [p[0] for p in paired]
    c_sorted = [p[1] for p in paired]

    mono = _compute_monotonicity(c_sorted)
    smooth = _compute_smoothness(c_sorted)

    return GDSResult(
        gds=gds,
        n_levels=n,
        stress_levels=s_sorted,
        completion_rates=c_sorted,
        monotonicity=mono,
        smoothness=smooth,
        na_reason=None,
    )

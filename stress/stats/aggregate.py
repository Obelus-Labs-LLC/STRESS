from __future__ import annotations

import math
from dataclasses import dataclass
from typing import List, Optional


@dataclass(frozen=True)
class SummaryStats:
    mean: Optional[float]
    std: Optional[float]
    ci95_low: Optional[float]
    ci95_high: Optional[float]
    n_included: int
    n_na: int


# Two-tailed 95% t-distribution critical values for df=1..29.
# For df >= 30, normal approximation z=1.96 is adequate.
_T_CRIT_95 = {
    1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571,
    6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262, 10: 2.228,
    11: 2.201, 12: 2.179, 13: 2.160, 14: 2.145, 15: 2.131,
    16: 2.120, 17: 2.110, 18: 2.101, 19: 2.093, 20: 2.086,
    21: 2.080, 22: 2.074, 23: 2.069, 24: 2.064, 25: 2.060,
    26: 2.056, 27: 2.052, 28: 2.048, 29: 2.045,
}


def _mean(xs: List[float]) -> float:
    return sum(xs) / len(xs)


def _std_sample(xs: List[float]) -> float:
    n = len(xs)
    if n < 2:
        return 0.0
    m = _mean(xs)
    var = sum((x - m) ** 2 for x in xs) / (n - 1)
    return math.sqrt(var)


def summarize(values: List[Optional[float]]) -> SummaryStats:
    """
    Compute mean/std/95% CI over included values.
    N/A values are excluded but counted.
    95% CI uses normal approximation: mean +/- 1.96 * (std/sqrt(n))
    CI is reported faithfully without clamping.
    """
    included = [v for v in values if v is not None]
    n_na = sum(1 for v in values if v is None)

    if not included:
        return SummaryStats(
            mean=None, std=None, ci95_low=None, ci95_high=None,
            n_included=0, n_na=n_na
        )

    n = len(included)
    m = _mean(included)
    s = _std_sample(included)

    if n == 1:
        return SummaryStats(
            mean=m, std=0.0, ci95_low=m, ci95_high=m,
            n_included=1, n_na=n_na
        )

    se = s / math.sqrt(n)
    df = n - 1
    crit = _T_CRIT_95.get(df, 1.96)
    lo = m - crit * se
    hi = m + crit * se

    return SummaryStats(
        mean=m, std=s, ci95_low=lo, ci95_high=hi,
        n_included=n, n_na=n_na
    )

from __future__ import annotations

import math
import statistics
from dataclasses import dataclass
from typing import List, Optional, Tuple


@dataclass(frozen=True)
class SummaryStats:
    mean: Optional[float]
    std: Optional[float]
    ci95_low: Optional[float]
    ci95_high: Optional[float]
    n_included: int
    n_na: int
    bootstrap_ci95_low: Optional[float] = None
    bootstrap_ci95_high: Optional[float] = None


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

    boot_lo, boot_hi = None, None
    boot = bootstrap_ci(included)
    if boot is not None:
        boot_lo, boot_hi = boot

    return SummaryStats(
        mean=m, std=s, ci95_low=lo, ci95_high=hi,
        n_included=n, n_na=n_na,
        bootstrap_ci95_low=boot_lo, bootstrap_ci95_high=boot_hi,
    )


def cohens_d(group_a: List[float], group_b: List[float]) -> Optional[float]:
    """Cohen's d effect size between two groups.

    d = (mean_a - mean_b) / pooled_std
    Returns None if either group has fewer than 2 values or pooled std is zero.
    """
    if len(group_a) < 2 or len(group_b) < 2:
        return None
    n_a, n_b = len(group_a), len(group_b)
    m_a, m_b = _mean(group_a), _mean(group_b)
    s_a, s_b = _std_sample(group_a), _std_sample(group_b)
    pooled_var = ((n_a - 1) * s_a ** 2 + (n_b - 1) * s_b ** 2) / (n_a + n_b - 2)
    pooled_std = math.sqrt(pooled_var)
    if pooled_std == 0.0:
        return None
    return (m_a - m_b) / pooled_std


def mad_outliers(values: List[float], threshold: float = 3.5) -> List[int]:
    """Detect outliers using Modified Z-score with Median Absolute Deviation.

    Returns indices of outlier values where |modified_z| > threshold.
    """
    if len(values) < 3:
        return []
    med = statistics.median(values)
    abs_devs = [abs(x - med) for x in values]
    mad = statistics.median(abs_devs)
    if mad == 0.0:
        return []
    outliers = []
    for i, x in enumerate(values):
        modified_z = 0.6745 * (x - med) / mad
        if abs(modified_z) > threshold:
            outliers.append(i)
    return outliers


def bootstrap_ci(
    values: List[float], confidence: float = 0.95, n_resamples: int = 9999,
) -> Optional[Tuple[float, float]]:
    """Bootstrap BCa confidence interval. Requires scipy >= 1.7.

    Returns (low, high) or None if scipy is not installed or data is insufficient.
    """
    if len(values) < 2:
        return None
    try:
        from scipy.stats import bootstrap as sp_bootstrap
        import numpy as np
    except ImportError:
        return None
    data = (np.array(values),)
    result = sp_bootstrap(
        data, statistic=np.mean, n_resamples=n_resamples,
        confidence_level=confidence, method="BCa",
    )
    return (float(result.confidence_interval.low), float(result.confidence_interval.high))

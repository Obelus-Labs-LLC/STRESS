from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Dict, Optional


@dataclass(frozen=True)
class SRIResult:
    sri: Optional[float]
    weights: Dict[str, float]
    na_reason: Optional[str] = None


def compute_sri(
    proxies: Dict[str, Optional[float]],
    weights: Optional[Dict[str, float]] = None,
) -> SRIResult:
    """
    SRI via weighted geometric mean, producing a score on [0, 100].

    Unweighted: SRI = (GDS * ARR * IST * REC * CFR)^(1/5) * 100
    Weighted:   SRI = exp(sum(w_i * ln(proxy_i))) * 100

    Geometric mean ensures zero resilience in any dimension drives SRI toward 0,
    preventing weak-link masking that arithmetic mean allows.

    If any required proxy is N/A, SRI is N/A (must be disclosed).
    """
    if weights is None:
        weights = {"gds": 0.2, "arr": 0.2, "ist": 0.2, "rec": 0.2, "cfr": 0.2}

    weight_sum = sum(weights.values())
    if abs(weight_sum - 1.0) > 0.01:
        return SRIResult(sri=None, weights=weights, na_reason=f"weights sum to {weight_sum}, expected 1.0")

    missing = [k for k in weights.keys() if k not in proxies]
    if missing:
        return SRIResult(sri=None, weights=weights, na_reason=f"missing proxies: {missing}")

    na = [k for k in weights.keys() if proxies.get(k) is None]
    if na:
        return SRIResult(sri=None, weights=weights, na_reason=f"SRI N/A because proxies N/A: {na}")

    values = {k: float(proxies[k]) for k in weights.keys()}

    # Zero proxy -> SRI = 0 (zero resilience in any dimension = zero overall)
    if any(v == 0.0 for v in values.values()):
        return SRIResult(sri=0.0, weights=weights, na_reason=None)

    # Weighted geometric mean: exp(sum(w_i * ln(proxy_i))) * 100
    log_sum = sum(float(w) * math.log(values[k]) for k, w in weights.items())
    sri = math.exp(log_sum) * 100.0
    sri = max(0.0, min(100.0, sri))

    return SRIResult(sri=sri, weights=weights, na_reason=None)


# ---------------------------------------------------------------------------
# Domain weighting profiles
# ---------------------------------------------------------------------------

WEIGHT_PROFILES: Dict[str, Dict[str, float]] = {
    "satellite-leo": {
        "gds": 0.20, "arr": 0.20, "ist": 0.35, "rec": 0.15, "cfr": 0.10,
    },
    "data-center": {
        "gds": 0.20, "arr": 0.20, "ist": 0.10, "rec": 0.15, "cfr": 0.35,
    },
    "tactical-edge": {
        "gds": 0.25, "arr": 0.25, "ist": 0.20, "rec": 0.20, "cfr": 0.10,
    },
}


def compute_weighted_sri(
    proxies: Dict[str, Optional[float]],
    profile_name: str,
) -> SRIResult:
    """
    Compute weighted SRI using a named domain weighting profile.

    SRI = exp(sum(w_i * ln(proxy_i))) * 100 (weighted geometric mean).
    Returns N/A if any required proxy is missing/None or weights don't sum to ~1.0.
    """
    if profile_name not in WEIGHT_PROFILES:
        return SRIResult(
            sri=None,
            weights={},
            na_reason=f"unknown profile: {profile_name}",
        )

    weights = WEIGHT_PROFILES[profile_name]
    return compute_sri(proxies, weights=weights)

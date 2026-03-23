from __future__ import annotations

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
    SRI = (sum of weighted proxies) * 100, producing a score on [0, 100].

    Per spec v0.2: SRI = (GDS + ARR + IST + REC + CFR) / 5 × 100

    If any required proxy is N/A, SRI is N/A (must be disclosed).
    """
    if weights is None:
        weights = {"gds": 0.2, "arr": 0.2, "ist": 0.2, "rec": 0.2, "cfr": 0.2}

    missing = [k for k in weights.keys() if k not in proxies]
    if missing:
        return SRIResult(sri=None, weights=weights, na_reason=f"missing proxies: {missing}")

    na = [k for k in weights.keys() if proxies.get(k) is None]
    if na:
        return SRIResult(sri=None, weights=weights, na_reason=f"SRI N/A because proxies N/A: {na}")

    raw = 0.0
    for k, w in weights.items():
        raw += float(proxies[k]) * float(w)

    # Scale to [0, 100] per spec v0.2
    sri = raw * 100.0

    # Clamp to [0, 100]
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

    SRI = (sum of w_i * proxy_i) * 100, clamped to [0, 100].
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

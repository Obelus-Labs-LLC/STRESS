"""Tests for SRI aggregation (geometric mean)."""
import math
from stress.metrics.sri import compute_sri


def test_perfect_sri():
    proxies = {"gds": 1.0, "arr": 1.0, "ist": 1.0, "rec": 1.0, "cfr": 1.0}
    result = compute_sri(proxies)
    assert result.sri == 100.0


def test_partial_na_makes_sri_na():
    proxies = {"gds": 1.0, "arr": None, "ist": 1.0, "rec": 1.0, "cfr": 1.0}
    result = compute_sri(proxies)
    assert result.sri is None
    assert result.na_reason is not None


def test_all_na():
    proxies = {"gds": None, "arr": None, "ist": None, "rec": None, "cfr": None}
    result = compute_sri(proxies)
    assert result.sri is None


def test_geometric_mean():
    proxies = {"gds": 0.8, "arr": 0.6, "ist": 1.0, "rec": 0.4, "cfr": 0.2}
    result = compute_sri(proxies)
    expected = (0.8 * 0.6 * 1.0 * 0.4 * 0.2) ** (1.0 / 5.0) * 100.0
    assert abs(result.sri - expected) < 0.1


def test_zero_proxy_means_zero_sri():
    proxies = {"gds": 1.0, "arr": 1.0, "ist": 1.0, "rec": 1.0, "cfr": 0.0}
    result = compute_sri(proxies)
    assert result.sri == 0.0


def test_geometric_penalizes_imbalance():
    balanced = {"gds": 0.6, "arr": 0.6, "ist": 0.6, "rec": 0.6, "cfr": 0.6}
    imbalanced = {"gds": 1.0, "arr": 1.0, "ist": 1.0, "rec": 1.0, "cfr": 0.0476}
    # Both have arithmetic mean 0.6, but geometric mean penalizes the imbalanced one
    r_balanced = compute_sri(balanced)
    r_imbalanced = compute_sri(imbalanced)
    assert r_balanced.sri > r_imbalanced.sri


def test_invalid_weight_sum_returns_na():
    proxies = {"gds": 1.0, "arr": 1.0, "ist": 1.0, "rec": 1.0, "cfr": 1.0}
    weights = {"gds": 0.5, "arr": 0.5, "ist": 0.5, "rec": 0.5, "cfr": 0.5}
    result = compute_sri(proxies, weights=weights)
    assert result.sri is None
    assert "weights sum to" in result.na_reason

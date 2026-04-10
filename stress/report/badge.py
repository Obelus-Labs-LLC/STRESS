"""SRI badge generation for READMEs and reports."""
from __future__ import annotations

from pathlib import Path
from typing import Optional


def _sri_color(sri: float) -> str:
    if sri >= 85:
        return "brightgreen"
    elif sri >= 70:
        return "green"
    elif sri >= 50:
        return "yellow"
    elif sri >= 30:
        return "orange"
    return "red"


def generate_badge_url(sri: float, profile_name: str = "equal") -> str:
    """Generate a Shields.io badge URL for an SRI score."""
    color = _sri_color(sri)
    label = f"SRI ({profile_name})" if profile_name != "equal" else "SRI"
    label_encoded = label.replace(" ", "%20").replace("(", "%28").replace(")", "%29")
    return f"https://img.shields.io/badge/{label_encoded}-{sri:.1f}-{color}"


def generate_badge_svg(sri: float, out_path: str) -> Optional[Path]:
    """Generate a local SVG badge file using pybadges.

    Returns path or None if pybadges is unavailable.
    """
    try:
        from pybadges import badge
    except ImportError:
        return None

    color_map = {
        "brightgreen": "#4c1", "green": "#97ca00", "yellow": "#dfb317",
        "orange": "#fe7d37", "red": "#e05d44",
    }
    color = color_map.get(_sri_color(sri), "#999")
    svg = badge(left_text="SRI", right_text=f"{sri:.1f}", right_color=color)

    path = Path(out_path)
    path.write_text(svg, encoding="utf-8")
    return path

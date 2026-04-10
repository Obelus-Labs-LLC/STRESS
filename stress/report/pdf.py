"""PDF report generation from HTML."""
from __future__ import annotations

from pathlib import Path
from typing import Optional


def generate_pdf_report(html_path: str, pdf_path: str) -> Optional[Path]:
    """Convert an HTML report to PDF using WeasyPrint.

    Returns path to generated PDF or None if weasyprint is unavailable.
    """
    try:
        from weasyprint import HTML
    except ImportError:
        return None

    HTML(filename=html_path).write_pdf(pdf_path)
    return Path(pdf_path)

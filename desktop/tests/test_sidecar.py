"""End-to-end tests for the realestate-extract sidecar.

Run with:
    cd desktop/sidecar
    PYTHONPATH= ../../tests .venv/bin/python -m pytest ../tests/ -v

Or run the demo (no pytest needed):
    bash ../scripts/run_demo.sh
"""
from __future__ import annotations
import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

SIDECAR = Path(__file__).parent.parent / "sidecar" / "bin" / "realestate-extract"
SAMPLE_INVOICE = Path(__file__).parent / "sample_invoice.txt"
SAMPLE_LEASE   = Path(__file__).parent / "sample_lease.txt"


def _run_sidecar(doc_path: str | Path, schema_id: str) -> dict:
    """Invoke the sidecar and return its JSON response."""
    if not SIDECAR.exists():
        pytest.skip(f"sidecar not built: {SIDECAR}")
    env = os.environ.copy()
    env["PYTHONPATH"] = ""  # don't let callers' PYTHONPATH leak in
    env["NEEDLE_TELEMETRY"] = "0"
    r = subprocess.run(
        [str(SIDECAR)],
        input=json.dumps({"doc_path": str(doc_path), "schema_id": schema_id}),
        capture_output=True, text=True, timeout=120, env=env,
    )
    assert r.returncode == 0, f"sidecar exit {r.returncode}: stderr={r.stderr}"
    return json.loads(r.stdout)


def test_invoice_extraction_returns_valid_result():
    out = _run_sidecar(SAMPLE_INVOICE, "invoice")
    assert out["error"] is None, f"sidecar error: {out['error']}"
    assert out["extraction"] is not None
    assert out["confidences"], "confidences should be a non-empty dict"
    assert 0.0 <= out["mean_confidence"] <= 1.0
    # The invoice sample contains "Acme Property Services"
    assert "acme" in out["extraction"]["vendor_name"].lower()
    # Date fields must be ISO-8601
    assert out["extraction"]["invoice_date"].count("-") == 2
    assert out["extraction"]["due_date"].count("-") == 2
    # The model should have picked up the total
    assert out["extraction"]["total"] >= 1000  # the sample has 10850


def test_lease_extraction_returns_valid_result():
    out = _run_sidecar(SAMPLE_LEASE, "lease_addendum")
    assert out["error"] is None, f"sidecar error: {out['error']}"
    assert out["extraction"] is not None
    assert "Skyline" in out["extraction"]["landlord"]
    assert "Maria" in out["extraction"]["tenant"]
    assert "Ashbury" in out["extraction"]["property_address"]
    assert out["extraction"]["monthly_rent"] == 3150.0
    assert out["extraction"]["currency"] == "USD"


def test_sidecar_handles_missing_file():
    out = _run_sidecar("/nonexistent/never.pdf", "invoice")
    assert out["error"] is not None
    assert out["error"]["code"] in ("NOT_FOUND", "INGEST_ERROR")
    assert out["extraction"] is None


def test_sidecar_handles_unknown_schema():
    out = _run_sidecar(SAMPLE_INVOICE, "no_such_schema")
    assert out["error"] is not None
    assert "unknown schema" in out["error"]["message"].lower()


def test_sidecar_handles_unsupported_file_type():
    bad = Path(__file__).parent / "sample_unsupported.xyz"
    bad.write_text("not a real document")
    try:
        out = _run_sidecar(bad, "invoice")
        assert out["error"] is not None
        assert out["error"]["code"] in ("UNSUPPORTED", "INGEST_ERROR")
    finally:
        bad.unlink(missing_ok=True)


def test_sidecar_emits_well_formed_response_on_valid_input():
    """The response contract must always have these keys, even on error."""
    out = _run_sidecar(SAMPLE_INVOICE, "invoice")
    for key in ("extraction", "confidences", "latency_ms", "model_version",
                "schema_id", "schema_version", "error"):
        assert key in out, f"missing key {key!r}"
    assert isinstance(out["latency_ms"], int)
    assert out["latency_ms"] > 0

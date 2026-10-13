"""Unit tests for the user-defined-schema builder (Phase 2).

These tests are pure-Python; they do NOT spawn the sidecar. They exercise
`build_schema_from_user_def` and the validation rules. End-to-end
tests (user schema + actual extraction) live in `test_sidecar.py`.

Run with:
    cd desktop/sidecar
    PYTHONPATH= ../../tests .venv/bin/python -m pytest ../tests/test_user_schemas.py -v
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

# Make the sidecar package importable without running `maturin develop`.
SIDE_PKG = Path(__file__).parent.parent / "sidecar"
sys.path.insert(0, str(SIDE_PKG))

from realestate_extract.schemas import (  # noqa: E402
    USER_FIELD_TYPES,
    build_schema_from_user_def,
)


# ---------------------------------------------------------------------------
# Test fixtures
# ---------------------------------------------------------------------------
@pytest.fixture
def vendor_invoice_fields() -> list[dict]:
    """A realistic user-defined schema: a vendor invoice for a property
    management company. Mirrors the example in docs/user-schemas-and-tags.md."""
    return [
        {
            "name": "vendor_name",
            "label": "Vendor Name",
            "type": "short_text",
            "description": "Name of the company that issued the invoice",
            "required": True,
        },
        {
            "name": "amount",
            "label": "Total Amount",
            "type": "number",
            "description": "Total amount due in USD",
            "required": True,
            "min": 0,
        },
        {
            "name": "due_date",
            "label": "Due Date",
            "type": "date",
            "required": False,
        },
        {
            "name": "status",
            "label": "Status",
            "type": "choice",
            "required": True,
            "options": ["Pending", "Paid", "Overdue", "Disputed"],
        },
    ]


# ---------------------------------------------------------------------------
# Happy path
# ---------------------------------------------------------------------------
def test_builds_pydantic_class_from_fields(vendor_invoice_fields):
    cls = build_schema_from_user_def(vendor_invoice_fields)
    # Must be a Pydantic BaseModel subclass
    from pydantic import BaseModel
    assert issubclass(cls, BaseModel)
    # Must have all 4 field names
    for f in ["vendor_name", "amount", "due_date", "status"]:
        assert f in cls.model_fields


def test_required_field_must_be_set(vendor_invoice_fields):
    cls = build_schema_from_user_def(vendor_invoice_fields)
    # vendor_name is required, must be provided
    with pytest.raises(Exception):
        cls(amount=100.0, status="Pending")
    # All required → ok
    m = cls(vendor_name="Acme", amount=100.0, status="Pending")
    assert m.vendor_name == "Acme"
    assert m.amount == 100.0
    assert m.status == "Pending"


def test_optional_field_defaults_to_none(vendor_invoice_fields):
    cls = build_schema_from_user_def(vendor_invoice_fields)
    m = cls(vendor_name="Acme", amount=100.0, status="Pending")
    # due_date is optional → defaults to None
    assert m.due_date is None


def test_choice_field_rejects_invalid_value(vendor_invoice_fields):
    cls = build_schema_from_user_def(vendor_invoice_fields)
    with pytest.raises(Exception):
        cls(vendor_name="Acme", amount=100.0, status="NotAChoice")


def test_date_field_validates_iso_format(vendor_invoice_fields):
    cls = build_schema_from_user_def(vendor_invoice_fields)
    # Good date
    m = cls(vendor_name="Acme", amount=100.0, status="Pending", due_date="2026-10-13")
    assert m.due_date == "2026-10-13"
    # Bad date
    with pytest.raises(Exception):
        cls(vendor_name="Acme", amount=100.0, status="Pending", due_date="10/13/2026")


def test_number_field_respects_min(vendor_invoice_fields):
    cls = build_schema_from_user_def(vendor_invoice_fields)
    # min=0; negative must fail
    with pytest.raises(Exception):
        cls(vendor_name="Acme", amount=-10.0, status="Pending")
    # zero is fine
    m = cls(vendor_name="Acme", amount=0.0, status="Pending")
    assert m.amount == 0.0


# ---------------------------------------------------------------------------
# Determinism (cacheable)
# ---------------------------------------------------------------------------
def test_same_input_same_class(vendor_invoice_fields):
    a = build_schema_from_user_def(vendor_invoice_fields)
    b = build_schema_from_user_def(vendor_invoice_fields)
    assert a is b, "same field set should produce identical class (cacheable)"


def test_different_input_different_class():
    a = build_schema_from_user_def([
        {"name": "foo", "label": "Foo", "type": "short_text", "required": True}
    ])
    b = build_schema_from_user_def([
        {"name": "foo", "label": "Foo", "type": "short_text", "required": False}
    ])
    assert a is not b, "required vs optional is a different class"


# ---------------------------------------------------------------------------
# Validation errors
# ---------------------------------------------------------------------------
def test_empty_fields_raises():
    with pytest.raises(ValueError, match="at least one field"):
        build_schema_from_user_def([])


def test_unknown_field_type_raises():
    with pytest.raises(ValueError, match="unknown type"):
        build_schema_from_user_def([
            {"name": "foo", "label": "Foo", "type": "json_object", "required": True}
        ])


def test_duplicate_field_name_raises():
    with pytest.raises(ValueError, match="duplicate"):
        build_schema_from_user_def([
            {"name": "foo", "label": "Foo", "type": "short_text", "required": True},
            {"name": "foo", "label": "Foo2", "type": "short_text", "required": True},
        ])


def test_uppercase_field_name_raises():
    with pytest.raises(ValueError, match="snake_case"):
        build_schema_from_user_def([
            {"name": "VendorName", "label": "V", "type": "short_text", "required": True}
        ])


def test_choice_without_options_raises():
    with pytest.raises(ValueError, match="at least one option"):
        build_schema_from_user_def([
            {"name": "status", "label": "Status", "type": "choice", "required": True, "options": []}
        ])


def test_too_many_fields_raises():
    fields = [
        {"name": f"f{i}", "label": f"F{i}", "type": "short_text", "required": True}
        for i in range(51)
    ]
    with pytest.raises(ValueError, match="too many"):
        build_schema_from_user_def(fields)


# ---------------------------------------------------------------------------
# Field type coverage
# ---------------------------------------------------------------------------
def test_all_field_types_in_vocabulary():
    """USER_FIELD_TYPES must include the documented user-facing types."""
    expected = {"short_text", "long_text", "number", "date", "yes_no", "choice", "list"}
    assert expected.issubset(USER_FIELD_TYPES)


def test_yes_no_field():
    cls = build_schema_from_user_def([
        {"name": "is_signed", "label": "Signed?", "type": "yes_no", "required": True}
    ])
    m = cls(is_signed=True)
    assert m.is_signed is True
    with pytest.raises(Exception):
        cls(is_signed="yes")  # must be bool, not "yes"/"no"


def test_list_field():
    cls = build_schema_from_user_def([
        {"name": "defects", "label": "Defects", "type": "list", "required": True}
    ])
    m = cls(defects=["leaky faucet", "broken window"])
    assert m.defects == ["leaky faucet", "broken window"]


# ---------------------------------------------------------------------------
# JSON roundtrip
# ---------------------------------------------------------------------------
def test_json_roundtrip(vendor_invoice_fields):
    """The field list must survive a JSON roundtrip (it does, since it's
    the wire format the Rust side uses)."""
    s = json.dumps(vendor_invoice_fields)
    cls = build_schema_from_user_def(json.loads(s))
    m = cls(vendor_name="Acme", amount=100.0, status="Pending")
    assert m.vendor_name == "Acme"

# RealEstate-Bench — Built-in Schemas
# Pydantic v2 models, each with field-level needle.Field() constraints
# (grammar-constrained decode) and per-field descriptions.

from __future__ import annotations
from typing import Annotated, Literal, Optional
from pydantic import BaseModel, Field, create_model
import needle

# ---------------------------------------------------------------------------
# Lease Addendum — primary first-use-case
# ---------------------------------------------------------------------------
class LeaseAddendum(BaseModel):
    """Structured extraction for a residential or commercial lease addendum.

    Intended document type: 1-10 page PDF/photo of a signed lease addendum,
    renewal, or amendment. Works on born-digital and OCR'd scans.
    """

    landlord: Annotated[str, needle.Field(description="Legal name of the landlord, property owner, or lessor entity")]
    tenant:   Annotated[str, needle.Field(description="Legal name of the tenant or lessee entity")]
    property_address: Annotated[str, needle.Field(description="Full street address of the leased property, including unit/suite if any")]
    monthly_rent: Annotated[float, needle.Field(gt=0, le=10_000_000, description="Monthly rent amount in the stated currency (numeric, no symbol)")]
    currency: Annotated[Literal["USD", "CAD", "EUR", "GBP", "MXN", "AUD", "INR", "JPY", "CHF", "SGD"], needle.Field(description="ISO 4217 currency code")]
    start_date: Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Lease start date in ISO 8601 (YYYY-MM-DD) format")]
    end_date:   Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Lease end date in ISO 8601 (YYYY-MM-DD) format")]
    security_deposit: Annotated[Optional[float], needle.Field(ge=0, le=10_000_000, description="Security deposit amount in the same currency; null if not stated")] = None
    renewal_option: Annotated[Literal["none", "month-to-month", "1-year", "2-year", "3-year", "5-year", "other"], needle.Field()] = "none"
    pets_allowed: Annotated[Literal["yes", "no", "with-deposit", "case-by-case"], needle.Field()] = "no"
    notes: Annotated[str, needle.Field(description="Any other material terms not captured above; empty string if none")] = ""


# ---------------------------------------------------------------------------
# Invoice — generic invoice extraction
# ---------------------------------------------------------------------------
class Invoice(BaseModel):
    """Structured extraction for a vendor invoice / bill / receipt.

    Intended document type: 1-3 page invoice from a vendor, contractor, or
    utility. Captures vendor, line items, totals, payment terms.
    """

    vendor_name: Annotated[str, needle.Field(description="Name of the vendor or supplier issuing the invoice")]
    vendor_address: Annotated[Optional[str], needle.Field(description="Vendor's full mailing address; null if not present")] = None
    invoice_number: Annotated[str, needle.Field(description="Vendor's invoice number / ID")]
    invoice_date: Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Date the invoice was issued, in YYYY-MM-DD format")]
    due_date: Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Date payment is due, in YYYY-MM-DD format")]
    currency: Annotated[Literal["USD", "CAD", "EUR", "GBP", "MXN", "AUD", "INR", "JPY", "CHF", "SGD"], needle.Field()]
    subtotal: Annotated[float, needle.Field(ge=0, le=1_000_000_000, description="Subtotal before tax in the stated currency")]
    tax:      Annotated[float, needle.Field(ge=0, le=1_000_000_000, description="Total tax amount in the stated currency; 0 if not stated")]
    total:    Annotated[float, needle.Field(gt=0, le=1_000_000_000, description="Total amount due in the stated currency")]
    payment_terms: Annotated[Literal["due-on-receipt", "net-15", "net-30", "net-45", "net-60", "cod", "prepaid", "other"], needle.Field()] = "net-30"
    notes: str = ""


# ---------------------------------------------------------------------------
# Rent Roll — one row per unit in a property
# ---------------------------------------------------------------------------
class RentRoll(BaseModel):
    """Structured extraction for a single unit's row in a property rent roll.

    Intended document type: a row of a rent roll spreadsheet or a single
    unit summary; the document should mention unit, tenant, sqft, and rent.
    For multi-row rent rolls, run this schema once per row (split the doc first).
    """

    property_name: Annotated[str, needle.Field(description="Name of the property / building")]
    unit_number:   Annotated[str, needle.Field(description="Unit identifier (e.g. '101', 'A-3', 'Penthouse')")]
    tenant_name:   Annotated[Optional[str], needle.Field(description="Current tenant name; null if vacant")] = None
    square_feet:   Annotated[Optional[int], needle.Field(gt=0, le=1_000_000, description="Rentable square feet; null if not stated")] = None
    monthly_rent:  Annotated[float, needle.Field(ge=0, le=10_000_000, description="Current monthly rent in USD")]
    rent_per_sqft: Annotated[Optional[float], needle.Field(gt=0, le=1000, description="Monthly rent per sqft (USD); null if not stated")] = None
    lease_start:   Annotated[Optional[str], needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Current lease start, YYYY-MM-DD; null if vacant")] = None
    lease_end:     Annotated[Optional[str], needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Current lease end, YYYY-MM-DD; null if vacant")] = None
    status:        Annotated[Literal["occupied", "vacant", "notice-to-vacate", "model-unit", "down"], needle.Field()] = "occupied"


# ---------------------------------------------------------------------------
# Medical SOAP Note — first non-real-estate industry pack (proves §3 of
# docs/extending-to-other-industries.md works on this codebase).
# Tested with a fictional, de-identified encounter note (no PHI).
# ---------------------------------------------------------------------------
class MedicalSoapNote(BaseModel):
    """Structured extraction for a clinical SOAP note.

    Intended document type: 1-2 page encounter summary from a primary-care
    or specialty visit. The engine does not persist PHI itself; the user
    controls where the JSON goes. See docs/extending-to-other-industries.md §4.2
    for the production-redaction recipe.
    """

    patient_id:            Annotated[str, needle.Field(pattern=r"^P-[0-9]{6}$", description="De-identified patient ID, format P-NNNNNN")]
    encounter_date:        Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Date of encounter in YYYY-MM-DD")]
    chief_complaint:       Annotated[str, needle.Field(description="Primary reason for the visit, in patient's own words or provider's summary")]
    hpi:                   Annotated[str, needle.Field(description="History of present illness: symptom onset, duration, severity, alleviating/aggravating factors")]
    past_medical_history:  Annotated[str, needle.Field(description="Relevant past medical history, comma-separated conditions; empty if none")]
    medications:           Annotated[str, needle.Field(description="Current medications as free text, semicolon-separated; empty if none")]
    allergies:             Annotated[str, needle.Field(description="Known drug allergies, semicolon-separated; 'NKDA' if none")]
    vital_signs:           Annotated[Optional[str], needle.Field(description="Vitals if recorded (BP, HR, T, RR, SpO2); null if absent")] = None
    physical_exam:         Annotated[str, needle.Field(description="Physical exam findings by system; empty if not documented")]
    assessment:            Annotated[list[str], needle.Field(description="Provider's diagnostic assessment, one ICD-10-style description per list item, empty list if not stated")]
    plan:                  Annotated[list[str], needle.Field(description="Treatment plan items: medications prescribed, tests ordered, follow-up timing")]
    follow_up:             Annotated[Optional[str], needle.Field(description="Follow-up instructions, e.g. 'return in 2 weeks'; null if none")] = None
    provider_name:         Annotated[str, needle.Field(description="Name of the documenting provider")]


# ---------------------------------------------------------------------------
# Registry — used by both the sidecar and the Tauri app
# ---------------------------------------------------------------------------
SCHEMA_REGISTRY: dict[str, type[BaseModel]] = {
    "lease_addendum":   LeaseAddendum,
    "invoice":          Invoice,
    "rent_roll":        RentRoll,
    "medical_soap_note": MedicalSoapNote,
}


def list_schemas() -> list[dict]:
    """Return metadata for all built-in schemas (id, name, field count, version)."""
    return [
        {
            "id":     schema_id,
            "name":   schema_cls.__name__,
            "doc":    (schema_cls.__doc__ or "").strip().splitlines()[0] if schema_cls.__doc__ else "",
            "fields": list(schema_cls.model_fields.keys()),
            "version": "1.0.0",
        }
        for schema_id, schema_cls in SCHEMA_REGISTRY.items()
    ]


def get_schema(schema_id: str) -> type[BaseModel]:
    """Look up a schema class by id; raises KeyError if not found."""
    return SCHEMA_REGISTRY[schema_id]


# ---------------------------------------------------------------------------
# Dynamic Pydantic schema from a user-defined field list (Phase 2)
# ---------------------------------------------------------------------------
#
# A non-technical user creates a schema in the desktop app. The Rust side
# stores it in SQLite as a JSON array of fields. When the user runs an
# extraction, the Rust side forwards that JSON to the sidecar, and we
# build a Pydantic v2 BaseModel dynamically. Same model every time the
# same JSON is passed in (the model name is derived from the field set
# so dynamic model creation is deterministic and cacheable).

import hashlib
from typing import Annotated, Any, Literal, Optional

# Field-type vocabulary. Keep in lock-step with the UserFieldType enum on
# the Rust side (desktop/src-tauri/src/user_schemas.rs).
USER_FIELD_TYPES = {
    "short_text",
    "long_text",
    "number",
    "date",
    "yes_no",
    "choice",
    "list",
}

MAX_DESCRIPTION = 2000
MAX_LABEL = 80
MAX_NAME = 64
MAX_FIELDS = 50


def _stable_model_name(fields: list[dict]) -> str:
    """Generate a deterministic Pydantic class name from the field set.

    Determinism lets pydantic cache the model across calls with the same
    definition, which matters when a user clicks "extract" multiple times
    on the same schema.
    """
    h = hashlib.sha256()
    for f in fields:
        h.update(f.get("name", "").encode())
        h.update(b"|")
        h.update(f.get("type", "").encode())
        h.update(b"|")
        for o in f.get("options", []):
            h.update(o.encode())
            h.update(b",")
        h.update(b"|")
        h.update(b"1" if f.get("required") else b"0")
        h.update(b";")
    return f"UserSchema_{h.hexdigest()[:12]}"


def _make_needle_field(spec: dict) -> Any:
    """Build a `needle.Field(...)` constraint for one user field.

    Pydantic v2 does NOT honor constraints stored only on a `needle.Field`
    metadata object (they're not Pydantic FieldInfo instances). To make
    Pydantic actually validate the value (date pattern, number min/max),
    we wrap the needle constraint in a Pydantic Field() using
    `json_schema_extra` to pass through the metadata for cactus-needle,
    AND set the Pydantic-native constraints directly. Both layers see
    the constraint; the user benefits from BOTH:
      - Pydantic enforces pattern/ge/le on input.
      - cactus-needle reads the description and uses it as the LLM prompt.
    """
    desc = spec.get("description") or ""
    if spec.get("type") == "choice" and spec.get("options"):
        if desc:
            desc = desc.rstrip(".") + f"; allowed values: {', '.join(spec['options'])}"
        else:
            desc = f"Pick one of: {', '.join(spec['options'])}"
    pyd_kwargs: dict[str, Any] = {"description": desc or None}
    if spec.get("type") == "date":
        pyd_kwargs["pattern"] = r"^\d{4}-\d{2}-\d{2}$"
    if spec.get("type") == "number":
        if (mn := spec.get("min")) is not None:
            pyd_kwargs["ge"] = mn
        if (mx := spec.get("max")) is not None:
            pyd_kwargs["le"] = mx
    if spec.get("type") == "yes_no":
        # Pydantic v2 coerces "yes"/"no"/1/0/strings; we want strict bool
        # because that's what the LLM should be emitting.
        pyd_kwargs["strict"] = True
    return needle.Field(description=desc), pyd_kwargs


def build_schema_from_user_def(fields: list[dict]) -> type[BaseModel]:
    """Build a Pydantic v2 BaseModel class from a user-defined field list.

    The same input always produces the same class (deterministic name), so
    the Pydantic model cache and any `__hash__` consumers stay consistent.

    Implementation: use `pydantic.create_model` and pass each field as
    `(name, Annotated[T, needle.Field(...)], FieldInfo_kwargs)`. The
    needle constraint is what cactus-needle reads; the pydantic.Field()
    constraints are what enforces pattern/min/max/strict at validation
    time.
    """
    if not fields:
        raise ValueError("user schema must have at least one field")
    if len(fields) > MAX_FIELDS:
        raise ValueError(f"too many fields (>{MAX_FIELDS})")

    # Validate first
    seen_names: set[str] = set()
    for i, f in enumerate(fields):
        if not isinstance(f, dict):
            raise ValueError(f"field #{i}: not an object")
        name = (f.get("name") or "").strip()
        if not name:
            raise ValueError(f"field #{i}: name is required")
        if len(name) > MAX_NAME:
            raise ValueError(f"field #{i}: name too long")
        if not all(c.isascii() and (c.islower() or c.isdigit() or c == "_") for c in name):
            raise ValueError(
                f"field #{i}: name must be snake_case ASCII (got {name!r})"
            )
        if name in seen_names:
            raise ValueError(f"field #{i}: duplicate name {name!r}")
        seen_names.add(name)
        ftype = f.get("type")
        if ftype not in USER_FIELD_TYPES:
            raise ValueError(f"field #{i}: unknown type {ftype!r}")
        if (label := f.get("label", "")) and len(label) > MAX_LABEL:
            raise ValueError(f"field #{i}: label too long")
        if (desc := f.get("description", "")) and len(desc) > MAX_DESCRIPTION:
            raise ValueError(f"field #{i}: description too long")
        if ftype == "choice":
            opts = f.get("options") or []
            if not opts:
                raise ValueError(f"field #{i}: choice needs at least one option")

    name = _stable_model_name(fields)

    cm_kwargs: dict[str, Any] = {}
    for f in fields:
        fname = f["name"]
        ftype = f["type"]
        required = bool(f.get("required"))
        needle_field, pyd_kwargs = _make_needle_field(f)
        base: Any
        if ftype in ("short_text", "long_text", "date"):
            base = str
        elif ftype == "number":
            base = float
        elif ftype == "yes_no":
            base = bool
        elif ftype == "choice":
            opts = f.get("options") or []
            base = Literal[tuple(opts)]
        elif ftype == "list":
            base = list[str]
        else:
            raise ValueError(f"unknown field type: {ftype!r}")  # unreachable

        # The annotation: always carry needle.Field so cactus-needle sees
        # the description; on the class-level default we attach a
        # pydantic.Field() with the strict constraints.
        annotation = Annotated[base, needle_field]
        if required:
            cm_kwargs[fname] = (annotation, Field(**pyd_kwargs))
        else:
            pyd_kwargs_with_default = {**pyd_kwargs, "default": None}
            cm_kwargs[fname] = (Optional[annotation], Field(**pyd_kwargs_with_default))

    # Cache: identical field list → identical class object
    cls = _SCHEMA_CACHE.get(name)
    if cls is not None:
        return cls
    cls = create_model(name, **cm_kwargs)
    cls.__doc__ = "User-defined schema (built dynamically from the app's schema builder)"
    _SCHEMA_CACHE[name] = cls
    return cls


# Module-level cache so identical field lists → identical class object.
_SCHEMA_CACHE: dict[str, type] = {}

# RealEstate-Bench — Built-in Schemas
# Pydantic v2 models, each with field-level needle.Field() constraints
# (grammar-constrained decode) and per-field descriptions.

from __future__ import annotations
from typing import Annotated, Literal, Optional
from pydantic import BaseModel, Field
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
# Registry — used by both the sidecar and the Tauri app
# ---------------------------------------------------------------------------
SCHEMA_REGISTRY: dict[str, type[BaseModel]] = {
    "lease_addendum": LeaseAddendum,
    "invoice":        Invoice,
    "rent_roll":      RentRoll,
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

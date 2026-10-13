# User-Defined Schemas & Tags

**Status:** spec (Phase 2 feature)
**Date:** 2026-10-13
**Audience:** product, designers, engineers implementing Phase 2

## Why this matters

RealEstate-Bench ships with 4 hardcoded schemas (lease addendum, invoice, rent roll, medical SOAP). The user has a property management company, a real estate law firm, a fund, or a brokerage — they have **their own document types and their own fields**.

- A property manager has *vendor invoices* and *CAM reconciliations* — built-in
  Invoice is close but not right.
- A brokerage has *purchase contracts* with 30 fields that vary by state.
- A property management company has *inspection reports* with categorical
  pass/fail items.

**The current model** (hardcoded Pydantic classes, hardcoded Rust `SCHEMAS`
const) is wrong for this. The user should be able to define their own.

## What the user can do (Phase 2)

1. **Create a schema** in the desktop app — name it, describe it, add
   fields with types and descriptions. No code, no terminal.
2. **Pick it from the schema picker** the same way they pick "Lease Addendum."
3. **Extract** a document with their custom schema.
4. **Tag** the result with free-form labels ("Q3 2026", "Property XYZ",
   "Reviewed", "Disputed").
5. **Filter** the history by tag.
6. **Edit / delete** their custom schema at any time.
7. **Built-in schemas stay** — lease addendum, invoice, rent roll, medical
   SOAP. They are not editable; user schemas are.

## Field types for non-technical users

The field-type vocabulary is small and concrete. The user picks one from
a dropdown; the system maps it to a Pydantic + needle schema.

| User-facing label | Pydantic / needle representation | Example use |
|---|---|---|
| **Short text** | `str` | Tenant name, vendor name, address |
| **Long text** | `str` (description hints long-form) | Notes, description, comments |
| **Number** | `float` (with optional min/max) | Sqft, count, percent |
| **Date** | `str` with `^\d{4}-\d{2}-\d{2}$` pattern | Lease start, due date |
| **Yes / No** | `bool` | Signed, renewed, paid in full |
| **Choice** | `Literal["A", "B", "C"]` from options | Status, class, type |
| **List of items** | `list[str]` | Co-tenants, line items, defects |

> **Why no "money" type?** A user-defined schema is about *what the user
> sees*, not strict validation. We expose "Number" with an optional unit
> label the user types in the description. The system doesn't try to
> enforce currency precision — that's the user's call.

## Data model (SQLite)

Two new tables; one column added.

```sql
-- User-defined schemas. Built-in schemas are NOT stored here.
CREATE TABLE user_schemas (
    id           TEXT PRIMARY KEY,           -- e.g. "u_8f3a2c" (generated short id)
    name         TEXT NOT NULL,              -- "Vendor Invoice"
    description  TEXT NOT NULL DEFAULT '',   -- user-written prompt for the LLM
    fields_json  TEXT NOT NULL,              -- JSON array of field definitions
    created_at   TEXT NOT NULL,              -- ISO-8601
    updated_at   TEXT NOT NULL,              -- ISO-8601
    is_archived  INTEGER NOT NULL DEFAULT 0
);

-- Tags on extractions. Free-form strings, normalized to lowercase + trim.
ALTER TABLE extractions ADD COLUMN tags_json TEXT NOT NULL DEFAULT '[]';

CREATE INDEX idx_extraction_tags ON extractions(tags_json);
```

The `fields_json` shape:

```json
[
  {
    "name": "vendor_name",
    "label": "Vendor Name",
    "type": "short_text",
    "description": "Name of the company that issued the invoice",
    "required": true
  },
  {
    "name": "amount",
    "label": "Total Amount",
    "type": "number",
    "description": "Total amount due in USD",
    "required": true,
    "min": 0
  },
  {
    "name": "due_date",
    "label": "Due Date",
    "type": "date",
    "required": false
  },
  {
    "name": "status",
    "label": "Status",
    "type": "choice",
    "required": true,
    "options": ["Pending", "Paid", "Overdue", "Disputed"]
  }
]
```

## Architecture: where the user schema runs

The current pipeline is `Rust → sidecar.py → cactus-needle`. Built-in
schemas are Pydantic classes in `schemas.py` and registered in
`SCHEMA_REGISTRY`. The sidecar does:

```python
schema_cls = SCHEMA_REGISTRY[schema_id]  # e.g. LeaseAddendum
result = needle.extract(text, schema_cls)
```

For user schemas, we add a parallel path:

```python
schema_cls = build_schema_from_user_def(user_def_dict)  # dynamic Pydantic model
result = needle.extract(text, schema_cls)
```

`build_schema_from_user_def` is a new function in `schemas.py` that
takes the JSON definition and returns a Pydantic v2 `BaseModel`
class. It's deterministic — the same definition always produces the
same class. (Determinism is important for caching.)

The sidecar accepts a second CLI command: `extract_user <doc> <schema_json_path>`.
The Rust side knows whether a schema_id is built-in or user-defined and
dispatches to the right command.

## Rust IPC surface (new commands)

| Command | Args | Returns |
|---|---|---|
| `list_schemas` | (none) | `{built_in: [...], user: [...]}` |
| `get_user_schema` | `id: String` | full user schema (or null) |
| `create_user_schema` | `{name, description, fields: [...]}` | `{id, name, ...}` |
| `update_user_schema` | `{id, name?, description?, fields?}` | updated schema |
| `delete_user_schema` | `id: String` | `{ok: true}` |
| `add_extraction_tag` | `{extraction_id, tag}` | `{tags: [...]}` |
| `remove_extraction_tag` | `{extraction_id, tag}` | `{tags: [...]}` |
| `list_all_tags` | (none) | `[{tag, count}]` |
| `filter_extractions_by_tag` | `tag: String` | `[{extraction_id, ...}]` |

## React UI

Three changes:

1. **New page: `SchemasPage`** — at route `/schemas`. Lists built-in (read-only)
   and user (editable) schemas. "New Schema" button opens the
   `SchemaBuilder` modal.

2. **New component: `SchemaBuilder`** — a modal with:
   - Schema name (single text input)
   - Description (multi-line text, optional)
   - Field list with Add / Remove / Reorder
   - Each field: Label, Field Key (auto from label, editable), Type
     dropdown, Description, Required toggle, plus options list for Choice
   - Save / Cancel buttons

3. **New component: `TagPicker`** — a chip-style input that shows existing
   tags as chips and lets the user add a new one. Used on `ExtractionView`.

4. **Update `ExtractionView`** — render the TagPicker below the extracted
   fields. Calls `add_extraction_tag` / `remove_extraction_tag` on
   changes.

5. **Update `DropPage`** — the schema picker now shows user-defined
   schemas grouped under "Your schemas" with the same UI affordance
   as built-in ones.

6. **Update sidebar** — add "Schemas" link.

7. **Update `HistoryPage`** (if it exists) — add a tag filter dropdown.

## What does **not** change

- The extraction engine (cactus-needle + the sidecar) — same model, same
  IPC, same DB layout for the rest of it.
- The built-in schemas — they stay hardcoded, versioned, and unaffected
  by user schema changes.
- The export pipeline — Excel, JSON, CSV exports already work on the
  `extraction.extraction_json` column; user-schema extractions flow
  through unchanged.
- The Rust `SCHEMAS` const in `commands.rs` — it stays for built-in
  schemas only; user schemas are loaded from SQLite on the sidecar path.

## Test plan

1. **Unit tests (Rust, `cargo test`):**
   - `create_user_schema` persists a row, returns the id.
   - `update_user_schema` updates the row, bumps `updated_at`.
   - `delete_user_schema` removes the row.
   - `add_extraction_tag` updates the extraction's `tags_json`.
   - `list_all_tags` returns the right counts.

2. **Unit tests (Python, `pytest`):**
   - `build_schema_from_user_def` produces a Pydantic model with the
     right fields, types, and constraints.
   - `extract_user` CLI runs end-to-end on a sample document.

3. **End-to-end test (manual + automated smoke):**
   - Create a user schema "Vendor Invoice" with 4 fields.
   - Drop a sample invoice PDF.
   - Pick "Vendor Invoice" from the schema picker.
   - Verify the JSON output has all 4 fields populated.
   - Add a tag "Q3 2026".
   - Filter history by that tag.
   - Edit the schema (add a field). Re-run. Verify the new field
     is populated and the old ones didn't break.

## Migration path

The two schema changes (new table, new column) run on app start. They're
additive — no data loss. The migration lives in `db.rs::init`.

## Open questions

1. **Schema versioning** — when a user edits a schema, what happens to
   extractions done with the old version? **MVP:** keep both. The
   extraction stores the schema definition that was used at the time
   (denormalized into `extractions.schema_def_json`).
2. **Sharing** — can a user export a schema to share with a colleague?
   **MVP:** out of scope. JSON export of the schema definition is
   trivial to add later.
3. **Field constraints beyond Choice** — regex pattern, min/max length.
   **MVP:** only Choice options are exposed. Number has min/max. Future
   versions can add a "Pattern" field type.
4. **Multi-page schemas** — one schema per document, or one schema that
   produces an array? **MVP:** one schema = one extraction. The existing
   pattern (run schema once per document) holds.

## Implementation order

1. Migration + new tables (Rust)
2. `build_schema_from_user_def` (Python)
3. New `extract_user` CLI command (Python)
4. New Rust IPC commands
5. `SchemasPage` + `SchemaBuilder` component (React)
6. `TagPicker` component (React)
7. `ExtractionView` integration
8. `DropPage` schema picker update
9. Tests
10. Commit + push

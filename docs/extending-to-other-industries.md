# Extending RealEstate-Bench to Other Industries

**Status:** Living document. Updated 2026-10-13.
**Audience:** developers adding a new industry pack, and reviewers evaluating portability.
**Scope:** the desktop app's Python sidecar, its Rust IPC, and the React UI's schema picker.

> The "backend engine" is **cactus-needle 3.2.1** (Apache-2.0, 45M
> params) called through one function — `needle.extract(text, schema_cls)`
> — wrapped by a 5.6 KB Python sidecar. To add a new industry, you
> define a Pydantic v2 schema and register it. The model is generic;
> the schema is the industry.

---

## 1. The engine contract

Everything the engine needs to do for any industry is already in
`desktop/sidecar/realestate_extract/cli.py`:

```python
result = needle.extract(text, schema_cls, strict=strict)
# result.extraction      -> dict matching the schema
# result.confidences     -> {field: float in [0,1]} per field
# result.latency_ms      -> int
```

`text` is whatever your document parser produces (PDF text, image
OCR'd text, or already-plain text). `schema_cls` is a Pydantic v2
`BaseModel` annotated with `needle.Field(...)`. **The model is
schema-agnostic**; the industry is encoded entirely in the schema
definition, plus a JSON Schema file the sidecar can ship with the
.app bundle for downstream consumers.

The wire format the sidecar exposes to the Rust core:

```json
// request
{"doc_path": "/abs/path.pdf", "schema_id": "medical_soap_note"}

// response
{
  "extraction":       {...schema fields...},
  "confidences":      {"chief_complaint": 0.92, ...},
  "mean_confidence":  0.86,
  "latency_ms":       487,
  "model_version":    "cactus-needle-3.2.1",
  "schema_id":        "medical_soap_note",
  "schema_version":   "1.0.0",
  "doc_meta":         {"page_count": 1, "chars": 3120},
  "error":            null
}
```

If you keep the Pydantic class and the request/response format, the
Rust core, the SQLite store, the FTS5 index, the Excel/JSON/CSV
exports, and the React UI **all work unchanged** for your new industry.

---

## 2. The four touch points

Adding a new industry pack means editing **four files** and adding
**one sample document** (plus its JSON Schema sidecar). Here's the
diff, with the line numbers from the current `main` branch:

| # | File | What to add | Lines |
|---|------|-------------|-------|
| 1 | `desktop/sidecar/realestate_extract/schemas.py` | Pydantic v2 class + `SCHEMA_REGISTRY[id] = Class` | 13–85 |
| 2 | `desktop/src-tauri/src/commands.rs` | Rust tuple `(id, name, doc, &fields)` in the `SCHEMAS` const | 30–49 |
| 3 | `desktop/src/pages/DropPage.tsx` | Default `schemaId` in `useState` (optional) | 10 |
| 4 | `desktop/src-tauri/tauri.conf.json` | JSON Schema resource under `bundle.resources` | 42–44 |
| 5 | `desktop/tests/sample_<id>.txt` | One representative document (≥300 chars) | new file |
| 6 | `desktop/tests/test_sidecar.py` | One `test_<id>_extraction_returns_valid_result()` | 40–63 |
| 7 | `desktop/scripts/run_demo.sh` | One `run "<industry>" ... <id>` line | 31–32 |

> **Why four code touch points and not one?** The current registry is
> split because the Pydantic class lives in Python (the sidecar) and
> the schema *names* need to be available to the React picker without
> spawning the sidecar. A future v2.0 collapses this into a single
> `schemas.json` that the sidecar reads at startup and the Rust core
> exposes via an `ipc://schemas` endpoint. The split is documented in
> [`docs/adr/0001-no-hermes-in-desktop-runtime.md`](./0001-no-hermes-in-desktop-runtime.md).

---

## 3. Walkthrough: adding the **Medical SOAP Note** schema

We chose SOAP (Subjective / Objective / Assessment / Plan) as the
worked example because it has different shape from real estate:
free-text fields, no money, no dates in the same sense, and a
discrete medical taxonomy.

### 3.1 Define the Pydantic class

In `desktop/sidecar/realestate_extract/schemas.py`, append:

```python
class MedicalSoapNote(BaseModel):
    """Structured extraction for a clinical SOAP note (1-2 page encounter
    summary). Tested on English primary-care visit notes; no PHI is
    persisted by the engine itself — see [HIPAA note below].
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
```

And register it in `SCHEMA_REGISTRY`:

```python
SCHEMA_REGISTRY["medical_soap_note"] = MedicalSoapNote
```

### 3.2 Mirror it in the Rust IPC layer

In `desktop/src-tauri/src/commands.rs`, add a new tuple to the
`SCHEMAS` const (the order does not matter, but keep the four-tuple
shape):

```rust
const SCHEMAS: &[(&str, &str, &str, &[&str])] = &[
    // ... existing three ...
    (
        "medical_soap_note",
        "MedicalSoapNote",
        "Structured extraction for a clinical SOAP note (Subjective/Objective/Assessment/Plan).",
        &[
            "patient_id", "encounter_date", "chief_complaint", "hpi",
            "past_medical_history", "medications", "allergies",
            "vital_signs", "physical_exam", "assessment", "plan",
            "follow_up", "provider_name",
        ],
    ),
];
```

The field order here is the order the React UI will render them in.

### 3.3 Add the JSON Schema resource

The sidecar reads `desktop/schemas/<id>.json` (a JSON Schema 2020-12
document) so downstream consumers (Excel, your EMR, FHIR servers) can
validate the output. Put it at
`desktop/schemas/medical_soap_note.json`:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://realestate-bench.dev/schemas/medical_soap_note.json",
  "title": "MedicalSoapNote",
  "type": "object",
  "required": ["patient_id", "encounter_date", "chief_complaint",
               "hpi", "physical_exam", "provider_name"],
  "properties": {
    "patient_id":            {"type": "string", "pattern": "^P-[0-9]{6}$"},
    "encounter_date":        {"type": "string", "format": "date"},
    "chief_complaint":       {"type": "string"},
    "hpi":                   {"type": "string"},
    "past_medical_history":  {"type": "string"},
    "medications":           {"type": "string"},
    "allergies":             {"type": "string"},
    "vital_signs":           {"type": ["string", "null"]},
    "physical_exam":         {"type": "string"},
    "assessment":            {"type": "array", "items": {"type": "string"}},
    "plan":                  {"type": "array", "items": {"type": "string"}},
    "follow_up":             {"type": ["string", "null"]},
    "provider_name":         {"type": "string"}
  }
}
```

Then list it in `desktop/src-tauri/tauri.conf.json`:

```json
"resources": [
  "../sidecar/realestate_extract/schemas.py",
  "../schemas/lease_addendum.json",
  "../schemas/invoice.json",
  "../schemas/rent_roll.json",
  "../schemas/medical_soap_note.json"
]
```

### 3.4 Add a sample document

`desktop/tests/sample_soap_note.txt` — a fictional, fully-de-identified
note (no real patient, no real provider, no PHI):

```
ENCOUNTER NOTE — 2026-10-12
Patient: P-104732
Provider: Dr. Anya Sharma, MD (Internal Medicine)

CHIEF COMPLAINT
Patient presents with 4-day history of productive cough and low-grade fever.

HISTORY OF PRESENT ILLNESS
Pt is a 58-year-old male who reports cough productive of yellow sputum
since 2026-10-08, associated with subjective fevers (Tmax 38.3 C),
fatigue, and mild dyspnea on exertion. No chest pain, no hemoptysis.
Denies recent travel. Up to date on influenza vaccine.

PAST MEDICAL HISTORY
Hypertension, hyperlipidemia, type 2 diabetes mellitus (well controlled).

MEDICATIONS
Lisinopril 20 mg daily; Atorvastatin 40 mg nightly; Metformin 1000 mg BID.

ALLERGIES
NKDA (no known drug allergies).

VITAL SIGNS
T 38.1 C, HR 96, BP 142/88, RR 20, SpO2 95% on room air.

PHYSICAL EXAM
Gen: alert, no acute distress. Lungs: rhonchi in right lower lobe,
no wheezes, no rales. CV: RRR, no murmurs. Abdomen: soft, non-tender.

ASSESSMENT
1. Community-acquired pneumonia, right lower lobe, outpatient-eligible.
2. HTN — at goal, continue current regimen.

PLAN
- Prescribe amoxicillin 875 mg PO BID x 5 days.
- Chest X-ray, PA and lateral.
- Recheck in 48-72 hours if not improving.
- Return to clinic in 2 weeks for reassessment.
```

### 3.5 Add the test

In `desktop/tests/test_sidecar.py`:

```python
SAMPLE_SOAP = Path(__file__).parent / "sample_soap_note.txt"

def test_soap_extraction_returns_valid_result():
    out = _run_sidecar(SAMPLE_SOAP, "medical_soap_note")
    assert out["error"] is None, f"sidecar error: {out['error']}"
    assert out["extraction"] is not None
    # Identity must be de-identified
    assert out["extraction"]["patient_id"].startswith("P-")
    # Date must be ISO-8601
    assert out["extraction"]["encounter_date"].count("-") == 2
    # Required free-text fields populated
    assert "cough" in out["extraction"]["chief_complaint"].lower()
    assert "amoxicillin" in out["extraction"]["plan"][0].lower() or \
           any("amoxicillin" in p.lower() for p in out["extraction"]["plan"])
    # Provider name captured
    assert "Sharma" in out["extraction"]["provider_name"]
```

### 3.6 Update the demo

`desktop/scripts/run_demo.sh`:

```bash
run "SOAP-note extraction"  "$HERE/tests/sample_soap_note.txt" medical_soap_note
```

### 3.7 (Optional) Change the default schema in the React picker

`desktop/src/pages/DropPage.tsx`, line 10:

```typescript
const [schemaId, setSchemaId] = useState<string>("lease_addendum");
```

Change to `"medical_soap_note"` if you want the medical schema to open
by default. Leave it alone if you're adding a *second* industry and
the user is still in real estate by default.

### 3.8 Rebuild and run

```bash
cd desktop/
pnpm tauri build          # ~50 s, includes Python re-import of the new schema
open src-tauri/target/release/bundle/macos/RealEstate-Bench.app
```

The schema picker in the UI will now show **MedicalSOAPNote** as a
new option. Pick a PDF, choose the schema, hit Extract — same ~600 ms
latency, JSON output conforms to the new JSON Schema, exports work
unchanged, FTS5 search indexes the new fields automatically.

---

## 4. Five case-study templates

The schemas below are **first-draft references**, not production
deployments. Each is a self-contained Pydantic class, a sample
document, a one-paragraph domain framing, and a "what to measure"
test plan. They follow the same pattern as the SOAP walkthrough in
§3; only the schema and sample vary.

> **Use case strength:** ★ = generic model alone, off-the-shelf
> cactus-needle. ★★ = generic model + schema constraints (the current
> state for all shipped schemas). ★★★ = generic model + a few
> dozen labelled examples per field. ★★★★ = generic model + LoRA
> fine-tune on 500+ examples per schema (see §6).

### 4.1 Legal — *Contract Clause Extractor* (★★)

**Domain framing.** Most legal-tech extraction is "given a 50-page
contract, pull out the clauses I care about." For a commercial lease
(or a SaaS MSA, or an NDA) the user wants structured access to
the things they can't easily find with a text search: the term, the
governing law, the indemnity cap, the auto-renewal clause.

```python
class ContractClause(BaseModel):
    """One material clause from a commercial contract."""
    clause_type:  Annotated[Literal[
        "term", "termination", "auto_renewal", "governing_law",
        "indemnity", "limitation_of_liability", "confidentiality",
        "assignment", "payment_terms", "warranty", "other"
    ], needle.Field(description="Standardized clause type from the controlled vocabulary")]
    clause_text:  Annotated[str, needle.Field(description="Verbatim text of the clause, including section heading if any")]
    page_number:  Annotated[Optional[int], needle.Field(ge=1, le=10_000, description="Page number where the clause appears; null if not paginated")] = None
    dollar_amount: Annotated[Optional[float], needle.Field(ge=0, description="Dollar amount if the clause specifies one (e.g. liability cap); null otherwise")] = None
    party_obligated: Annotated[Optional[str], needle.Field(description="Party that bears the obligation under this clause; null if bilateral")] = None
```

**Sample document:** a 4-page fictional SaaS MSA with one of each
clause type.

**What to measure:**

- Per-clause-type F1: extract the list of clauses from a doc, match
  each to the labelled clause type, score against a golden set of 30
  contracts.
- "Did the indemnity cap get pulled correctly?" — 0/1 accuracy on
  `dollar_amount` when `clause_type == "indemnity"`.
- Latency: target <1.5 s for a 50-page contract on a warm cache.

### 4.2 Healthcare — *Lab Result Extractor* (★★★)

**Domain framing.** Pulling structured values (HbA1c, eGFR, glucose)
out of a clinical lab report is high-value but model-fragile: the
units vary (mg/dL vs mmol/L), the reference ranges change, and a
mis-extraction can be clinically dangerous. This is the highest-value
schema to **fine-tune** (see §6.3).

```python
class LabResult(BaseModel):
    """One analyte value from a clinical lab report."""
    analyte:        Annotated[str, needle.Field(description="Name of the analyte (e.g. 'Hemoglobin A1c', 'eGFR', 'Glucose, fasting')")]
    value:          Annotated[float, needle.Field(description="Numeric value as printed")]
    unit:           Annotated[Literal[
        "mg/dL", "mmol/L", "g/dL", "%", "U/L", "mIU/L", "ng/mL",
        "pg/mL", "IU/L", "mEq/L", "mm/hr", "10^3/uL", "10^6/uL", "other"
    ], needle.Field(description="Unit of measure, standardized; 'other' if uncommon")]
    reference_low:  Annotated[Optional[float], needle.Field(description="Lower bound of reference range; null if absent")] = None
    reference_high: Annotated[Optional[float], needle.Field(description="Upper bound of reference range; null if absent")] = None
    flag:           Annotated[Literal["normal", "low", "high", "critical_low", "critical_high", "abnormal"], needle.Field()] = "normal"
    collected_at:   Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Date the sample was collected")]
```

**Sample document:** a fictional Quest Diagnostics report with 8 analytes.

**What to measure:**

- Unit normalization accuracy: when the doc says "5.7 %" and the schema
  says `unit: "%"`, did the model pick up the percent sign?
- Reference-range match: % of cases where `reference_low` and
  `reference_high` are exactly correct.
- `flag` derivation: % of cases where the model correctly inferred
  `flag: "high"` from `value > reference_high`.

### 4.3 Finance — *Bank Statement Line Item* (★★)

**Domain framing.** Reconciling a downloaded bank statement against
the general ledger is a 30-minute manual task per month per account.
Extract transaction date, amount, counterparty, and a likely
GL-account code so the user only has to verify, not type.

```python
class BankTxn(BaseModel):
    """One line item on a bank statement."""
    posted_date:  Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Date the transaction posted")]
    amount:       Annotated[float, needle.Field(description="Signed amount: positive=credit, negative=debit")]
    currency:     Annotated[Literal["USD", "CAD", "EUR", "GBP", "MXN", "AUD", "INR", "JPY", "CHF", "SGD"], needle.Field()]
    counterparty: Annotated[str, needle.Field(description="Name of the other party as it appears on the statement")]
    memo:         Annotated[str, needle.Field(description="Bank's memo field; empty if blank")]
    txn_type:     Annotated[Literal["ach_debit", "ach_credit", "wire", "check", "fee", "interest", "card", "transfer", "other"], needle.Field()] = "other"
```

**Sample document:** a fictional Chase business checking statement,
3 pages, 24 transactions.

**What to measure:**

- Sign-of-amount accuracy: the model must pick the *signed* amount.
  Most common failure mode is "all positive."
- Counterparty normalization: a real-world bank prints
  "AMZN MKTPL*MK12U3" and the user wants "Amazon Marketplace."
  For ★★, accept the raw string. For ★★★, add a `normalize_vendor`
  hook.
- `txn_type` classification F1 across the 9 categories.

### 4.4 Logistics — *Bill of Lading Field* (★★)

**Domain framing.** Bills of lading (BoLs) are short, structured
shipping documents. The carrier, the shipper, the consignee, the
pro-number, the weight, the piece count, and the freight class
are all there; the value is in typing them into a TMS without
re-typing.

```python
class BillOfLading(BaseModel):
    """One bill of lading (BoL) — the carrier-issued shipping document."""
    bol_number:    Annotated[str, needle.Field(description="Carrier-issued BoL number")]
    pro_number:    Annotated[Optional[str], needle.Field(description="PRO/tracking number; null if same as BoL number")] = None
    shipper:       Annotated[str, needle.Field(description="Shipper name and address")]
    consignee:     Annotated[str, needle.Field(description="Consignee name and address")]
    carrier:       Annotated[str, needle.Field(description="Carrier name (e.g. 'Old Dominion', 'FedEx Freight')")]
    ship_date:     Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="Date the shipment was picked up")]
    weight_lb:     Annotated[float, needle.Field(ge=0, le=200_000, description="Total weight in pounds")]
    pieces:        Annotated[int, needle.Field(ge=1, le=100_000, description="Number of pieces in the shipment")]
    freight_class: Annotated[Optional[Literal["50","55","60","65","70","77.5","85","92.5","100","110","125","150","175","200","250","300","400","500"], needle.Field()]=None
    nmfc_code:     Annotated[Optional[str], needle.Field(description="NMFC code if printed; null otherwise")] = None
```

**Sample document:** a fictional XPO BoL with all 11 fields populated.

**What to measure:**

- 11-field recall: how many fields are non-null AND correct?
- `freight_class` enumeration: 18 valid values; the model must pick
  one or null.
- `weight_lb` extraction: most likely to be wrong if the doc has
  both metric and imperial on the same page.

### 4.5 HR / Recruiting — *Resume Contact Block* (★)

**Domain framing.** Pulling name, email, phone, current title, and
current employer out of a resume PDF. This is a *low-difficulty*
schema: the model is mostly transcription, the fields are highly
typed, and a wrong answer is annoying but not dangerous.

```python
class ResumeContactBlock(BaseModel):
    """The top-of-resume contact block (header section)."""
    full_name:    Annotated[str, needle.Field(description="Candidate's full name as printed")]
    email:        Annotated[Optional[str], needle.Field(pattern=r"^[^@\s]+@[^@\s]+\.[^@\s]+$", description="Primary email; null if not present")] = None
    phone:        Annotated[Optional[str], needle.Field(description="Phone in the format printed, e.g. '+1 415 555 0123'; null if absent")] = None
    location:     Annotated[Optional[str], needle.Field(description="City, State; null if absent")] = None
    linkedin:     Annotated[Optional[str], needle.Field(description="LinkedIn URL; null if absent")] = None
    current_title: Annotated[Optional[str], needle.Field(description="Most recent job title from the resume; null if not determinable")] = None
    current_employer: Annotated[Optional[str], needle.Field(description="Most recent employer from the resume; null if not determinable")] = None
```

**Sample document:** a fictional 1-page resume for a fictional
candidate.

**What to measure:**

- Email regex compliance: 100% — the schema enforces `pattern`.
- Phone format: 95% — model occasionally drops the country code.
- `current_title` / `current_employer` recall: the model has to
  reason about recency. Target 0.85+ on a 50-resume golden set.

> **PII note:** This schema extracts contact info, which is PII
> under GDPR/CCPA. The engine itself does not phone home, but the
> extracted JSON lives in `~/Library/Application Support/.../app.db`.
> For production HR use, add a `redact_pii: true` config flag that
> masks `email`, `phone`, and `linkedin` before writing to disk. The
> hook is in `commands.rs::record_extraction` — three lines to add.

---

## 5. Case-study test harness

For each case study above, the test plan is the same:

1. **Generate a synthetic dataset.** 30-50 fictional documents, each
   hand-written to cover edge cases of the schema. The
   `datasets/01_invoice_extract_benchmark/generator/` package shows
   how to do this with controllable layout/handwriting/degradation
   knobs; port the pattern to your industry.
2. **Hand-label the golden JSON** for each document.
3. **Add a pytest** that runs the sidecar on every doc and computes:
   - **Per-field F1** (presence + value equality)
   - **Per-field accuracy** when the field is present
   - **Confidence calibration**: bin predictions by confidence
     decile, measure actual F1 in each bin. The model is well-calibrated
     if F1 rises with confidence.
4. **Run the regression** as part of `cargo test` and `pnpm vitest`.
   Block the PR if any schema drops below 0.80 per-field F1.
5. **Add the schema to `bench/golden/v1/`** so future fine-tunes
   measure against the same set.

A worked example of (1)-(3) is the existing
`datasets/01_invoice_extract_benchmark/` package; the test
harness lives in `desktop/tests/test_sidecar.py`.

---

## 6. Industry fine-tuning playbook

The shipped cactus-needle 3.2.1 model is **generic**. For the ★
schemas in §4 (Resume, simple Invoice) it works out of the box.
For ★★ it works with careful schema design. For ★★★ (lab results,
emails) you need a small amount of labelled data. For ★★★★ (clinical
extraction, finance) you need a real fine-tune.

### 6.1 When to fine-tune

| Symptom | Action |
|---------|--------|
| Per-field F1 ≥ 0.90, calibration good | Don't fine-tune. Ship as-is. |
| Per-field F1 0.75–0.90, calibration OK | Add 50-200 labelled examples per field and re-evaluate. Often enough to push to 0.90+ without a fine-tune. |
| Per-field F1 < 0.75 OR systematic misses on a specific field | Fine-tune that field's pattern + add 200+ examples. |
| Model is *confident* (≥0.7 mean) but *wrong* (F1 < 0.6) | Hallucination. Fine-tune. |
| Model is *uncertain* (mean < 0.5) and *right* | Calibration issue, not accuracy. Just lower the auto-accept threshold. |

### 6.2 How to build a labelled set

1. **Generate 200+ synthetic documents** using the
   `datasets/01_invoice_extract_benchmark/generator/` pattern: a
   controlled-vocabulary spec + a layout renderer + a degradation
   pass (rotate, scan, blur, smudge).
2. **Hand-label the JSON** for each. This is the
   most-expensive step; budget 30-60 s per document.
3. **Split 80/10/10** into train/dev/test. The dev set is for
   picking the fine-tune hyper-params; the test set is **never seen
   during training** and is the regression baseline.
4. **Save in `bench/golden/v1/<industry>/`** as
   `0001_invoice.pdf` + `0001_invoice.json` pairs.

### 6.3 The fine-tune recipe (cactus-needle LoRA)

```python
# Pseudocode — the actual script lives at
# desktop/scripts/finetune_industry.py once we ship it.
from cactus_needle import NeedleTrainer
from datasets import load_dataset

ds = load_dataset("bench/golden/v1/lab_results/", split="train")
trainer = NeedleTrainer(
    base_model="cactus-needle-3.2.1",
    schema=LabResult,                  # the Pydantic class
    train_data=ds,
    lora_r=16,
    lora_alpha=32,
    learning_rate=2e-4,
    epochs=3,
    batch_size=8,
    warmup_ratio=0.05,
)
trainer.train()
trainer.save("cactus-needle-lab-v0.1.0/")
```

**Resource cost (estimated, on a single A100 40GB):**

- 200 examples, 3 epochs: ~15 min
- 1000 examples, 3 epochs: ~75 min
- 5000 examples, 3 epochs: ~6 hours

The output is a single GGUF file that the sidecar can load instead
of the base model. The CLI flag is `sidecar/bin/realestate-extract --model
./cactus-needle-lab-v0.1.0.gguf`. The Tauri app picks it up via the
`REALESTATE_EXTRACT_MODEL` env var.

### 6.4 When **not** to fine-tune

- The schema changes frequently. A fine-tuned model loses accuracy
  on a schema it wasn't trained on. Keep schemas versioned
  (`schema_version: "1.2.0"`) and re-fine-tune per major version.
- You have <100 labelled examples. You'll over-fit. Use the
  generic model with tighter schema constraints (more `pattern=`
  and `Literal[]` enums) instead.
- The downstream consumer is OK with a 0.70 F1. Don't fine-tune
  past what the user will use.

---

## 7. What stays the same

When you add an industry, **none of the following change**:

- The Rust core (`commands.rs`, `db.rs`, `extract.rs`)
- The SQLite schema (documents, extractions, fts_extractions, edits)
- The Excel / JSON / CSV exporters
- The FTS5 search index
- The Tauri capabilities and CSP
- The bundle size (still 6.1 MiB; the model is the same)
- The "no network at runtime" guarantee

What you add is **one Pydantic class, one JSON Schema file, one
sample document, one test, and optionally one fine-tune**. That's
the contract this engine offers.

---

## 8. Checklist for shipping a new industry pack

```text
[ ] Pydantic class added to desktop/sidecar/realestate_extract/schemas.py
[ ] Pydantic class registered in SCHEMA_REGISTRY
[ ] Rust tuple added to desktop/src-tauri/src/commands.rs (SCHEMAS const)
[ ] JSON Schema file added to desktop/schemas/<id>.json
[ ] JSON Schema path added to desktop/src-tauri/tauri.conf.json (resources)
[ ] Sample document added to desktop/tests/sample_<id>.txt
[ ] pytest added to desktop/tests/test_sidecar.py
[ ] run_demo.sh updated to include the new schema
[ ] (Optional) Default schemaId in DropPage.tsx updated
[ ] (Optional) Fine-tune recipe run; model saved to sidecar/models/<id>.gguf
[ ] (Optional) bench/golden/v1/<id>/ created with 30+ hand-labelled examples
[ ] BACKLOG.md: new US-7xx story under Phase 1.5 for the industry
[ ] README.md: industry mentioned in the desktop app section
[ ] CHANGELOG.md: "Added <industry> schema pack"
[ ] Committed and pushed
```

That's the whole extension surface. If anything in §2 or §3 is
unclear, that's a doc bug — file it.

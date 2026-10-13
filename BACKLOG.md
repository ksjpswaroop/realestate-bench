# RealEstate-Bench — Sprint Backlog

User stories organized by phase. Story IDs are stable: US-001..099 = Phase 0, US-101..199 = Phase 1, etc.

## Conventions

### Story points (Fibonacci)
- **1pt:** trivial config, typo, cosmetic
- **2pt:** small component, clear path
- **3pt:** typical feature with edge cases
- **5pt:** multi-component feature, needs design
- **8pt:** cross-cutting, needs spike
- **13pt:** too big — split

### Status icons
- ⬜ pending
- 🚧 in progress
- ✅ done
- 🔒 blocked on external
- ⏸ deferred

### Priority labels
- **P0** — Must ship for phase gate
- **P1** — Should ship, doesn't gate
- **P2** — Could ship, no commitment

### Story format
```
### US-XXX [status] [P0/P1/P2] — Title [Npt]

**As a** [role]
**I want to** [action]
**So that** [outcome]

**Acceptance criteria:**
- [ ] Criterion 1 (specific, testable)
- [ ] Criterion 2

**Tasks (≥5pt only):**
- Sub-task 1
- Sub-task 2

**Cross-refs:**
- FR-XXX in PRD
- ARCH §Y
```

---

## Phase 0 — Model Selection (October 2026, 1 sprint)

**Goal:** Choose the model with evidence. Gate = per-field F1 ≥ 0.80 on drew-ipp invoice-extraction-benchmark.

### US-001 [✅] [P0] — Pull benchmark datasets [3pt]
**As a** founder
**I want to** all 7 benchmark datasets in `datasets/`
**So that** I can run the Phase 0 eval suite

**Acceptance criteria:**
- [x] ExtractBench (786 MB), SROIE (508 MB), CORD (2.1 GB), FUNSD (16 MB), drew-ipp invoice benchmark (38 MB), invoice-extraction-v2 (60 KB)
- [x] NuExtract GitHub repo cloned
- [x] `datasets/README.md` with per-dataset stats + license

**Cross-refs:** ARCH §2.2 ingest, PRD §3 market analysis

### US-002 [⬜] [P0] — Define benchmark schemas [2pt]
**As a** developer
**I want to** two Pydantic v2 schemas: `BenchmarkInvoice` and `BenchmarkReceipt`
**So that** every model is scored against the same targets

**Acceptance criteria:**
- [ ] `bench/schema.py` exports `BenchmarkInvoice` and `BenchmarkReceipt`
- [ ] Both schemas have field descriptions matching the dataset ground truth
- [ ] Schema versioning via `__version__ = "1.0.0"`
- [ ] Test: importable, instantiable, serializable

### US-003 [⬜] [P0] — Build dataset loaders [5pt]
**As a** developer
**I want to** loaders that return `(doc_text, schema, ground_truth)` tuples
**So that** the runner has uniform input

**Acceptance criteria:**
- [ ] `bench/loaders.py` exports one loader per dataset
- [ ] Loaders handle: ExtractBench (jsonl), SROIE (parquet), CORD (parquet), FUNSD (parquet), invoice-extraction-benchmark (PDF + JSON), invoice-extraction-v2 (jsonl)
- [ ] Each loader yields `Iterable[BenchmarkSample]`
- [ ] Test: smoke test loads 10 samples per dataset

**Tasks:**
- Loader for ExtractBench short split (252 docs)
- Loader for drew-ipp corpus (181 docs)
- Loader for SROIE/CORD/FUNSD (parquet → text)
- Loader for invoice-extraction-v2 (jsonl)

### US-004 [⬜] [P0] — Build the eval runner [5pt]
**As a** developer
**I want to** `bench/run.py --model <id> --dataset <name>` that produces predictions.jsonl
**So that** every model produces a comparable artifact

**Acceptance criteria:**
- [ ] Loads model via `realestate_bench.backends.registry`
- [ ] Iterates dataset; calls model; writes predictions.jsonl
- [ ] Per-prediction row: `{doc_id, model, latency_ms, raw_output, parsed_json, parse_ok}`
- [ ] Handles model timeouts (skip, log, continue)
- [ ] Test: mock backend, 3 docs, predictions.jsonl matches expected

### US-005 [⬜] [P0] — Build the scorer [5pt]
**As a** developer
**I want to** `bench/score.py` that computes per-field F1
**So that** I can rank models

**Acceptance criteria:**
- [ ] Per-field F1: exact-match for primitives, Levenshtein for strings, IoU for arrays
- [ ] Aggregate: mean per-field F1 per dataset
- [ ] Output: JSON with per-field, per-dataset, overall scores
- [ ] Test: known inputs → known scores

### US-006 [⬜] [P0] — Wire NuExtract3 backend [5pt]
**As a** developer
**I want to** NuExtract3 callable from the runner
**So that** we can score it

**Acceptance criteria:**
- [ ] `backends/vllm.py` loads `numindai/NuExtract3-4B`
- [ ] JSON-mode output enforced
- [ ] Test: smoke run on 1 doc, see scores

### US-007 [⬜] [P0] — Wire ReaderLM-v2 backend [3pt]
**As a** developer
**I want to** ReaderLM-v2 callable from the runner
**So that** we can score it

**Acceptance criteria:**
- [ ] `backends/vllm.py` also supports `jinaai/ReaderLM-v2`
- [ ] Test: smoke run on 1 doc

### US-008 [⬜] [P0] — Wire GLiNER2 backend [3pt]
**As a** developer
**I want to** GLiNER2 callable (entity subset only)
**So that** we can score it as a fallback

**Acceptance criteria:**
- [ ] `backends/gliner2.py` for entity tagging
- [ ] Scorer adapts: only scores those fields
- [ ] Test: smoke run

### US-009 [⬜] [P0] — Run eval on ExtractBench short [3pt]
**As a** founder
**I want to** results in `bench/results/<date>/extractbench.json`
**So that** I can compare models

**Acceptance criteria:**
- [ ] Run all 3-4 models on ExtractBench short (252 docs)
- [ ] Wall time < 2 hours on 1× A100
- [ ] Report: per-field F1 per model + comparison table

### US-010 [⬜] [P0] — Run eval on invoice benchmark [3pt]
**As a** founder
**I want to** results on drew-ipp invoice benchmark
**So that** I know which model wins on real invoices

**Acceptance criteria:**
- [ ] Run all models on 181 docs
- [ ] Wall time < 1 hour
- [ ] Report: per-field F1, comparison with IPP's 96.1% as ceiling

### US-011 [⬜] [P0] — Pick the model + log decision [2pt]
**As a** founder
**I want to** `MODEL_PICK.md` with the winner + 1-paragraph why
**So that** future work has a clear target

**Acceptance criteria:**
- [ ] Weighted score: `0.5 × ExtractBench + 0.3 × invoice + 0.2 × real_estate`
- [ ] Gate: invoice benchmark F1 ≥ 0.80
- [ ] Tie-breakers: cost, latency, model size
- [ ] Logged with the run that produced it

**Phase 0 Gate:** US-001..US-011 all complete; decision logged.

---

## Phase 1 — CLI MVP (November 2026, 2 sprints)

**Goal:** Working CLI that extracts a Lease Addendum with ≥ 0.85 F1 on a 30-doc golden set.

### Scaffolding (US-101 to US-105)

### US-101 [⬜] [P0] — Project scaffolding [2pt]
**As a** developer
**I want to** `pyproject.toml` with `hatchling` + pinned deps
**So that** the package is installable

**Acceptance criteria:**
- [ ] `pip install realestate-bench` works from local path
- [ ] `realestate-bench --help` works
- [ ] `realestate-bench --version` shows 0.1.0

### US-102 [⬜] [P0] — Typer CLI skeleton [3pt]
**As a** developer
**I want to** Typer app with stub commands
**So that** commands are typed and `--help` works

**Acceptance criteria:**
- [ ] `init`, `extract`, `batch`, `test`, `schemas`, `cache`, `config` registered
- [ ] Each command has docstring + arg types
- [ ] Test: `--help` for each command

### US-103 [⬜] [P0] — Logging setup [2pt]
**As a** developer
**I want to** structured logging to `~/.local/share/realestate-bench/logs/`
**So that** every action is traceable

**Acceptance criteria:**
- [ ] `cli/logging.py` configures rotating handler (7 days)
- [ ] `--log-format json|text` toggle
- [ ] Test: log file created, rotation works

**Cross-refs:** FR-009 (Logging + observability)

### US-104 [⬜] [P0] — Config management [2pt]
**As a** developer
**I want to** YAML config at `~/.config/realestate-bench/config.yaml`
**So that** users can set defaults

**Acceptance criteria:**
- [ ] `config get <key>` / `config set <key> <value>`
- [ ] Schema validation for config values
- [ ] Test: set + get roundtrip

### US-105 [⬜] [P0] — State DB + migrations [3pt]
**As a** developer
**I want to** SQLite state DB with `001_initial.sql`
**So that** we persist extractions and cache

**Acceptance criteria:**
- [ ] `state/db.py` opens DB, runs migrations on `init`
- [ ] Tables: documents, schemas, extractions, cache, golden_docs, golden_results
- [ ] `PRAGMA user_version` migration tracking
- [ ] Test: in-memory DB + migration roundtrip

### Ingestion (US-110 to US-115)

### US-110 [⬜] [P0] — MIME detection [1pt]
**As a** developer
**I want to** detect MIME from extension + magic bytes
**So that** FR-001 is satisfied

**Acceptance criteria:**
- [ ] `ingest/detect.py` returns MIME + confidence
- [ ] Supports: PDF, JPEG, PNG, TIFF, HEIC
- [ ] Test: known fixtures

**Cross-refs:** FR-001 (Document ingestion)

### US-111 [⬜] [P0] — PDF text extraction [3pt]
**As a** developer
**I want to** extract text via `pypdf` (safe reader)
**So that** digital PDFs are fast

**Acceptance criteria:**
- [ ] `ingest/pdf.py` returns pages + full text
- [ ] Handles multi-page PDFs
- [ ] Rejects encrypted PDFs (PASSWORD_REQUIRED)
- [ ] Test: real PDF fixture

**Cross-refs:** FR-001 (Document ingestion)

### US-112 [⬜] [P0] — Image normalization [3pt]
**As a** developer
**I want to** Pillow-based normalization for photos
**So that** JPEG/PNG inputs work

**Acceptance criteria:**
- [ ] `ingest/image.py` returns text via PIL + (Phase 2) OCR
- [ ] Deskew + denoise for photos
- [ ] Test: image fixture

### US-113 [⬜] [P0] — Streaming hash [1pt]
**As a** developer
**I want to** SHA-256 streaming hash
**So that** large files don't OOM

**Acceptance criteria:**
- [ ] `ingest/hash.py` returns sha256 of file
- [ ] Test: known file → known hash

### US-114 [⬜] [P1] — HEIC support [2pt]
**As a** user with iPhone photos
**I want to** HEIC inputs to work
**So that** I don't have to convert

**Acceptance criteria:**
- [ ] Auto-convert HEIC → JPEG via Pillow-HEIF
- [ ] Optional dep: `[heic]` extra

### US-115 [⬜] [P1] — Image OCR fallback [5pt]
**As a** user with scanned photos
**I want to** OCR to work
**So that** scanned PDFs and photos are usable

**Acceptance criteria:**
- [ ] `ingest/ocr.py` uses pytesseract
- [ ] Triggered when no text layer
- [ ] Optional dep: `[ocr]` extra

### Schema (US-120 to US-126)

### US-120 [⬜] [P0] — Schema registry [3pt]
**As a** developer
**I want to** `schemas/registry.py` that loads built-in + user schemas
**So that** `realestate-bench extract --schema <id>` works

**Acceptance criteria:**
- [ ] Built-in schemas: lease_addendum, rent_roll, vendor_invoice, lender_statement
- [ ] User schemas from `~/.config/realestate-bench/schemas/`
- [ ] Schema versioning tracked
- [ ] Test: list + load

**Cross-refs:** FR-002 (Schema declaration)

### US-121 [⬜] [P0] — LeaseAddendum schema [3pt]
**As a** user
**I want to** extract lease addenda
**So that** I can do rent comps

**Acceptance criteria:**
- [ ] `schemas/built_in/lease_addendum.py` with 12+ fields
- [ ] Fields: lease_id, tenant, premises, effective_date, expiration_date, monthly_rent, deposit, escalation, renewal_option, signatures, etc.
- [ ] Each field has `description=` for prompt inclusion
- [ ] Test: schema instantiable, serializable

**Cross-refs:** FR-002 (Schema declaration), FR-003 (Extraction execution)

### US-122 [⬜] [P0] — VendorInvoice schema [3pt]
**As a** user
**I want to** extract vendor invoices
**So that** I can paste into AppFolio

**Acceptance criteria:**
- [ ] Fields: invoice_no, vendor, date, due_date, line_items[], subtotal, tax, total, currency
- [ ] Each field documented
- [ ] Test: sample invoice → instantiable

### US-123 [⬜] [P0] — Prompt template for LeaseAddendum [3pt]
**As a** developer
**I want to** `prompts/lease_addendum.prompt.md`
**So that** the model gets clear instructions

**Acceptance criteria:**
- [ ] Template includes schema JSON + field descriptions + output format example
- [ ] Token count < 2K for typical schema
- [ ] Test: prompt generation produces expected structure

### US-124 [⬜] [P1] — RentRoll schema [3pt]
**As a** user
**I want to** extract rent rolls
**So that** I can underwrite deals

**Acceptance criteria:**
- [ ] Fields: unit_id, sqft, rent, lease_start, lease_end, escalation
- [ ] List-of-rows shape
- [ ] Test: sample rent roll

### US-125 [⬜] [P1] — LenderStatement schema [3pt]
**As a** user
**I want to** extract lender statements
**So that** I can do comps

**Acceptance criteria:**
- [ ] Fields: lender, statement_date, balance, transactions[]
- [ ] Test: sample statement

### US-126 [⬜] [P1] — Schema introspection commands [2pt]
**As a** user
**I want to** `schemas list` and `schemas show <id>`
**So that** I can discover what's available

**Acceptance criteria:**
- [ ] `list` shows all built-in + user schemas
- [ ] `show <id>` dumps the schema + per-field docs
- [ ] Test: list returns known schemas

**Cross-refs:** FR-010 (Schema introspection)

### Extraction (US-130 to US-145)

### US-130 [⬜] [P0] — Extraction runner [5pt]
**As a** developer
**I want to** `extract/runner.py` that orchestrates the pipeline
**So that** `extract` command works

**Acceptance criteria:**
- [ ] Pipeline: load schema → build prompt → call backend → parse → validate → score → persist
- [ ] Returns structured error codes on failure
- [ ] Test: end-to-end with mock backend

**Tasks:**
- Pipeline orchestrator
- Error code mapping
- Per-stage timing instrumentation

**Cross-refs:** FR-003 (Extraction execution)

### US-131 [⬜] [P0] — Prompt builder [2pt]
**As a** developer
**I want to** schema → prompt template
**So that** the model gets the right context

**Acceptance criteria:**
- [ ] Includes schema field descriptions
- [ ] Includes doc text (chunked if > 4K tokens)
- [ ] Includes output format example (JSON)

**Cross-refs:** FR-003 (Extraction execution)

### US-132 [⬜] [P0] — JSON parser [3pt]
**As a** developer
**I want to** robust JSON extraction from model output
**So that** parse failures are rare

**Acceptance criteria:**
- [ ] Handles JSON in code fences
- [ ] Handles JSON without fences
- [ ] Returns clear error on parse failure
- [ ] Test: 10 known model outputs

### US-133 [⬜] [P0] — Pydantic validation [2pt]
**As a** developer
**I want to** Pydantic v2 validation
**So that** type errors are caught early

**Acceptance criteria:**
- [ ] Construct Pydantic instance from parsed JSON
- [ ] Catch ValidationError → return VALIDATION_FAILED
- [ ] Test: invalid input → clear error

### US-134 [⬜] [P0] — Confidence scorer [5pt]
**As a** user
**I want to** per-field confidence
**So that** I can route low-confidence fields to review

**Acceptance criteria:**
- [ ] Per-field score [0.0, 1.0]
- [ ] Sources: model logits (when available), heuristic (schema match + business-rule agreement), ensemble agreement
- [ ] Test: known input → known scores

**Tasks:**
- Logit-based confidence (model backend reports)
- Heuristic confidence (fallback)
- Ensemble confidence (Phase 2)

### US-140 [⬜] [P0] — Totals math validator [2pt]
**As a** user
**I want to** catch arithmetic errors
**So that** I don't have to manually verify

**Acceptance criteria:**
- [ ] `validators/totals_math.py`: line items sum = subtotal
- [ ] PASS/FAIL/WARN result
- [ ] Test: known inputs

**Cross-refs:** FR-004 (Business-rule validation)

### US-141 [⬜] [P0] — Date sanity validator [2pt]
**As a** user
**I want to** catch date errors
**So that** I don't have to manually verify

**Acceptance criteria:**
- [ ] `validators/date_sanity.py`: effective < expiration, no future dates on old docs
- [ ] Test: known inputs

**Cross-refs:** FR-004 (Business-rule validation)

### US-142 [⬜] [P0] — Currency normalize validator [2pt]
**As a** user
**I want to** consistent currency codes
**So that** mixed currencies don't sneak through

**Acceptance criteria:**
- [ ] `validators/currency_normalize.py`: ISO 4217 codes
- [ ] Test: known inputs

**Cross-refs:** FR-004 (Business-rule validation)

### US-143 [⬜] [P0] — Confidence routing [2pt]
**As a** user
**I want to** low-confidence fields queued for review
**So that** I focus attention where it's needed

**Acceptance criteria:**
- [ ] `--confidence-threshold 0.90`
- [ ] Items below threshold appended to `--review-queue`
- [ ] Test: threshold + queue behavior

### US-144 [⬜] [P0] — `extract` command [5pt]
**As a** user
**I want to** `realestate-bench extract --doc X --schema Y --out Z`
**So that** I can extract from the CLI

**Acceptance criteria:**
- [ ] All FR-001 to FR-005 acceptance criteria wired through CLI
- [ ] `--no-cache`, `--timeout`, `--confidence-threshold`, `--log-format`
- [ ] Exit codes: 0 success, 1 failure
- [ ] Test: smoke run on golden set

### US-145 [⬜] [P0] — `init` first-run wizard [3pt]
**As a** user
**I want to** `realestate-bench init` walks me through setup
**So that** I can get started in 5 minutes

**Acceptance criteria:**
- [ ] Welcome screen
- [ ] Model selection (default = Phase 0 winner)
- [ ] Downloads weights on first run
- [ ] Runs example doc
- [ ] Ready state

### Golden set (US-150 to US-174)

### US-150 [⬜] [P0] — Lease Addendum golden set, 30 docs [8pt]
**As a** founder
**I want to** 30 hand-labeled lease addenda
**So that** I can regression-test

**Acceptance criteria:**
- [ ] `bench/golden/v1/leases/` with 30 (doc, ground_truth.json) pairs
- [ ] All docs redacted (PII removed)
- [ ] Doc source: public examples + (later) partner-provided
- [ ] Labeled by ksjswaroop with a labeled_by convention

**Cross-refs:** FR-008 (Golden-set regression testing)

### US-151 [⬜] [P0] — Golden test runner [5pt]
**As a** developer
**I want to** `realestate-bench test --golden ...`
**So that** CI can gate on it

**Acceptance criteria:**
- [ ] Loads golden set; runs extract on each
- [ ] Scores per-field F1
- [ ] Compares to baseline (regression > 2% → fail)
- [ ] Writes report to bench/results/<date>/

**Tasks:**
- Loader for golden docs
- F1 computation per field
- Baseline comparison
- Report writer

**Cross-refs:** FR-008 (Golden-set regression testing)

### US-152 [⬜] [P0] — CI integration [3pt]
**As a** developer
**I want to** GitHub Actions running the golden test
**So that** PRs can't regress

**Acceptance criteria:**
- [ ] `.github/workflows/ci.yml` runs on push
- [ ] Lint + unit + golden test
- [ ] PR comment with F1 numbers
- [ ] Test: workflow syntax valid

**Cross-refs:** FR-008 (Golden-set regression testing), FR-009 (Logging + observability)

### US-170 [⬜] [P0] — End-to-end smoke test [3pt]
**As a** developer
**I want to** integration test that runs full pipeline on 3 known docs
**So that** the wiring is always verified

**Acceptance criteria:**
- [ ] `tests/integration/test_extract.py` with 3 (doc, expected_json) pairs
- [ ] Uses mock model backend (canned JSON)
- [ ] Test: pipeline produces expected output

### US-171 [⬜] [P0] — `test` command [3pt]
**As a** developer
**I want to** `realestate-bench test --golden ...`
**So that** I can run golden tests from CLI

**Acceptance criteria:**
- [ ] All US-151 logic accessible via CLI
- [ ] Markdown report printed
- [ ] Exit 0 on pass, 1 on fail

### US-172 [⬜] [P1] — Baseline file [2pt]
**As a** developer
**I want to** `bench/baseline.json` with the Phase 0 winner's scores
**So that** regression has a reference

**Acceptance criteria:**
- [ ] Per-field F1 per schema from Phase 0 winner
- [ ] Committed to repo

### US-173 [⬜] [P2] — Multi-seed run [3pt]
**As a** developer
**I want to** run golden set 3 times with different seeds
**So that** we know the variance

**Acceptance criteria:**
- [ ] `--runs 3` averages results
- [ ] Reports mean + std

### US-174 [⬜] [P2] — Public benchmark submission [3pt]
**As a** founder
**I want to** submit our model to ExtractBench leaderboard
**So that** we gain credibility

**Acceptance criteria:**
- [ ] Follow ExtractBench submission instructions
- [ ] Record our position
- [ ] Reference in README

### Packaging (US-180 to US-195)

### US-180 [⬜] [P1] — PyPI publish [3pt]
**As a** developer
**I want to** publish to PyPI on tag
**So that** `pip install realestate-bench` works

**Acceptance criteria:**
- [ ] `pyproject.toml` complete
- [ ] GitHub Action publishes on tag
- [ ] Test: install from TestPyPI works

### US-181 [⬜] [P1] — Docker image [5pt]
**As a** ops person
**I want to** a docker image
**So that** CI + Phase 3 hosted can use it

**Acceptance criteria:**
- [ ] `Dockerfile` builds
- [ ] Image runs CLI
- [ ] Image < 4 GB (excl. model weights)

### US-182 [⬜] [P2] — Pre-commit hooks [2pt]
**As a** developer
**I want to** pre-commit ruff + black
**So that** style is enforced

**Acceptance criteria:**
- [ ] `.pre-commit-config.yaml`
- [ ] Hooks run on commit

### US-190 [⬜] [P1] — README + quickstart [3pt]
**As a** user
**I want to** a 5-minute quickstart in README
**So that** I can try the tool

**Acceptance criteria:**
- [ ] Quickstart: install, init, extract, see result
- [ ] GIF or screenshot of CLI
- [ ] Link to PRD/Architecture

### US-191 [⬜] [P1] — CONTRIBUTING.md [2pt]
**As a** contributor
**I want to** contribution guide
**So that** I know how to help

**Acceptance criteria:**
- [ ] Setup, run tests, submit PR

### US-192 [⬜] [P1] — LICENSE [1pt]
**As a** user
**I want to** a license file
**So that** I know what I can do

**Acceptance criteria:**
- [ ] Apache-2.0 text
- [ ] Datasets: CC-BY-4.0 attribution

### US-195 [⬜] [P1] — GitHub repo creation [2pt]
**As a** founder
**I want to** `ksjswaroop/realestate-bench` public repo
**So that** the world can see the work

**Acceptance criteria:**
- [ ] Repo created with this initial commit
- [ ] Topics: document-ai, extraction, real-estate, llm
- [ ] Description: "Honest document extraction for real estate"

**Phase 1 Gate:** US-101..US-195 all complete (P0 only); per-field F1 ≥ 0.85 on lease_addendum golden set; `pip install realestate-bench` works.

---

## Phase 2 — Reliability + second schema (Dec 2026 → Jan 2027, 3 sprints)

### US-200 [⬜] [P0] — Rent Roll 30-doc golden set [5pt]
**As a** founder
**I want to** 30 hand-labeled rent rolls
**So that** we have a second schema with regression coverage

**Acceptance criteria:**
- [ ] `bench/golden/v1/rent_rolls/` with 30 (doc, ground_truth.json) pairs

### US-201 [⬜] [P0] — Combined 60-doc golden set gate [3pt]
**As a** developer
**I want to** golden test runs both schemas
**So that** CI catches regressions in either

**Acceptance criteria:**
- [ ] `--golden bench/golden/v1/` runs all 60 docs
- [ ] Per-schema baseline in `bench/baseline.json`

### US-210 [⬜] [P0] — Confidence calibration (AUGRC) [5pt]
**As a** developer
**I want to** confidence scores are calibrated
**So that** users can trust the routing

**Acceptance criteria:**
- [ ] `bench/calibrate.py` computes AUGRC
- [ ] Targets: AUGRC < 0.05 deviation from ideal
- [ ] Logged in benchmark results

### US-211 [⬜] [P0] — Confidence threshold tuning [3pt]
**As a** founder
**I want to** find the best default threshold
**So that** auto-accept rate ≥ 80% with < 5% errors

**Acceptance criteria:**
- [ ] Sweep thresholds 0.80, 0.85, 0.90, 0.95
- [ ] Plot precision-recall vs threshold
- [ ] Default = best trade-off

### US-220 [⬜] [P0] — `batch` command [5pt]
**As a** user
**I want to** `realestate-bench batch --docs GLOB --schema ID --out DIR --concurrency N`
**So that** I can bulk-extract

**Acceptance criteria:**
- [ ] All FR-007 acceptance criteria wired through CLI
- [ ] Progress bar with ETA
- [ ] Per-doc results + summary
- [ ] Test: smoke run on 10 docs

### US-221 [⬜] [P1] — Review queue CLI [5pt]
**As a** user
**I want to** `review list` and `review resolve <id>`
**So that** I can process the review queue from CLI

**Acceptance criteria:**
- [ ] `list` shows pending items
- [ ] `resolve <id> --correction <json>` saves correction + marks resolved
- [ ] Corrections feed back into golden set (with consent)

### US-230 [⬜] [P0] — OCR fallback ensemble [8pt]
**As a** user with scanned docs
**I want to** OCR works robustly
**So that** scanned leases are extractable

**Acceptance criteria:**
- [ ] `ingest/ocr.py` uses Tesseract 5 + PaddleOCR; vote on confidence
- [ ] Triggered when PDF has no text layer
- [ ] Test: 3 scanned PDFs

**Tasks:**
- Tesseract wrapper
- PaddleOCR wrapper
- Voting logic

### US-231 [⬜] [P1] — PaddleOCR backend [3pt]
**As a** developer
**I want to** PaddleOCR wrapped
**So that** we have a second OCR option

**Acceptance criteria:**
- [ ] `ingest/ocr.py` supports paddleocr
- [ ] Optional dep: `[ocr]` extra

### US-240 [⬜] [P0] — Caching + idempotency [5pt]
**As a** developer
**I want to** results cached by (doc, schema) hash
**So that** repeat runs are cheap

**Acceptance criteria:**
- [ ] All FR-006 acceptance criteria wired
- [ ] Cache key = sha256(doc) + sha256(schema)
- [ ] Cache hit < 100ms
- [ ] Test: second run returns cached

### US-241 [⬜] [P1] — Cache management [2pt]
**As a** user
**I want to** `cache list` and `cache clear`
**So that** I can manage my cache

**Acceptance criteria:**
- [ ] `cache list` shows all cache entries
- [ ] `cache clear` empties the cache

### US-250 [⬜] [P1] — Structured logging (JSON) [3pt]
**As a** developer
**I want to** `--log-format json` for ingestion pipelines
**So that** logs are machine-parseable

**Acceptance criteria:**
- [ ] JSON output with stable keys
- [ ] Per-extraction row with trace fields
- [ ] Test: parse log file as JSON

### US-260 [⬜] [P1] — Performance benchmarks [3pt]
**As a** developer
**I want to** latency + throughput benchmarks
**So that** we know we hit performance targets

**Acceptance criteria:**
- [ ] `bench/perf/` measures single-doc + batch
- [ ] Reports p50/p95/p99 latency
- [ ] Comparison vs targets in ARCH §10

**Phase 2 Gate:** US-200..US-260 all complete (P0 only); per-field F1 ≥ 0.90 on combined 60-doc set; confidence AUGRC < 0.05.

---

## Phase 3 — Hosted API + first customers (Q1 2027, 2 sprints)

### US-300 [⬜] [P0] — FastAPI service skeleton [5pt]
**As a** founder
**I want to** `uvicorn realestate_bench.api:app`
**So that** I have a hosted endpoint

**Acceptance criteria:**
- [ ] POST /v1/extract
- [ ] API key auth
- [ ] All FR-001..FR-005 wired

### US-301 [⬜] [P0] — Stripe billing [5pt]
**As a** founder
**I want to** Stripe subscriptions
**So that** I can charge customers

**Acceptance criteria:**
- [ ] Stripe Checkout for self-serve signup
- [ ] Webhook updates API key + quota
- [ ] Test: end-to-end with Stripe test mode

### US-302 [⬜] [P0] — Usage metering [3pt]
**As a** founder
**I want to** per-key usage tracked
**So that** I can enforce quotas

**Acceptance criteria:**
- [ ] Per-key requests/day + cost accumulator
- [ ] 429 QUOTA_EXCEEDED on overage
- [ ] `/v1/usage` endpoint

### US-303 [⬜] [P1] — Rate limiting [3pt]
**As a** founder
**I want to** per-key rate limits
**So that** one noisy neighbor doesn't kill the box

**Acceptance criteria:**
- [ ] 100 req/min default per key
- [ ] 429 RATE_LIMITED on overage

### US-310 [⬜] [P0] — Landing page [5pt]
**As a** founder
**I want to** a one-page landing site
**So that** I have something to link to

**Acceptance criteria:**
- [ ] Hero: claim + benchmark chart
- [ ] Pricing section
- [ ] Signup → Stripe checkout
- [ ] Hosted at `realestate.bench.app` (or similar)

### US-311 [⬜] [P0] — Public benchmark dashboard [5pt]
**As a** founder
**I want to** a page showing our model on ExtractBench
**So that** prospects see the numbers

**Acceptance criteria:**
- [ ] Auto-updated from `bench/results/`
- [ ] Comparison vs NuExtract3, ReaderLM-v2
- [ ] Methodology explained

### US-320 [⬜] [P0] — First design partner LOI [8pt]
**As a** founder
**I want to** 1 signed design-partner LOI
**So that** we have a paying customer and case study

**Acceptance criteria:**
- [ ] 1 paid pilot ($100-500/mo) signed
- [ ] 30-day evaluation period
- [ ] Before/after metrics captured

### US-321 [⬜] [P0] — Onboarding flow [3pt]
**As a** new customer
**I want to** a smooth signup → first-extraction flow
**So that** I get value in <5 minutes

**Acceptance criteria:**
- [ ] Email signup → API key emailed
- [ ] Quickstart doc with copy-paste curl
- [ ] First extraction success metric

### US-330 [⬜] [P1] — Observability (OpenTelemetry) [5pt]
**As a** founder
**I want to** OTel export
**So that** I can debug issues

**Acceptance criteria:**
- [ ] Per-extraction span
- [ ] Sub-spans for model call, validation, scoring
- [ ] OTel collector integration tested

### US-331 [⬜] [P1] — Alerts [3pt]
**As a** founder
**I want to** PagerDuty alerts on high error rate
**So that** I sleep at night

**Acceptance criteria:**
- [ ] Error rate > 5% over 5 min → page
- [ ] Latency p95 > 30s → warn

### US-340 [⬜] [P1] — Helm chart [3pt]
**As a** ops person
**I want to** a Helm chart
**So that** the hosted service can be deployed reproducibly

**Acceptance criteria:**
- [ ] Chart with API + worker pods
- [ ] Postgres + Redis subcharts

**Phase 3 Gate:** US-300..US-340 all complete (P0 only); 3-5 paying design partners; $500-2,500 MRR.

---

## Phase 4 — Schema marketplace (Q2 2027, 2 sprints)

### US-400 [⬜] [P0] — Schema registry repo [5pt]
**As a** schema author
**I want to** `realestate-bench/schemas` GitHub repo
**So that** I can submit my schema

**Acceptance criteria:**
- [ ] Public repo with CONTRIBUTING
- [ ] PR template + CI (lint + golden test)
- [ ] `realestate-bench schemas install <name>` fetches from registry

### US-410 [⬜] [P0] — 10 community schemas [8pt]
**As a** founder
**I want to** 10 community-contributed schemas merged
**So that** the marketplace has critical mass

**Acceptance criteria:**
- [ ] Schemas include: PMS exports, bank statements, credit reports, 1099s, vendor invoices (additional categories)

### US-420 [⬜] [P1] — Per-schema leaderboard [5pt]
**As a** user
**I want to** per-schema accuracy leaderboard
**So that** I can pick the right model per schema

**Acceptance criteria:**
- [ ] Accuracy on golden set
- [ ] Latency p95
- [ ] Cost/doc
- [ ] Updated weekly

### US-430 [⬜] [P0] — Web UI (upload + result) [8pt]
**As a** non-CLI user
**I want to** upload a doc + pick a schema + see result
**So that** I don't have to use the CLI

**Acceptance criteria:**
- [ ] Drag/drop upload
- [ ] Schema picker
- [ ] Result page with confidence coloring
- [ ] Review queue inline editor

**Phase 4 Gate:** US-400..US-430 all complete (P0 only); 20+ schemas in registry; $10-25K MRR.

---

## Phase 5 — Enterprise + multi-doc reasoning (H2 2027, 3 sprints)

### US-500 [⬜] [P0] — Multi-doc reasoning [13pt]
**As a** user with a portfolio
**I want to** extract from N docs in one workflow
**So that** I can do roll-ups

**Acceptance criteria:**
- [ ] Cross-reference leases within a portfolio
- [ ] Reconcile rent rolls across properties
- [ ] Test: 50-doc portfolio end-to-end

**Tasks:**
- Workflow orchestrator
- Cross-doc entity linking
- Roll-up aggregation

### US-510 [⬜] [P0] — Top-3 ExtractBench long split [8pt]
**As a** founder
**I want to** our model in the top 3 of ExtractBench long split
**So that** we have credibility

**Acceptance criteria:**
- [ ] F1 ≥ 80% on long split
- [ ] Submission + publication

### US-520 [⬜] [P0] — AppFolio/Buildium/Yardi export [8pt]
**As a** user
**I want to** extract → export to my PMS
**So that** data lands in the right place

**Acceptance criteria:**
- [ ] AppFolio CSV export
- [ ] Buildium JSON export
- [ ] Yardi YRL export

### US-530 [⬜] [P1] — SOC2 Type II [13pt]
**As a** enterprise buyer
**I want to** SOC2
**So that** I can buy

**Acceptance criteria:**
- [ ] Type II certification
- [ ] Public trust page
- [ ] Pen-test report

**Phase 5 Gate:** US-500..US-530 all complete; $1-10M ARR range.

---

## Deferred / V2 candidates

- Multi-language extraction (Spanish lease addenda)
- Custom fine-tuning as a service
- Tenant-facing features (never)
- Mobile app (never)

---

## Dependencies & Risks

| ID | Dependency / Risk | Mitigation |
|----|---|---|
| D1 | NuExtract3 / ReaderLM-v2 model availability | Track upstream; pin versions; have GLiNER2 fallback |
| D2 | vLLM / llama.cpp compatibility with macOS | Test on M-series before Phase 1 ship |
| D3 | PaddleOCR license (Apache-2.0, OK) | None |
| D4 | ExtractBench eval harness changes | Pin a commit; mirror to local |
| D5 | RealEstate365 design-partner timeline | Standalone product is viable; LOI doesn't gate |

---

## Velocity assumptions

- 6-hour focused sessions, ~3 sessions/week
- Solo + AI agents (Atlas/Vera/Kite from CADENCE; Hermes here)
- 1 sprint = 2 weeks
- ~15-25 points per sprint (solo + AI)
- Phase 0: 1 sprint (~25pt)
- Phase 1: 2 sprints (~50pt)
- Phase 2: 3 sprints (~60pt)
- Phase 3: 2 sprints (~40pt)
- Phase 4: 2 sprints (~40pt)
- Phase 5: 3 sprints (~60pt)

---

*Backlog v1.0 — RealEstate-Bench © 2026 ksjswaroop*
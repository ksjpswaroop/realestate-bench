# RealEstate-Bench — Architecture

## 1. System Overview

RealEstate-Bench is a CLI + Python library with a phased hosted-API future. The core pipeline: ingest → extract → validate → score → route. Architecture is intentionally thin — most of the work is delegated to a model backend.

```
┌─────────────────────────────────────────────────────────────────────┐
│                          User (CLI / Future API)                    │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      realestate-bench CLI (Typer)                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │  init    │ │ extract  │ │  batch   │ │   test   │ │ schemas  │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                  │
        ┌─────────────────────────┼─────────────────────────┐
        ▼                         ▼                         ▼
┌────────────────┐      ┌────────────────┐       ┌────────────────┐
│   Ingestion    │      │   Extraction   │       │  Golden Test   │
│                │      │                │       │                │
│ ┌────────────┐ │      │ ┌────────────┐ │       │ ┌────────────┐ │
│ │ pypdf       │ │      │ │ Prompt     │ │       │ │ load golden│ │
│ │ Pillow     │ │      │ │ builder    │ │       │ │ set        │ │
│ │ pdfplumber │ │      │ └─────┬──────┘ │       │ └─────┬──────┘ │
│ │ OCR fallback│ │      │       ▼        │       │       ▼        │
│ └────────────┘ │      │ ┌────────────┐ │       │ ┌────────────┐ │
└────────────────┘      │ │ Model      │ │       │ │ scorer     │ │
        │               │ │ Backend    │ │       │ └────────────┘ │
        │               │ │ (swappable)│ │       └────────────────┘
        ▼               │ └─────┬──────┘ │
┌────────────────┐      │       ▼        │
│  Document     │      │ ┌────────────┐ │       ┌────────────────┐
│  Cache        │◀────▶│ │ Parser     │ │       │   Validators   │
│  (SQLite)     │      │ └─────┬──────┘ │       │                │
└────────────────┘      │       ▼        │       │ - totals_math  │
                        │ ┌────────────┐ │       │ - date_sanity  │
                        │ │ Pydantic   │ │◀─────▶│ - currency     │
                        │ │ Validation │ │       │   normalize    │
                        │ └─────┬──────┘ │       └────────────────┘
                        │       ▼        │
                        │ ┌────────────┐ │       ┌────────────────┐
                        │ │Confidence  │ │       │ Review Queue   │
                        │ │ Scorer     │ ├──────▶│  (JSONL)       │
                        │ └────────────┘ │       └────────────────┘
                        └────────────────┘
                                  │
                                  ▼
                        ┌────────────────┐
                        │   Output       │
                        │   - JSON file  │
                        │   - Logs       │
                        │   - State DB   │
                        └────────────────┘
```

## 2. Component Breakdown

### 2.1 CLI (`realestate_bench/cli/`)
- **Framework:** Typer (type-hint-driven CLI, auto-generated `--help`)
- **Commands:** `init`, `extract`, `batch`, `test`, `schemas`, `cache`, `config`
- **Shared logic:** argument validation, error formatting, color/format toggle

### 2.2 Ingestion (`realestate_bench/ingest/`)
- **`detect.py`:** MIME detection (extension + magic bytes)
- **`pdf.py`:** PDF text-layer extraction via `pypdf` (safe, no JS); fallback to `pdfplumber` for tables
- **`image.py`:** Pillow-based normalization (deskew, denoise)
- **`ocr.py`:** Optional OCR (Tesseract 5 + PaddleOCR); used when no text layer
- **`hash.py`:** SHA-256 streaming hash

### 2.3 Schemas (`realestate_bench/schemas/`)
- **`registry.py`:** Loads built-in schemas from `realestate_bench/schemas/built_in/` and user schemas from `~/.config/realestate-bench/schemas/`
- **`built_in/`:** Shipped schemas: `lease_addendum.py`, `rent_roll.py`, `vendor_invoice.py`, `lender_statement.py`
- **`prompts/`:** One prompt template per schema (`lease_addendum.prompt.md`)
- **`validators/`:** Reusable validators (`totals_math.py`, `date_sanity.py`, `currency_normalize.py`)

### 2.4 Extraction (`realestate_bench/extract/`)
- **`runner.py`:** Orchestrates: build prompt → call backend → parse → validate → score
- **`prompt_builder.py`:** Schema → prompt template (field descriptions + doc text + output format)
- **`parser.py`:** JSON-mode model output → Pydantic instance; handles JSON-in-code-fence extraction
- **`confidence.py`:** Per-field confidence (model logits if available, else heuristic: schema match + business-rule agreement)

### 2.5 Model Backends (`realestate_bench/backends/`)
- **`base.py`:** `class ModelBackend(Protocol)` with `def extract(doc_text, prompt) -> ModelOutput`
- **`vllm.py`:** Local vLLM server (CUDA)
- **`llama_cpp.py`:** llama.cpp / MLX (Metal/CUDA/CPU)
- **`http.py`:** Remote hosted (NuExtract API, OpenAI, etc.)
- **`registry.py`:** Maps model name → backend class; downloads weights on first use

### 2.6 State (`realestate_bench/state/`)
- **`db.py`:** SQLite wrapper using `sqlite3` (stdlib)
- **`migrations/`:** `001_initial.sql`, applied via `PRAGMA user_version`
- **`cache.py`:** Idempotency cache keyed by (doc_hash, schema_hash)
- **`models.py`:** Dataclasses for Document, Schema, Extraction, GoldenDoc, GoldenResult

### 2.7 Golden Test (`realestate_bench/golden/`)
- **`runner.py`:** Loads golden set, runs extraction, scores per-field F1
- **`scorer.py`:** F1 math (per-field exact match, Levenshtein for strings, IoU for arrays)
- **`report.py`:** Markdown + JSON report writer

### 2.8 External Services
- **Phase 1:** Hugging Face Hub (model weight download only)
- **Phase 3 (future):** Stripe (billing), PagerDuty (alerts)

## 3. Desktop App Component (Phase 1.5)

The desktop app is a **Tauri 2** shell that hosts a React/TypeScript UI and a Rust core; the Rust core spawns a **Python sidecar** for the actual extraction work. Nothing leaves the machine.

### 3.1 Why this split (Tauri + Python sidecar)

We chose not to write the model runner in Rust because:
1. **Cactus-needle is Apache-2.0 and ships as a Python wheel** (`pip install cactus-needle`, ~115 KB) with a battle-tested CLI. Reimplementing the grammar compiler + Simple Attention Network inference in Rust is months of work for zero accuracy gain.
2. **Tauri's `externalBin` / `resources`** let us bundle a Python interpreter + a `uv`-pinned venv with the app, so the user sees a single `.app` / `.msi` / `.AppImage`.
3. The model binary itself is **14 MB** — a tiny fraction of the bundle. The Python sidecar is the only piece that needs `uv` to ship slim.

The Rust core handles everything we want in a native, type-safe, low-overhead way:
- File system (PDF/image read, drag-and-drop ingestion)
- SQLite (`rusqlite[bundled]`) for the extracted-records store
- Excel export (`rust_xlsxwriter` — small, fast, no Java)
- Process spawn + IPC to the Python sidecar

### 3.2 Process topology

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Tauri Webview (React/TS UI)                     │
│  - Drag-drop a PDF                                                  │
│  - Pick a schema (lease / invoice / custom)                         │
│  - Render extracted JSON with per-field confidence coloring         │
│  - Edit fields inline                                               │
│  - Click "Export Excel" / "Export JSON" / "Open DB folder"          │
└─────────────────────────────────────────────────────────────────────┘
                                  │ Tauri commands (typed)
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                 Rust Core (src-tauri/src/)                          │
│  - commands/extract.rs  → spawn realestate-extract, parse JSON      │
│  - commands/db.rs       → SQLite CRUD on extractions + edits        │
│  - commands/export.rs   → rust_xlsxwriter, JSON, CSV                │
│  - db.rs                → rusqlite[bundled], FTS5                   │
│  - schema_loader.rs     → load Pydantic schemas from YAML/JSON      │
└─────────────────────────────────────────────────────────────────────┘
            │                                            │
            ▼                                            ▼
┌──────────────────────────┐              ┌──────────────────────────┐
│  Python Sidecar          │              │  Local SQLite DB         │
│  realestate-extract      │              │  ~/Library/.../app.db    │
│  (pip install            │              │  - documents             │
│   realestate-bench +     │              │  - extractions           │
│   cactus-needle)         │              │  - edits                 │
│                          │              │  - schemas (custom)      │
│  Reads PDF (pypdf) /     │              └──────────────────────────┘
│  OCR (paddleocr or       │
│  tesseract) → plain      │              ┌──────────────────────────┐
│  text → cactus-needle.   │              │  Tauri resources/        │
│  extract(text, Schema)   │              │  bundled/needle.blob     │
│  → JSON + confidences    │              │  (14 MB, model)          │
└──────────────────────────┘              └──────────────────────────┘
```

### 3.3 The cactus-needle contract

```python
import needle
from pydantic import BaseModel, Field
from typing import Annotated, Literal

class LeaseAddendum(BaseModel):
    landlord: Annotated[str, needle.Field(description="Legal name of the landlord or property owner")]
    tenant:   Annotated[str, needle.Field(description="Legal name of the tenant")]
    monthly_rent: Annotated[float, needle.Field(gt=0, le=1_000_000, description="USD per month")]
    currency:   Annotated[Literal["USD", "CAD", "EUR", "GBP", "MXN"], needle.Field()]
    start_date: Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="ISO-8601 date")]
    end_date:   Annotated[str, needle.Field(pattern=r"^\d{4}-\d{2}-\d{2}$", description="ISO-8601 date")]
    renewal_option: Annotated[Literal["none", "1-year", "2-year", "month-to-month"], needle.Field()] = "none"
    notes: str = ""

result = needle.extract(lease_text, LeaseAddendum)
# → LeaseAddendum(landlord="...", tenant="...", monthly_rent=2450.0, currency="USD", ...)
# Grammar-constrained decode means the output always parses; Field constraints
# (gt/le/pattern/enum) are compiled into the byte-level grammar at runtime.
```

The sidecar CLI (`realestate-extract`) takes JSON on stdin (path to PDF/image, schema name or schema definition), prints JSON on stdout (extraction + per-field confidence + tokens/sec + model version).

### 3.4 Tauri command surface (desktop)

| Command | Args | Returns |
|---|---|---|
| `list_schemas` | — | `Vec<SchemaMeta>` (id, name, fields, version) |
| `load_schema` | `schema_id: String` | `SchemaYaml` (the full schema definition) |
| `extract_document` | `doc_path: String, schema_id: String` | `ExtractionResult { fields, confidences, latency_ms }` |
| `save_extraction` | `extraction: ExtractionResult` | `i64` (db row id) |
| `update_field` | `extraction_id: i64, field: String, value: Value` | `()` |
| `list_extractions` | `filter: Option<String>` | `Vec<ExtractionRow>` (id, doc_name, schema, created_at) |
| `export_excel` | `extraction_ids: Vec<i64>, out_path: String` | `()` |
| `export_json` | `extraction_ids: Vec<i64>, out_path: String` | `()` |
| `export_csv` | `extraction_ids: Vec<i64>, out_path: String` | `()` |
| `get_app_data_dir` | — | `String` (path to the local SQLite DB) |

### 3.5 Storage schema (desktop `app.db`)

```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL,
    file_hash TEXT NOT NULL UNIQUE,   -- sha256 of file bytes
    page_count INTEGER,
    bytes INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_documents_path ON documents(path);

CREATE TABLE extractions (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES documents(id),
    schema_id TEXT NOT NULL,           -- e.g. 'lease_addendum'
    schema_version TEXT NOT NULL,      -- e.g. '1.0.0'
    raw_json TEXT NOT NULL,            -- the full extracted JSON
    confidences_json TEXT NOT NULL,    -- per-field confidence scores
    mean_confidence REAL NOT NULL,
    latency_ms INTEGER NOT NULL,
    model_version TEXT NOT NULL,       -- 'cactus-needle-3.2.1'
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_extractions_doc ON extractions(document_id);
CREATE INDEX idx_extractions_schema ON extractions(schema_id);
CREATE INDEX idx_extractions_created ON extractions(created_at);

CREATE TABLE edits (
    id INTEGER PRIMARY KEY,
    extraction_id INTEGER NOT NULL REFERENCES extractions(id),
    field_path TEXT NOT NULL,          -- JSON path, e.g. 'monthly_rent'
    old_value TEXT,
    new_value TEXT,
    edited_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_edits_extraction ON edits(extraction_id);

-- FTS5 for full-text search across all extractions
CREATE VIRTUAL TABLE extractions_fts USING fts5(
    doc_path, schema_id, raw_json,
    content='extractions', content_rowid='id'
);
```

### 3.6 Privacy & network policy

- **Zero network calls at runtime.** The app does not call HF, npm, Sentry, analytics, or anywhere.
- **Tauri CSP** set to `default-src 'self'; connect-src 'none'` to block any accidental XHR.
- **Tauri capabilities** grant only the permissions we need: `core:default`, `core:window:default`, `dialog:default`, `dialog:allow-save`, `dialog:allow-open`.
- **Wireshark-verified** during Phase 1.5 acceptance: run the app, do a full extraction + export, confirm no packets leave the machine (other than OS-level mDNS, which we filter).

## 4. Data Flow

### Flow A: Single-doc extraction (`extract` command)

```
User: realestate-bench extract --doc leases/42.pdf --schema lease_addendum --out out/42.json

1. CLI parses args
2. Ingestion:
   - detect MIME: PDF (magic bytes %PDF-)
   - extract text via pypdf → "This Lease Addendum..."
   - compute SHA-256 → a3f4e2...
3. Cache check:
   - key = sha256(doc) + sha256(schema)
   - HIT → return cached JSON in <100ms
   - MISS → continue
4. Schema load:
   - load lease_addendum v1.0.0 → 8e0d...
5. Prompt build:
   - template: schema field descriptions + doc text + output format example
   - return prompt string (~3-5K tokens)
6. Model call (VLLMBackend):
   - POST prompt to local vLLM server
   - receive JSON-mode output
   - timeout 60s
7. Parser:
   - extract JSON from response (handle code fences)
   - construct Pydantic LeaseAddendum instance
   - VALIDATION: schema type errors → ValidationFailed error
8. Validators:
   - run totals_math, date_sanity, currency_normalize
   - collect results
9. Confidence:
   - per-field scores (0.0-1.0)
10. Routing:
    - threshold 0.90 default
    - auto-accept if all fields ≥ 0.90
    - else: append to review_queue.jsonl
11. Persist:
    - write to out/42.json
    - log to state DB
    - cache the result
12. Return:
    - exit code 0
    - print summary line
```

### Flow B: Batch extraction (`batch` command)

```
User: realestate-bench batch --docs "invoices/*.pdf" --schema vendor_invoice --out out/ --concurrency 4

1. CLI parses args; glob → 47 paths
2. For each doc (parallel, 4 workers):
   - run Flow A (without cache miss path)
   - collect result
3. Persist results:
   - out/extracted/<doc_id>.json (47 files)
   - out/summary.json (counts)
   - out/review_queue.jsonl (low-confidence items)
4. Print summary:
   - "47/47 complete, 41 auto-accepted, 6 in review"
```

### Flow C: Golden-set regression (`test` command)

```
User: realestate-bench test --golden bench/golden/v1/ --baseline bench/baseline.json

1. Load golden set (30 docs, ground truths)
2. For each (doc, schema) pair:
   - run Flow A
3. Score:
   - per-field F1 vs ground truth
   - aggregate
4. Compare to baseline:
   - regression > 2% on any field → FAIL
   - else PASS
5. Write report:
   - bench/results/<date>/report.md
   - bench/results/<date>/metrics.json
6. Exit code:
   - 0 on PASS
   - 1 on FAIL
```

## 5. Concurrency Model

### Phase 1 (CLI)
- **Sequential by default** for `extract`
- **ProcessPoolExecutor** for `batch` with `--concurrency N` (each worker is a separate process; no shared state in memory)
- **Thread-safe:** State DB uses SQLite WAL mode; writes serialized by SQLite
- **GIL-friendly:** Model inference is in C++/Metal (vLLM, llama.cpp), not Python; Python threads only for I/O

### Phase 3 (hosted API)
- **FastAPI + Uvicorn:** Single-process async; one request per worker at a time
- **Bounded semaphore:** `MAX_CONCURRENT_INFERENCES` (typically = number of GPU slots)
- **Queue:** When semaphore full, request waits (timeout 60s)

### Failure isolation
- Each request is independent; failure of one does not affect others
- State DB errors don't crash the server; logged and returned as 500

## 6. Tauri/REST Command Surface

### CLI commands (Phase 1)
| Command | Purpose | Phase |
|---|---|---|
| `init` | First-run wizard; downloads model weights | 1 |
| `extract` | Single-doc extraction | 1 |
| `batch` | Multi-doc extraction | 2 |
| `test` | Golden-set regression | 1 |
| `schemas list/show` | Schema introspection | 1 |
| `cache list/clear` | Cache management | 2 |
| `config get/set` | Configuration | 1 |
| `review list/resolve` | Review queue mgmt | 2 |

### REST endpoints (Phase 3, future)
| Endpoint | Method | Auth | Purpose |
|---|---|---|---|
| `/v1/extract` | POST | API key | Single-doc |
| `/v1/batch` | POST | API key | Multi-doc |
| `/v1/schemas` | GET | API key | List |
| `/v1/schemas/{id}` | GET | API key | Detail |
| `/v1/usage` | GET | API key | Per-key usage |
| `/healthz` | GET | none | Liveness |

## 7. Security & Privacy

| Concern | Mitigation |
|---|---|
| Untrusted PDF (embedded JS) | Use `pypdf` (no JS execution); reject PDFs with embedded executables |
| API key leak (Phase 3) | argon2id-hashed storage; never logged; rotate via CLI |
| PII in logs | `--no-log-data` flag; auto-redact SSN/EIN/account# patterns |
| Doc exfiltration to cloud | Local-first default; hosted requires explicit `--allow-cloud` |
| Model weight tampering | SHA-256 verify on download; signed weights from upstream |
| Cache poisoning | Cache key = SHA-256(doc) + SHA-256(schema); immutable entries |
| Code injection in schema | Pydantic v2 sandboxed; no `eval`/`exec`; safe by design |
| SQL injection | Parameterized queries only; no string interpolation |
| Path traversal | Input paths validated against allowed roots; symlinks rejected |

### Trust boundaries
- **Process boundary:** CLI is a single process; no IPC needed
- **Backend boundary:** Model backends are local (vLLM, llama.cpp) — same trust domain; or HTTP for hosted (separate process, authenticated)
- **Network boundary:** Phase 1 has no network calls except first-run weight download; Phase 3 has API endpoints only

### Threat model (Phase 1)
- Adversary: malicious PDF
- Capabilities: embedded JS, embedded exploits
- Mitigation: `pypdf` is a safe reader (does not execute embedded code); reject PDFs with active content
- Residual risk: zero-day in `pypdf` — accept (low probability)

## 8. Platform-Specific Concerns

### macOS (M-series)
- **Backend:** llama.cpp / MLX (Metal acceleration)
- **Memory:** Unified memory; 12 GB minimum for NuExtract3
- **Install:** `pip install realestate-bench`; first-run downloads ~2-4 GB model
- **Quirks:** Metal Performance Shaders; avoid `--concurrency > 2` on 16 GB RAM

### Linux (CUDA)
- **Backend:** vLLM (preferred) or llama.cpp
- **Memory:** Dedicated VRAM; 16 GB for NuExtract3 fp16
- **Install:** `pip install realestate-bench` + CUDA toolkit
- **Quirks:** Watch OOM on long docs (chunked extraction mitigates)

### Linux (CPU)
- **Backend:** llama.cpp, quantized model (Q4_K_M)
- **Memory:** 8-12 GB RAM
- **Latency:** ~10× slower than GPU; OK for batch
- **Use case:** No GPU available, privacy-first ops

### Windows (best-effort)
- **Backend:** llama.cpp (Windows build)
- **Install:** `pip install realestate-bench`; first-run downloads model
- **Quirks:** Some POSIX paths; avoid in CI; defer to Phase 2

## 9. Build & Distribution

### Build system
- **`pyproject.toml`** with `hatchling` backend
- Pinned deps: `pydantic>=2.6`, `typer>=0.12`, `pypdf>=4.0`, `pillow>=10.0`, `tqdm>=4.66`
- Optional: `[ocr]` extra for pytesseract + paddleocr; `[vllm]` extra for vllm

### Distribution
- **PyPI:** `pip install realestate-bench` (primary)
- **Docker:** `docker pull ksjswaroop/realestate-bench` (Phase 3 hosted)
- **Single-binary:** PyInstaller bundle for non-Python shops (Phase 4)

### CI/CD
- GitHub Actions on PR: lint + unit + integration + golden-set regression
- On merge to main: build + publish to TestPyPI
- On tag: publish to PyPI + build docker image

## 10. Performance Targets

| Metric | Target | Hardware | Measurement |
|---|---|---|---|
| Single-doc latency (Lease Addendum, 4 pages) | < 30s p95 | M-series Mac | bench/perf/ |
| Single-doc latency | < 5s p95 | 1× A100 | bench/perf/ |
| Batch throughput | ≥ 200 docs/hour | 1× A100 | bench/perf/ |
| Cache hit | < 100ms p99 | any | bench/perf/ |
| Schema validation | < 50ms p99 | any | unit test |
| Confidence scoring | < 20ms p99 | any | unit test |
| Cold-start (model load) | < 60s | any | bench/perf/ |
| Memory (idle) | < 500 MB | any | `psutil` |
| Memory (extraction) | < 16 GB VRAM | A100 | `nvidia-smi` |
| Memory (extraction) | < 12 GB unified | M-series | `psutil` |

## 11. Failure Modes & Recovery

| Failure | Detection | Recovery |
|---|---|---|
| Model timeout (> 60s) | backend raises TimeoutError | Return MODEL_TIMEOUT; user retries with --timeout 120 or smaller model |
| Out of VRAM | CUDA OOM exception | Return MODEL_OOM; suggest quantized model |
| Malformed JSON from model | Pydantic ValidationError | Return VALIDATION_FAILED with parse error; raw response in trace for debugging |
| Missing text layer (PDF) | pypdf returns empty | Fallback to OCR (Phase 2); else return NO_TEXT |
| Schema file not found | ImportError | Return SCHEMA_NOT_FOUND |
| Doc not found | FileNotFoundError | Return DOC_NOT_FOUND |
| State DB corrupted | sqlite3.DatabaseError | `realestate-bench init --recover` rebuilds from cached files |
| Disk full on cache write | OSError | Log warning; continue without caching |

### No silent failures
- Every failure returns a structured error code (JSON or exit code)
- Errors logged with traceback (DEBUG level) + user-friendly message (INFO level)

## 12. Testing Strategy

### Layer 1: Unit tests (pytest)
- `tests/unit/` — pure logic
- Target: 100% of `extract/`, `schemas/`, `state/`, `golden/scorer.py`
- Run: < 30s

### Layer 2: Integration tests
- `tests/integration/` — end-to-end on 3 known docs
- Each test has a known input → known output
- Uses real model (slow; in CI: use small mock model)

### Layer 3: Golden-set regression
- `realestate-bench test --golden bench/golden/v1/`
- 30 docs for Lease Addendum (Phase 1), 60 docs combined (Phase 2)
- CI gate: per-field F1 doesn't regress > 2%

### Layer 4: Benchmark suite
- `bench/run.py` — full eval on 7 public datasets
- Phase 0 only (run before model selection)
- Result: `bench/results/<date>/BENCH_RESULTS.md`

### Layer 5: Property-based tests (Phase 4)
- Hypothesis-generated docs + schemas
- Catches edge cases in validator logic

### Test data
- 30 lease addenda (hand-labeled, public/redacted)
- 60 combined golden (Phase 2)
- Mock OCR outputs for OCR tests

### Mocking strategy
- **Model backends:** Mock class returns canned JSON; allows testing without GPU
- **OCR:** Mock returns deterministic text
- **State DB:** In-memory SQLite per test

## 13. Open Architectural Questions

- OA1: Will we need a message queue (Redis/RabbitMQ) for hosted batching in Phase 3? (Defer; FastAPI + asyncio + semaphore is sufficient for v1.)
- OA2: How to handle schema evolution when a customer upgrades? (Phase 4 decision; current plan = versioned schemas + migration tool.)
- OA3: Should the state DB be Postgres in Phase 3 hosted, or stick with SQLite + Litestream? (Lean Postgres for multi-instance.)
- OA4: Web UI framework (Phase 4): Tauri, Next.js, Streamlit? (Defer.)
- OA5: Multi-language docs: is the model good enough, or do we need per-language models? (Phase 4 eval.)

## 14. Future Extensions

### Phase 4 additions
- Web UI (Tauri desktop or Next.js web)
- Schema marketplace (GitHub-based PR workflow)
- Multi-doc reasoning (cross-referencing leases within a portfolio)
- OpenTelemetry export

### Phase 5 additions
- ERP/PMS export (AppFolio, Buildium, Yardi)
- Multi-tenant isolation
- SOC2 Type II certification
- Custom fine-tuning as a service

### Out of scope forever
- Replacing Yardi/AppFolio/Buildium
- Tenant-facing features
- Mobile apps
- HIPAA workloads

## 15. Appendix: Crate/Dep Tree

```
realestate-bench/
├── pyproject.toml
├── README.md
├── LICENSE                                  # Apache-2.0
├── PRD.md
├── ROADMAP.md
├── BACKLOG.md
├── ARCHITECTURE.md                          # this file
├── docs/
│   ├── DESIGN_SYSTEM.md
│   ├── WIREFRAMES.md
│   ├── VALIDATORS.md
│   ├── CI.md
│   └── PROMPT_TEMPLATES.md
├── datasets/
│   └── (7 benchmark datasets, see datasets/README.md)
├── bench/
│   ├── run.py                               # main benchmark runner
│   ├── score.py                             # per-field F1
│   ├── schema.py                            # benchmark schemas
│   ├── loaders.py                           # dataset loaders
│   ├── golden/
│   │   └── v1/                              # hand-labeled golden set
│   ├── baseline.json
│   └── results/                             # per-run reports
├── src/realestate_bench/
│   ├── __init__.py
│   ├── cli/
│   │   ├── __init__.py
│   │   ├── main.py                          # Typer app
│   │   ├── init.py
│   │   ├── extract.py
│   │   ├── batch.py
│   │   ├── test.py
│   │   ├── schemas.py
│   │   ├── cache.py
│   │   └── config.py
│   ├── ingest/
│   │   ├── detect.py
│   │   ├── pdf.py
│   │   ├── image.py
│   │   ├── ocr.py
│   │   └── hash.py
│   ├── schemas/
│   │   ├── registry.py
│   │   ├── built_in/
│   │   │   ├── lease_addendum.py
│   │   │   ├── rent_roll.py
│   │   │   ├── vendor_invoice.py
│   │   │   └── lender_statement.py
│   │   ├── prompts/
│   │   │   └── lease_addendum.prompt.md
│   │   └── validators/
│   │       ├── totals_math.py
│   │       ├── date_sanity.py
│   │       └── currency_normalize.py
│   ├── extract/
│   │   ├── runner.py
│   │   ├── prompt_builder.py
│   │   ├── parser.py
│   │   └── confidence.py
│   ├── backends/
│   │   ├── base.py
│   │   ├── vllm.py
│   │   ├── llama_cpp.py
│   │   ├── http.py
│   │   └── registry.py
│   ├── state/
│   │   ├── db.py
│   │   ├── cache.py
│   │   ├── models.py
│   │   └── migrations/
│   │       └── 001_initial.sql
│   └── golden/
│       ├── runner.py
│       ├── scorer.py
│       └── report.py
└── tests/
    ├── unit/
    ├── integration/
    └── test_golden.py
```

## 16. Appendix: Data Directory Layout

```
~/.local/share/realestate-bench/
├── state.db                                 # SQLite (extractions, cache, configs)
├── logs/
│   ├── realestate-bench.log                 # current
│   └── realestate-bench.log.1.gz            # rotated (7 days)
└── review_queue.jsonl                       # current review queue

~/.cache/realestate-bench/
├── models/
│   ├── nuextract3-4b/
│   │   ├── config.json
│   │   ├── model.safetensors
│   │   └── tokenizer.json
│   └── readerlm-v2-1.5b/
│       └── ...
└── results.db                                # idempotency cache (separate from state)

~/.config/realestate-bench/
├── config.yaml                               # user preferences
└── schemas/                                  # user-defined schemas (git ignored)
    └── my_custom_schema.py
```

---

*Architecture v1.0 — RealEstate-Bench © 2026 ksjswaroop — Apache-2.0 (code), CC-BY-4.0 (data/docs)*
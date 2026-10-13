# RealEstate-Bench — Wireframes

> CLI "wireframes" are terminal-output mockups. Component names match `docs/DESIGN_SYSTEM.md` exactly. Future Web UI mockups (Phase 4) are HTML at the bottom of this doc.

---

## 1. CLI: `realestate-bench --help`

```
╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║  RealEstate-Bench 0.1.0                                        ║
║  Honest document extraction for real estate.                   ║
║                                                                ║
║  Usage: realestate-bench [OPTIONS] COMMAND [ARGS]...          ║
║                                                                ║
║  Commands:                                                     ║
║    init          First-run setup wizard                        ║
║    extract       Extract structured data from a document       ║
║    batch         Extract from many documents                   ║
║    test          Run golden-set regression tests               ║
║    schemas       List or show schemas                          ║
║    cache         Manage the idempotency cache                  ║
║    config        Get or set configuration values               ║
║    review        Manage the review queue                       ║
║                                                                ║
║  Options:                                                      ║
║    --version              Show version                         ║
║    --help                 Show this help                       ║
║    --log-format [json|text]                                   ║
║    --verbose              Increase verbosity                   ║
║    --debug                Show debug-level logs                ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝
```

**Specs:** Width 80ch, padding 1em, no border decoration in actual TUI (here shown as box for illustration).

---

## 2. CLI: `realestate-bench init` — first-run wizard

```
╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║   Welcome to RealEstate-Bench v0.1.0                           ║
║                                                                ║
║   This wizard will:                                            ║
║     1. Pick a model backend (download weights on first run)    ║
║     2. Run a sample extraction to verify everything works      ║
║     3. Set your default confidence threshold                   ║
║                                                                ║
║   Press Enter to continue, Ctrl+C to abort.                    ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝

[INFO] Detecting system...

  OS: macOS 14.5 (arm64)
  CPU: Apple M2 Max (12 cores)
  Memory: 32 GB unified
  GPU: Apple M2 Max (Metal)
  Python: 3.11.9
  pip: 24.0

[OK] System looks good.

  Model backend options:

  > NuExtract3 (4B) - Recommended, 8 GB download, ~12 sec/doc
    ReaderLM-v2 (1.5B) - 3 GB, faster, ~6 sec/doc
    GLiNER2 (486M) - 1 GB, CPU-friendly, entity-only fields
    Custom path - Specify a local model

  Use arrow keys to move, Enter to select.
  Press q to quit.
```

After model selection:

```
[INFO] Downloading model: NuExtract3 (4B)
  ██████████████████████████████░░░░░░░░░  4.2 GB / 8.0 GB  (00:01:24)

[OK] Download complete.

[INFO] Running sample extraction on docs/sample_lease.pdf...

  Detection: PDF (digital), 4 pages
  Hash: a3f4e2c1...
  Model: NuExtract3-4B
  Schema: lease_addendum v1.0.0
  Extraction: 11.2 sec, 4521 in / 187 out tokens

  [OK] Sample extraction succeeded.

  ┌────────────────────────────────────┐
  │ lease_id       LA-2026-0042       │
  │ tenant         Acme Corp           │
  │ monthly_rent   $4,500.00 USD       │
  │ confidence     0.94 (mean)         │
  └────────────────────────────────────┘

  [OK] Ready. Try: realestate-bench extract --doc YOUR_FILE.pdf --schema lease_addendum
```

**Specs:** Wizard uses `rich.prompt` for interactive selection. Progress bars use `tqdm`.

---

## 3. CLI: `realestate-bench extract` — happy path

```
$ realestate-bench extract \
    --doc leases/addendum_42.pdf \
    --schema lease_addendum \
    --out out/addendum_42.json \
    --confidence-threshold 0.85 \
    --verbose

[INFO] Document: leases/addendum_42.pdf
[INFO] Detected: PDF (digital), 4 pages, 38 KB
[INFO] Hash: a3f4e2c1...
[INFO] Schema: lease_addendum v1.0.0 (sha: 8e0d9f4a)
[INFO] Model: NuExtract3-4B (local, Metal)
[INFO] Confidence threshold: 0.85
[DEBUG] Prompt: 1247 tokens
[INFO] Calling model... (timeout 60s)

  ████████████████████████░░░░░░  72% (00:08)

[INFO] Model returned 187 tokens in 11.2 sec
[DEBUG] Raw output: {"lease_id": "LA-2026-0042", ...}
[DEBUG] Pydantic validation: PASS
[DEBUG] Validators: totals_math PASS, date_sanity PASS, currency_normalize PASS
[DEBUG] Confidence: 11/12 fields >= 0.85

[OK] Extraction complete. Wrote out/addendum_42.json (1.2 KB)
[OK] 11/12 fields auto-accepted (mean confidence 0.92)
[WARN] 1 field below threshold, added to review queue:
        renewal_option (confidence 0.72)

$ ls -la out/
-rw-r--r--  1 ksjswaroji  staff   1.2 KB  Oct 13 11:42 addendum_42.json
-rw-r--r--  1 ksjswaroji  staff   312 B   Oct 13 11:42 review_queue.jsonl
```

**Specs:** Color codes `[INFO]` info, `[OK]` pass/green, `[WARN]` warn/yellow, `[ERROR]` error/red. `--verbose` adds `[DEBUG]` dim/gray lines.

---

## 4. CLI: `realestate-bench batch`

```
$ realestate-bench batch \
    --docs "invoices/*.pdf" \
    --schema vendor_invoice \
    --out out/ \
    --concurrency 4

[INFO] 47 documents matched
[INFO] Schema: vendor_invoice v1.0.0
[INFO] Concurrency: 4 workers

  ████████████████████████████████░░░░  44/47 (00:02:13) ETA 00:00:09

[INFO] 47/47 extractions complete
[INFO] Mean latency: 8.3 sec/doc, p95: 14.1 sec/doc
[INFO] Cost: $0.012/doc, total: $0.56

[OK] Results:
  out/extracted/      47 JSON files (52 KB total)
  out/summary.json     1.2 KB
  out/review_queue.jsonl  6 items (12.8% of total)
```

Summary report (`out/summary.json`):

```json
{
  "batch_id": "batch-2026-10-13-1145",
  "schema": "vendor_invoice",
  "model": "NuExtract3-4B",
  "total_docs": 47,
  "auto_accepted": 41,
  "review_queued": 6,
  "validation_passed": 45,
  "validation_failed": 2,
  "errors": 0,
  "mean_latency_ms": 8300,
  "p95_latency_ms": 14100,
  "total_cost_usd": 0.56,
  "started_at": "2026-10-13T11:45:32Z",
  "completed_at": "2026-10-13T11:48:05Z"
}
```

---

## 5. CLI: `realestate-bench test` — golden-set regression

```
$ realestate-bench test --golden bench/golden/v1/ --verbose

[INFO] Loading golden set: bench/golden/v1/
[INFO] Found 30 lease_addendum docs + 30 rent_roll docs (60 total)
[INFO] Baseline: bench/baseline.json
[INFO] Running extraction on each...

  lease_addendum  ████████████████████  30/30
  rent_roll       ████████████████████  30/30

[INFO] Scoring per-field F1...

  === lease_addendum ===
  Field                  Baseline   Current   Delta
  lease_id               0.95       0.97      +0.02
  tenant                 0.92       0.94      +0.02
  premises               0.88       0.91      +0.03
  effective_date         0.96       0.97      +0.01
  expiration_date        0.96       0.97      +0.01
  monthly_rent.amount     0.93       0.94      +0.01
  monthly_rent.currency  0.99       0.99      0.00
  deposit.amount         0.90       0.91      +0.01
  deposit.currency       0.99       0.99      0.00
  escalation.type        0.87       0.88      +0.01
  escalation.annual_pct  0.84       0.85      +0.01
  renewal_option         0.71       0.72      +0.01
  ──────────────────────────────────────────────
  Mean                   0.91       0.92      +0.01

  === rent_roll ===
  Field                  Baseline   Current   Delta
  unit_id                0.96       0.97      +0.01
  sqft                   0.94       0.95      +0.01
  rent                   0.93       0.94      +0.01
  lease_start            0.95       0.96      +0.01
  lease_end              0.95       0.96      +0.01
  escalation             0.88       0.89      +0.01
  ──────────────────────────────────────────────
  Mean                   0.94       0.95      +0.01

[OK] All fields within regression tolerance (< 2% drop).
[OK] Full report written to bench/results/2026-10-13-1145/report.md
```

Failure state:

```
  monthly_rent.amount     0.93       0.84      -0.09  <-- REGRESSION

[ERROR] Regression detected on field: monthly_rent.amount
[ERROR]   Current: 0.84
[ERROR]   Baseline: 0.93
[ERROR]   Delta: -0.09 (threshold -0.02)
[ERROR] 1 field failed regression check.

$ echo $?
1
```

---

## 6. CLI: `realestate-bench schemas list` / `show`

```
$ realestate-bench schemas list

[INFO] Built-in schemas:

  ID                  Version   Fields   Description
  ──────────────────────────────────────────────────────────────────────
  lease_addendum      1.0.0     12       Lease addenda (dates, escalation, deposit)
  rent_roll            1.0.0     6        Rent rolls (units, sqft, lease terms)
  vendor_invoice      1.0.0     9        Vendor invoices (line items, totals)
  lender_statement    1.0.0     7        Lender statements (balance, transactions)

[INFO] User schemas (from ~/.config/realestate-bench/schemas/):

  ID                  Version   Fields   Description
  ──────────────────────────────────────────────────────────────────────
  (none yet)
```

```
$ realestate-bench schemas show lease_addendum

[INFO] Schema: lease_addendum v1.0.0
[INFO] SHA: 8e0d9f4a...
[INFO] Fields (12):

  lease_id              str       Unique identifier (e.g. "LA-2026-0042")
  tenant                str       Tenant name (legal entity or individual)
  premises              str       Premises description (address or unit)
  effective_date        date      Lease effective date (YYYY-MM-DD)
  expiration_date       date      Lease expiration date (YYYY-MM-DD)
  monthly_rent          Money     Monthly rent amount + currency code
  deposit               Money     Security deposit amount + currency code
  escalation            Escalation  Type + parameters
  renewal_option        str?      Renewal terms (free text, optional)
  signatures            list      Signatory names + dates
  governing_law         str?      Governing jurisdiction (optional)
  notes                 str?      Free-form notes (optional)

[INFO] Validators registered: totals_math, date_sanity, currency_normalize
[INFO] Prompt template: realestate_bench/schemas/prompts/lease_addendum.prompt.md
[INFO] Example doc: docs/sample_lease.pdf
```

---

## 7. CLI: `realestate-bench review`

```
$ realestate-bench review list

[INFO] Pending review items: 6

  ID                Doc                              Schema             Fields below threshold
  ──────────────────────────────────────────────────────────────────────────────────────
  rev-a3f4          invoices/inv_044.pdf             vendor_invoice     monthly_rent.confidence
  rev-b7e2          invoices/inv_089.pdf             vendor_invoice     total
  rev-c1d8          invoices/inv_112.pdf             vendor_invoice     line_items[3].amount
  rev-d9f3          leases/lease_007.pdf              lease_addendum     renewal_option
  rev-e5a1          leases/lease_023.pdf              lease_addendum     deposit.amount
  rev-f2b6          leases/lease_041.pdf              lease_addendum     escalation.annual_pct

$ realestate-bench review resolve rev-a3f4 --correction '{"monthly_rent": 4500.00}'

[INFO] Original extraction: monthly_rent.confidence = 0.78
[INFO] Correction: 4500.00
[INFO] Marked as resolved.
[INFO] Saved to bench/corrections/rev-a3f4.json
[INFO] With your permission, corrections feed into the golden set.
       Run: realestate-bench review approve-corrections --since 2026-10-13
```

---

## 8. CLI: model timeout error

```
$ realestate-bench extract --doc huge_500page.pdf --schema rent_roll --timeout 60

[INFO] Document: huge_500page.pdf
[INFO] Detected: PDF (digital), 500 pages, 12 MB
[INFO] Hash: 9f8e7d6c...
[INFO] Schema: rent_roll v1.0.0
[INFO] Calling model... (timeout 60s)

[ERROR] MODEL_TIMEOUT: extraction took longer than 60s
[ERROR] Suggestions:
[ERROR]   - Try: --timeout 180 (gives 3x more time)
[ERROR]   - Or:   Use a smaller model (--model gliner2-large-v1)
[ERROR]   - Or:   Split the document into smaller chunks
[ERROR] Cached result not available for this (doc, schema) pair.

$ echo $?
1
```

---

## 9. CLI: cache hit on retry

```
$ realestate-bench extract --doc leases/addendum_42.pdf --schema lease_addendum

[INFO] Document: leases/addendum_42.pdf
[INFO] Cache hit for (a3f4e2c1..., 8e0d9f4a...) → returning cached result in 0.04s

[OK] Extraction complete (cached). Wrote out/addendum_42.json
```

---

## 10. Micro-interactions reference

| Trigger | Animation | Duration |
|---|---|---|
| Progress bar tick | tqdm standard | continuous |
| `--verbose` mode toggle | log lines appear as you type | instant |
| Color/NO_COLOR detection | n/a (env var check) | instant |
| `--help` | Typer auto-render | <100ms |
| `--version` | print + exit | <50ms |

---

## 11. (Future) Web UI: Upload + Result

**Phase 4 — Not in current scope.** Mock HTML at `docs/wireframes/upload-result.html` (TBD).

ASCII preview of the future UI:

```
┌─────────────────────────────────────────────────────────────────┐
│  RealEstate-Bench                                       [user]  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Upload a document                                             │
│                                                                 │
│   ┌───────────────────────────────────────────────────────┐   │
│   │                                                       │   │
│   │            Drop a PDF, JPEG, PNG, or TIFF             │   │
│   │                  or click to browse                   │   │
│   │                                                       │   │
│   └───────────────────────────────────────────────────────┘   │
│                                                                 │
│   Pick a schema:                                                │
│                                                                 │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│   │ Lease        │  │ Rent Roll    │  │ Vendor       │        │
│   │ Addendum     │  │              │  │ Invoice      │        │
│   │              │  │              │  │              │        │
│   │ 12 fields    │  │ 6 fields     │  │ 9 fields     │        │
│   │ v1.0.0       │  │ v1.0.0       │  │ v1.0.0       │        │
│   └──────────────┘  └──────────────┘  └──────────────┘        │
│                                                                 │
│   [ Upload ]                                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

Result page:

```
┌─────────────────────────────────────────────────────────────────┐
│  RealEstate-Bench                                       [user]  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ← Back   leases/addendum_42.pdf                                │
│                                                                 │
│  Schema: lease_addendum v1.0.0   Model: NuExtract3-4B           │
│  Latency: 11.2s              Tokens: 4521 in / 187 out          │
│                                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━     │
│                                                                 │
│  ▼ lease_id                              [0.98] ✓               │
│      LA-2026-0042                                                │
│                                                                 │
│  ▼ tenant                                [0.95] ✓               │
│      Acme Corp                                                   │
│                                                                 │
│  ▼ premises                              [0.92] ✓               │
│      Unit 4B, 123 Main St                                        │
│                                                                 │
│  ▼ effective_date                        [0.97] ✓               │
│      2026-01-01                                                  │
│                                                                 │
│  ▼ monthly_rent                          [0.94] ✓               │
│      amount: 4500.00                                             │
│      currency: USD                                               │
│                                                                 │
│  ▼ renewal_option                        [0.72] ⚠ REVIEW        │
│      Two 5-year options at fair market                          │
│                                                                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━     │
│                                                                 │
│  Validation: totals_math ✓   date_sanity ✓   currency ✓        │
│                                                                 │
│  [ Edit ]  [ Mark all correct ]  [ Re-run ]                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Component names (must match §7.4 of DESIGN_SYSTEM.md):** `UploadDropZone`, `SchemaPicker`, `JsonTree`, `ConfidenceBadge`, `ValidationPill`.

---

*Wireframes v1.0 — RealEstate-Bench © 2026 ksjswaroop*
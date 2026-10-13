# RealEstate-Bench — Product Requirements Document

## 1. Document Control
- Product: RealEstate-Bench
- Codename: `realestate-bench`
- Version: 0.1
- PRD: 1.0
- Status: Draft — Phase 0 in progress
- Author: ksjswaroop (with Hermes Agent assistance)
- Stakeholders: founder (ksjswaroop), RealEstate365 product, design-partner prospects
- Created: 2026-10-13
- Updated: 2026-10-13
- Repository: `~/projects/realestate-bench/` (local; GitHub target `ksjswaroop/realestate-bench`)
- License: Apache-2.0 (code), CC-BY-4.0 (data and docs)
- Supersedes: `BACKLOG.md` v0.1 and `ROADMAP.md` v0.1 — those become living trackers; this PRD is the contract

Revision history
| Ver | Date | Author | Notes |
|-----|------|--------|-------|
| 1.0 | 2026-10-13 | ksjswaroop + Hermes | Initial PRD; Phase 0 gate = per-field F1 ≥ 0.80 on invoice-extraction-benchmark |

## 2. Executive Summary

### Vision
A small, fast, accurate document-extraction engine that turns pricing pages, leases, rent rolls, contracts, and financial statements into validated JSON, accurate enough to trust inside RealEstate365 workflows and sellable as an API to small real-estate operators.

### Problem
Real-estate operators (1-50 person property managers, brokers, lenders, asset managers) receive hundreds of pages of structured-but-not-structured documents every month — leases with rent escalation clauses, rent rolls in Excel with hand-tweaked columns, vendor invoices with regional tax conventions, lender statements with line items buried in 40-page PDFs. Each document needs to become structured data to be useful for analytics, comps, or portfolio roll-ups. Today's options are: (1) copy/paste by humans (slow, error-prone, ~$4-12 per doc labor cost), (2) hosted AI extraction services (NuExtract, ReaderLM-v2 API, LlamaExtract — $0.05-0.50/page, accuracy varies wildly on real-estate docs, vendor lock-in), or (3) fine-tune a custom OCR+KIE pipeline (multi-month engineering project, requires ML team).

### Solution
RealEstate-Bench is a CLI + Python library + (later) hosted API that:
1. Picks the best small open model for the job using a transparent, reproducible benchmark (no vendor self-dealing)
2. Runs the model on pricing/date/line-item schemas specific to real estate
3. Validates outputs against Pydantic schemas and business rules (totals math, date sanity, currency normalize)
4. Flags low-confidence fields for human review
5. Improves over time as corrections accumulate

### Business value
- Saves $4-12/doc in human data-entry costs for a 1,000-doc/month portfolio operator ($48K-115K/year saved)
- Lets a small operator make portfolio decisions in hours instead of weeks (rent comps, lease renewals, NOI analysis)
- Beats the accuracy of hosted AI on real-estate documents because the schema and golden set are purpose-built for the domain
- Becomes a building block for RealEstate365 (education → data platform → PropTech SaaS) — students and operators see it first, then use it, then refer it

### Success definition (12-month horizon)
- 5 paying design partners × $100-500/mo = $500-$2,500 MRR by Q1 2027
- 30-doc held-out golden set for at least 2 schemas (Lease Addendum, Rent Roll) with per-field F1 ≥ 0.85
- Published ExtractBench short-split submission in the top-10 leaderboard bracket
- 1 LOI from a RealEstate industry partner for production deployment
- 1 co-marketing case study (operator's name + before/after metrics)

## 3. Strategic Context

### Market analysis

**TAM (Total Addressable Market) — Document AI / Intelligent Document Processing (IDP):**
Global IDP market $2.5B in 2025, projected $10-13B by 2030 (CAGR 25-30%). Source: Grand View Research, Markets and Markets 2025.
Real-estate-specific slice: ~5-8% of horizontal SaaS document processing = $125M-1B. RealEstate-specific vendors (AppFolio, Buildium, Yardi) process millions of lease/contract PDFs annually; their document extraction modules are the main internal cost.

**SAM (Serviceable Addressable Market) — US small-to-mid real-estate operators:**
~60,000 property management firms in the US with 10-500 units (NAA 2024), ~25,000 small brokerages, ~5,000 small mortgage shops. At $100-500/mo, 1% capture = $60K-$3M/year SAM.

**SOM (Serviceable Obtainable Market) — Year 1:**
50-200 design-partner-style customers in solo+AI bootstrap mode = $60K-$1.2M ARR realistic. Achieved through community (Indie Hackers, Bigger Pockets, Reddit r/realestate), content (case studies with anonymized metrics), and one RealEstate365 referral channel.

### Competitor analysis

| Competitor | Strength | Weakness | Implication for us |
|---|---|---|---|
| NuMind (NuExtract / NuExtract2) | Strong accuracy on benchmark, simple API | $0.04-0.12/page, schema changes require re-prompting, no real-estate-specific golden set | Match accuracy on real-estate docs, undercut on price, ship a golden set for free |
| Reducto | High accuracy on long/poor-quality docs | $0.10-0.30/page, enterprise-only pricing, no self-serve | Stay focused; long-docs is Phase 4, not Phase 1 |
| LlamaExtract (LlamaIndex) | Strong on long docs | $0.20+/page, requires LlamaIndex stack | Beat on price + specific real-estate schemas |
| Landing AI | Computer-vision specialist | Steep learning curve, image-first bias | Avoid; we're text+layout |
| Google DocAI / AWS Textract | Battle-tested OCR | Per-feature pricing, requires custom field training, no real-estate out of box | Could be our OCR backend in Phase 4 |
| Manual data entry (VA, offshored) | Cheap per-doc | Slow, error-prone, doesn't scale | Our labor-cost-savings pitch wins directly here |
| AppFolio/Buildium/Yardi internal tools | Integrated with PMS | Tied to that PMS, not portable | Don't fight the PMS, be the doc-extraction layer that feeds any PMS |

### Positioning
"The honest document extractor for real estate." Differentiators:

1. **Transparent benchmark** — Every model claim has a published number on ExtractBench + the drew-ipp invoice benchmark + our own real-estate golden set. No vendor self-dealing.
2. **Real-estate-first schemas** — Lease Addendum, Rent Roll, Vendor Invoice, Lender Statement. Not generic.
3. **Local-first by default** — Self-hosted model weights, no API keys required for MVP. Privacy-conscious operators trust this.
4. **Confidence-then-route** — Auto-accept high-confidence fields, queue the rest for review. Tells operators where to spend human time.
5. **Cheap** — $0.01-0.03/page target at scale (single-GPU self-host) vs $0.05-0.50 for hosted competitors.

### Differentiators (long-form)
- **Golden set in the open.** Most competitors publish accuracy on data they generated themselves or have a financial relationship with. RealEstate-Bench's per-domain golden set is from real documents with operators' permission and reviewable in `bench/golden/`. This is our long-term moat.
- **Model-agnostic.** The architecture supports swapping the model (NuExtract3, ReaderLM-v2, GLiNER2, custom fine-tune) without changing the API. Operators can A/B on their own docs.
- **Cost transparency.** Every extraction logs model, latency, GPU-seconds, and dollar-equivalent. No surprise bills.
- **No vendor lock-in on schemas.** Schemas are Pydantic v2 in a git repo. Operators can fork them.
- **Honest failures.** When confidence is low or schema validation fails, we say so with evidence (which fields, why). Most extractors silently produce wrong JSON.

## 4. Goals & Objectives

### Business goals
- B1: Reach $500-$2,500 MRR by Q1 2027 (5-25 design partners at $100-500/mo)
- B2: Reach $10-25K MRR by Q2 2027 (after Phase 4 schema marketplace launches)
- B3: Stay bootstrapped; do not raise external capital until $15K MRR or enterprise pipeline requires it
- B4: Hit 50%+ gross margin by Q2 2027 (single-GPU self-host, $0.01-0.03/page cost)
- B5: 1 named case study with quantified time/cost savings in Q1 2027

### User goals
- U1: Extract a 30-page rent roll in under 30 seconds with ≥ 0.90 line-item F1
- U2: Drop in a custom schema (Pydantic) and have the system adapt without re-engineering
- U3: See exactly which fields failed validation and why
- U4: Run locally without sending their lease PDFs to a third party
- U5: Bulk-extract 1,000 docs overnight on a single machine

### Technical goals
- T1: Per-field F1 ≥ 0.80 on the drew-ipp invoice-extraction-benchmark (Phase 0 gate)
- T2: Per-field F1 ≥ 0.85 on a 30-doc Lease Addendum golden set (Phase 1 gate)
- T3: Per-field F1 ≥ 0.90 on a 60-doc combined golden set (Phase 2 gate)
- T4: p95 latency < 30 sec/doc on M-series Mac (local), < 5 sec/doc on 1× A100 (server)
- T5: Cost ≤ $0.03/doc at hosted-API scale (single A100, batched, GPU-seconds logged)
- T6: Schema validation passes on 95%+ of extractions
- T7: 99.5% crash-free extractions (every error returns a structured response, never a traceback)

## 5. Personas

### Primary: Sarah — Operations Manager at a 200-unit property management firm
- **Demographics:** 32, Atlanta GA, manages 3 properties, 8-person team. Non-technical. Uses AppFolio daily.
- **Goals:** Get accurate data from vendor invoices, lease addenda, and rent rolls into AppFolio without copy-paste.
- **Pain points:** Spends 6 hours/week typing invoice data into AppFolio. New lease addenda sit in her inbox for 2-3 days before processing.
- **Success metrics:** "My Friday afternoons are mine again." Less than 5 minutes per invoice processing.
- **Buying behavior:** Will pay $100-300/mo if it saves her 6 hours/week ($50/hour loaded cost).

### Secondary: David — Solo Broker / Investor with a 12-unit portfolio
- **Demographics:** 45, runs his own boutique brokerage in Phoenix. Tech-comfortable, uses Excel.
- **Goals:** Pull comps from 200-page lender statements quickly; underwrite deals in Excel.
- **Pain points:** Manually builds rent rolls for lenders, takes 2-3 days per deal. Misses clauses buried in long lease addenda.
- **Success metrics:** "I can underwrite a 12-unit deal in 2 hours, not 2 days."
- **Buying behavior:** Will pay $50-100/mo or pay-per-doc ($0.50-2/doc). Self-serve only; will not do a sales call.

### Tertiary: Priya — Data Engineer at a small PropTech startup (design-partner aspirant)
- **Demographics:** 29, builds internal tools at a 20-person PropTech that aggregates landlord data. Technical.
- **Goals:** Pipeline 5,000 lease PDFs/month into a Postgres database. Currently uses LlamaExtract API at $1,500/month.
- **Pain points:** API bills are unpredictable, accuracy on unusual lease clauses (e.g. percentage rent) is poor, no way to inspect failures.
- **Success metrics:** Cost reduction to <$500/month, accuracy ≥ 0.90 on lease fields, ability to inspect/log every extraction.
- **Buying behavior:** Will pay $500+/mo for self-host + observability. Will want to talk to the founder before committing.

## 6. User Research

### Hypotheses (to validate)
- H1: Property managers will adopt a CLI/Library tool that produces JSON they can paste into Excel/their PMS
- H2: Schema accuracy on vendor invoices is more important than perfect recall; operators tolerate review queues
- H3: Local-first is a real differentiator for privacy-sensitive operators (lease docs often contain tenant PII)
- H4: Per-field confidence numbers are a 10× UX improvement over binary pass/fail
- H5: The 30-doc golden set we hand-label will be the moat — competitors don't have domain-specific sets

### Validation plan
- V1: 15 user interviews (3 property managers, 5 solo brokers, 2 PropTech data engineers, 5 real-estate tech-curious)
- V2: 7-day diary study with 3 operators using the CLI on real docs
- V3: Landing-page test with the accuracy benchmark + a "save $50K/year" calculator
- V4: 100-person survey via BiggerPockets / r/realestate for willingness-to-pay
- V5: 1 paid design-partner pilot with measurable before/after metrics

### Key findings (as they emerge)
- Will populate in subsequent PRD versions

## 7. Scope Definition

### In Scope (v1, Phase 0 + Phase 1)
- Benchmark harness with reproducible scoring on 7 public datasets
- CLI: `realestate-bench extract --schema <pydantic> --doc <pdf> --out <json>`
- One schema in Phase 1: **Lease Addendum** (dates, rent, deposit, escalation, renewal)
- Pydantic v2 schema validation + business-rule validation (totals math, date sanity)
- Per-field confidence scores; auto-accept or queue threshold
- Local-first (no API key required); single-binary `pip install realestate-bench`
- 30-doc Lease Addendum golden set (hand-labeled)
- Per-field F1 ≥ 0.85 on the golden set

### In Scope (v2, Phase 2)
- Second schema: **Rent Roll** (units, sqft, rent/sqft, lease dates, escalation)
- Field-level confidence calibration using the AUGRC metric (ExtractBench)
- OCR fallbacks: detect low-confidence text regions, retry with second OCR engine, vote
- PDF + image + scanned-photo input
- Golden-set regression gate: every commit must pass on 60-doc combined golden set
- Extraction trace logs (model, OCR, latency, cost-equivalent) → JSONL

### In Scope (v3, Phase 3)
- FastAPI hosted service: `POST /v1/extract`
- API-key auth with per-key rate limit + usage metering
- Pricing: $0.05/page, $50/mo minimum
- Landing page with public benchmark leaderboard
- 1 paid design-partner LOI

### Out of Scope (v1, v2, v3)
- Multi-tenant web UI (Phase 4)
- Schema marketplace / community contributions (Phase 4)
- Multi-document reasoning (Phase 5)
- ERP/PMS integrations (Phase 5)
- SOC2 Type I certification (only when enterprise deal requires)
- Replacing Yardi/AppFolio/Buildium (never)
- Fully autonomous destructive actions
- Mobile apps
- Tenant-facing features

### Future Scope (v4+, post-Phase 3)
- Schema marketplace with PR workflow
- Per-schema leaderboard (accuracy on golden set, latency, cost)
- Web UI: upload doc + pick schema → see extracted JSON with confidence coloring
- Human-in-loop store: corrections become eval data → fine-tune future model versions
- Multi-doc reasoning: rent rolls across all properties of a portfolio
- ERP/PMS export (AppFolio, Buildium, Yardi CSV/JSON)
- SOC2 Type II (enterprise)
- Pricing tiers: Starter $99/mo, Pro $499/mo, Enterprise custom
- Optional integration into RealEstate365 as a building block

## 8. User Journey

End-to-end journey for the primary persona (Sarah, ops manager):

```
Awareness
  ↓ (sees a Reddit post / BiggerPools thread with a benchmark chart)
  ↓ "ExtractBench shows RealEstate-Bench beats NuExtract on lease docs"
Landing Page Visit
  ↓ (reads the public benchmark + accuracy numbers + 1-min demo video)
  ↓ "Wait, this is local-first? No API key needed?"
Install
  ↓ (pip install realestate-bench)
  ↓ "Done. Now what?"
First-Run Onboarding
  ↓ (realestate-bench init → wizard picks 1 schema, loads example doc, runs it)
  ↓ "OK, that's the JSON I would have typed for 6 hours."
Try on Real Doc
  ↓ (realestate-bench extract --schema lease_addendum --doc lease.pdf --out extracted.json)
  ↓ "85% of fields right. 3 flagged for review. Took 12 seconds."
  ↓ (manually fixes the 3 → corrections logged → golden set grows)
Weekly Use
  ↓ (Friday: bulk-runs 30 invoices, opens the review queue, fixes the 4-5 flagged)
  ↓ "I got my Friday back."
Scaling
  ↓ (subscribes to Pro plan, gets hosted API for the team)
  ↓ "The team lead uses it for rent rolls; I use it for invoices."
Referral
  ↓ (mentions it in a BiggerPools thread; founders reach out for design-partner pilot)
```

Activation target: first useful extraction within 5 minutes of install.
Retention target: week-4 retention ≥ 40% (returns at least weekly).

## 9. Functional Requirements

Each FR has format: `FR-XXX [Priority] — Title` followed by user story + acceptance criteria + links.

### FR-001 [Must] — Document ingestion
**As a** Sarah (operator), **I want to** run the CLI on a PDF, image, or scanned photo, **so that** I can extract without manual conversion.

**Acceptance criteria:**
- [ ] `realestate-bench extract --doc <path>` accepts: PDF (digital), PDF (scanned), JPEG, PNG, TIFF
- [ ] MIME detection from extension + magic bytes (not just extension)
- [ ] File size limit: 100 MB per doc, configurable via `--max-size`
- [ ] Streaming input for large PDFs (no full in-memory load)
- [ ] Failed ingest returns a structured error code (NOT_FOUND, UNSUPPORTED, TOO_LARGE)
- [ ] SHA-256 hash logged per ingestion (for caching + golden-set dedup)

**Tested in:** BACKLOG US-101 to US-105 (Phase 1)

### FR-002 [Must] — Schema declaration
**As a** developer (Priya), **I want to** define a Pydantic v2 schema, **so that** the extractor returns validated JSON I can pipe into my data pipeline.

**Acceptance criteria:**
- [ ] `--schema <python.path.ClassName` or `--schema-file <schema.py>`
- [ ] Pydantic v2 models with type hints, field descriptions, optional `Field(...)` constraints
- [ ] Schema fields can be primitive types, nested models, lists, optional, enums
- [ ] Schema supports `description=` per field; descriptions are included in the prompt to the model
- [ ] Schema versioning: every extraction records the schema hash in the output
- [ ] Schema validation errors are caught and returned with field path + reason
- [ ] Standard library of pre-built schemas ships in the box: `LeaseAddendum`, `RentRoll`, `VendorInvoice`, `LenderStatement`

**Tested in:** BACKLOG US-110 to US-115 (Phase 1)

### FR-003 [Must] — Extraction execution
**As a** user, **I want to** run extraction against a document + schema, **so that** I get structured JSON.

**Acceptance criteria:**
- [ ] `realestate-bench extract --doc <path> --schema <id> --out <file>`
- [ ] Calls the selected model via a swappable backend (vLLM, llama.cpp, HTTP for hosted)
- [ ] Handles model timeouts gracefully (returns TIMEOUT, not crash)
- [ ] Logs: model name, schema hash, doc hash, latency, GPU-seconds, confidence, per-field timing
- [ ] Default model is configurable via `realestate-bench config set model <id>`
- [ ] Optional `--no-cache` flag bypasses the doc+schema result cache
- [ ] Returns validated JSON conforming to the Pydantic schema
- [ ] Reports per-field confidence in the output (additive `confidence` field per leaf)

**Tested in:** BACKLOG US-120 to US-126 (Phase 1)

### FR-004 [Must] — Business-rule validation
**As a** Sarah (operator), **I want to** have the system catch arithmetic and date errors, **so that** I don't have to manually verify totals math.

**Acceptance criteria:**
- [ ] Built-in validators registered per schema: `totals_math` (sum of line items = subtotal), `date_sanity` (effective < expiration), `currency_normalize` (consistent currency code)
- [ ] Each validation result has: pass/fail, severity, evidence (which fields, what values, what was expected)
- [ ] Validators can be customized via `@validator("field_name")` Pydantic decorators
- [ ] Failed validation marks the doc for review (does not fail the extraction)
- [ ] Validation rules are documented in `docs/VALIDATORS.md`

**Tested in:** BACKLOG US-130 to US-134 (Phase 1)

### FR-005 [Must] — Confidence scoring + routing
**As a** user, **I want to** see per-field confidence and route low-confidence fields to review, **so that** I spend human time only where it's needed.

**Acceptance criteria:**
- [ ] Every extracted field has a confidence score in [0.0, 1.0]
- [ ] Routing threshold: ≥ 0.90 = auto-accept; < 0.90 = review queue
- [ ] Threshold is configurable: `--confidence-threshold 0.85`
- [ ] `--review-queue <file.jsonl>` writes low-confidence extractions for later review
- [ ] Confidence is calibrated using ExtractBench's AUGRC metric
- [ ] Logs include `confidence_mean` and `confidence_p5` per doc

**Tested in:** BACKLOG US-140 to US-145 (Phase 1)

### FR-006 [Should] — Caching + idempotency
**As a** developer, **I want to** re-run extraction cheaply on the same (doc, schema) pair, **so that** bulk pipelines don't pay double.

**Acceptance criteria:**
- [ ] Result cache: SHA-256(doc) + SHA-256(schema) → cached JSON
- [ ] Default cache dir: `~/.cache/realestate-bench/results.db` (SQLite)
- [ ] `--no-cache` flag bypasses
- [ ] `realestate-bench cache clear` empties the cache
- [ ] Cache hits return in <100ms with a log entry `cache_hit=True`

**Tested in:** BACKLOG US-150 to US-152 (Phase 2)

### FR-007 [Should] — Bulk + batch execution
**As a** Sarah with 200 invoices/month, **I want to** extract from many docs at once, **so that** I can run Friday's batch in one command.

**Acceptance criteria:**
- [ ] `realestate-bench batch --docs <glob> --schema <id> --out <dir>`
- [ ] Parallel execution with `--concurrency 4`
- [ ] Progress bar with ETA
- [ ] Per-doc results as `extracted/<doc_id>.json`; per-run summary as `summary.json`
- [ ] Failures don't abort the batch (each doc is independent)

**Tested in:** BACKLOG US-150 (golden set authoring), US-151 (golden test runner), US-152 (CI integration), US-220 (batch command), US-170/US-171 (smoke + CLI test command). See ARCH §2.2 (ingest), §5 (concurrency model), §10 (performance targets).

### FR-008 [Should] — Golden-set regression testing
**As a** developer (ksjswaroop), **I want to** every commit to be tested against the golden set, **so that** regressions are caught before merge.

**Acceptance criteria:**
- [ ] `realestate-bench test --golden bench/golden/v1/`
- [ ] Per-field F1, exact-match, IoU metrics computed
- [ ] Threshold gate: `tests/test_golden.py` fails CI if F1 drops > 2% from baseline
- [ ] Result report in `bench/results/<commit>/report.md`
- [ ] CI integration documented in `docs/CI.md`

**Tested in:** BACKLOG US-170 to US-174 (Phase 1 gate + Phase 2)

### FR-009 [Could] — Logging + observability
**As a** Priya, **I want to** inspect every extraction's trace, **so that** I can debug failures.

**Acceptance criteria:**
- [ ] Per-extraction JSONL log: `{ts, doc_hash, schema_hash, model, latency_ms, gpu_seconds, confidence_mean, errors[]}`
- [ ] Structured logs (JSON) for ingestion pipelines
- [ ] `--log-format json|text` toggle
- [ ] Optional OpenTelemetry export (Phase 3 hosted)

**Tested in:** BACKLOG US-180 to US-182 (Phase 1)

### FR-010 [Could] — Schema introspection
**As a** user, **I want to** see what schemas are available and their field documentation, **so that** I can pick the right one.

**Acceptance criteria:**
- [ ] `realestate-bench schemas list` shows all built-in schemas with field counts
- [ ] `realestate-bench schemas show lease_addendum` dumps the schema + per-field docs
- [ ] CLI help for each schema's intended document type and example doc

**Tested in:** BACKLOG US-190 to US-192 (Phase 1)

### FR-011 [Must] — Desktop app shell
**As a** real-estate professional, **I want to** run RealEstate-Bench as a local desktop app on my laptop, **so that** I can extract structured data from leases and contracts without uploading them to a hosted service.

**Acceptance criteria:**
- [ ] Tauri 2 app launches on macOS, Windows, and Linux from a single installer (`.app`, `.msi`, `.AppImage`)
- [ ] App window opens in < 3 seconds on an M-series Mac
- [ ] No network calls at runtime (verified via Wireshark during smoke test)
- [ ] Bundle size < 100 MB (Tauri shell + Needle binary + Python sidecar)
- [ ] All extraction work happens in a local Python sidecar (`realestate-extract`)

**Tested in:** BACKLOG US-600 to US-604 (Phase 1.5)

### FR-012 [Must] — Drag-and-drop document ingestion
**As a** user, **I want to** drop a PDF, JPEG, PNG, or TIFF into the app, **so that** I can extract data from it without typing a file path.

**Acceptance criteria:**
- [ ] Drag a file onto the app window → ingestion starts
- [ ] Supported types: PDF, JPEG, PNG, TIFF, HEIC (auto-convert via Pillow)
- [ ] Files larger than 100 MB show a warning but are still processed
- [ ] Ingestion progress is shown (page-by-page for multi-page PDFs)
- [ ] Failed ingestions show a clear error (corrupt, encrypted, unsupported)

**Tested in:** BACKLOG US-605 to US-610 (Phase 1.5)

### FR-013 [Must] — On-device extraction with cactus-needle
**As a** user, **I want to** run extraction locally against a 45M-parameter Needle model, **so that** my documents never leave the machine.

**Acceptance criteria:**
- [ ] Pick a schema from a built-in list (lease_addendum, invoice, custom YAML)
- [ ] Extraction runs in a Python sidecar (`realestate-extract`); the sidecar returns the JSON + per-field confidence + latency
- [ ] Model binary (~14 MB) is bundled with the app; no download on first use
- [ ] Mean confidence per extraction is shown; fields with confidence < 0.7 are flagged for review
- [ ] 5-page lease extracts in < 30 seconds on an M-series Mac
- [ ] Pydantic validation always passes (grammar-constrained decode guarantees this)

**Tested in:** BACKLOG US-611 to US-618 (Phase 1.5)

### FR-014 [Must] — Per-field confidence display
**As a** user, **I want to** see a confidence score for every extracted field, **so that** I know which fields to verify by hand.

**Acceptance criteria:**
- [ ] Each field shows a number 0-1 next to its value
- [ ] Background color: green ≥ 0.7, yellow 0.4-0.7, red < 0.4
- [ ] Clicking a field shows the model's "reasoning trace" (which parts of the source text it pulled from)
- [ ] Mean confidence for the whole extraction is shown in the header
- [ ] Low-confidence fields (red) are listed in a "needs review" panel

**Tested in:** BACKLOG US-619 to US-623 (Phase 1.5)

### FR-015 [Must] — Edit fields inline
**As a** user, **I want to** edit any extracted field, **so that** I can correct the model's mistakes without leaving the app.

**Acceptance criteria:**
- [ ] Each field is an editable input
- [ ] Edits are stored in the `edits` table with timestamp, old value, new value
- [ ] Editing marks the field as "user-verified" (overrides confidence for downstream review)
- [ ] Edits are exportable as a separate "corrections" JSON (used as eval data later)

**Tested in:** BACKLOG US-624 to US-627 (Phase 1.5)

### FR-016 [Must] — Export to Excel
**As a** user, **I want to** export extractions to an Excel file (one sheet per document, one row per field), **so that** I can share them with non-technical stakeholders.

**Acceptance criteria:**
- [ ] Click "Export Excel" → native save dialog → .xlsx file written
- [ ] One sheet per document; first row is field names; one row per field/value
- [ ] Confidence column included (0-1) so reviewers can sort by lowest confidence first
- [ ] Edits are reflected in the export (latest value wins)
- [ ] File opens cleanly in Microsoft Excel and LibreOffice Calc (validated with real files)

**Tested in:** BACKLOG US-628 to US-632 (Phase 1.5)

### FR-017 [Must] — Export to local SQLite database
**As a** user, **I want to** have all extractions stored in a local SQLite database, **so that** I can query them later or hand them off to a data analyst.

**Acceptance criteria:**
- [ ] Every extraction is auto-saved to `app.db` immediately on success
- [ ] DB schema matches ARCHITECTURE §3.5 (documents, extractions, edits)
- [ ] FTS5 full-text search across all extractions works
- [ ] "Open DB folder" button reveals the path in Finder/Explorer
- [ ] DB is portable: copy `app.db` to another machine and the schema works

**Tested in:** BACKLOG US-633 to US-637 (Phase 1.5)

### FR-018 [Could] — Export to JSON / CSV
**As a** developer, **I want to** export extractions to JSON and CSV, **so that** I can pipe them into my own data pipeline.

**Acceptance criteria:**
- [ ] "Export JSON" → single .json file with all selected extractions
- [ ] "Export CSV" → flat table with columns = union of all fields across extractions
- [ ] Both formats include confidence and timestamp metadata

**Tested in:** BACKLOG US-638 to US-640 (Phase 1.5)

## 10. Feature Specifications

### Feature: Document Ingestion (FR-001)
- **Overview:** Accept PDF/JPEG/PNG/TIFF, detect MIME, compute SHA-256, stream-read where possible.
- **User stories:** FR-001
- **Workflow:** Detect → Hash → Stream-read → Pre-process (deskew, denoise for photos) → Send to next stage
- **Inputs:** File path or stdin
- **Outputs:** Normalized in-memory representation: `{"pages": [...], "text": "...", "metadata": {...}}`
- **Errors:** NOT_FOUND, UNSUPPORTED, TOO_LARGE, CORRUPT
- **Edge cases:** Encrypted PDFs (return PASSWORD_REQUIRED), forms with XFA (warn), HEIC (auto-convert via Pillow if available)
- **Permissions:** Least privilege; never read outside the input path

### Feature: Schema-driven Extraction (FR-002 + FR-003)
- **Overview:** Take a Pydantic schema, a document, and a model backend; produce validated JSON with per-field confidence.
- **User stories:** FR-002, FR-003
- **Workflow:** Load schema → Build prompt with field descriptions → Call model → Parse response → Validate → Score confidence → Return JSON
- **Inputs:** Schema (Pydantic class or file), Document (normalized representation), Model config
- **Outputs:** `{"data": {...validated JSON...}, "confidence": {...per-field...}, "validation": {...rule results...}, "trace": {...latency, tokens, model...}}`
- **Errors:** SCHEMA_INVALID, MODEL_TIMEOUT, MODEL_ERROR, VALIDATION_FAILED
- **Edge cases:** Empty pages, multi-language docs (warning only), very long docs (chunk + merge)
- **Permissions:** Model files loaded from `~/.cache/realestate-bench/models/` or `--model-path`

### Feature: Validation + Confidence Routing (FR-004 + FR-005)
- **Overview:** Apply business rules and confidence threshold to determine auto-accept vs review queue.
- **User stories:** FR-004, FR-005
- **Workflow:** Run validators → Compute per-field confidence → Compare to threshold → Mark fields/destinations
- **Inputs:** Extraction result, schema, validation rules, confidence threshold
- **Outputs:** Extraction result augmented with `"needs_review": bool`, `"review_reasons": [...]`, and (if batch) a `--review-queue` JSONL file
- **Errors:** Validator errors don't fail extraction; they're recorded with severity
- **Edge cases:** Confidence values near boundary (use hysteresis)

### Feature: Golden-set Regression (FR-008)
- **Overview:** Run the model on a held-out, hand-labeled set; compute per-field F1; gate commits on regressions.
- **User stories:** FR-008
- **Workflow:** Load golden set → Run extraction → Score per-field F1 → Compare against `bench/baseline.json` → Pass/fail
- **Inputs:** Golden set directory `bench/golden/v*/`, baseline file
- **Outputs:** Markdown report `bench/results/<date>/report.md`, JSON metrics `bench/results/<date>/metrics.json`
- **Errors:** Missing test dir, baseline mismatch (warn but don't fail)
- **Edge cases:** Schema evolution: golden set must be re-baselined when schema changes

## 11. User Flows

### Happy path (CLI)
```
$ realestate-bench extract \
    --doc leases/addendum_42.pdf \
    --schema lease_addendum \
    --out out/addendum_42.json

[INFO] Detected: PDF (digital), 4 pages, 38KB
[INFO] Hash: a3f4e2...
[INFO] Model: NuExtract3-4B (local)
[INFO] Schema: lease_addendum v1.0 (sha: 8e0d...)
[INFO] Extraction complete in 11.2s

[OUTPUT]
{
  "data": {
    "lease_id": "LA-2026-0042",
    "tenant": "Acme Corp",
    "premises": "Unit 4B, 123 Main St",
    "effective_date": "2026-01-01",
    "expiration_date": "2028-12-31",
    "monthly_rent": {"amount": 4500.00, "currency": "USD"},
    "deposit": {"amount": 9000.00, "currency": "USD"},
    "escalation": {"type": "fixed", "annual_pct": 3.0},
    "renewal_option": "Two 5-year options at fair market"
  },
  "confidence": {
    "lease_id": 0.98,
    "tenant": 0.95,
    "premises": 0.92,
    "effective_date": 0.97,
    "expiration_date": 0.97,
    "monthly_rent.amount": 0.94,
    "monthly_rent.currency": 0.99,
    "deposit.amount": 0.91,
    "deposit.currency": 0.99,
    "escalation.type": 0.88,
    "escalation.annual_pct": 0.85,
    "renewal_option": 0.72
  },
  "validation": {
    "totals_math": "PASS",
    "date_sanity": "PASS (effective < expiration)",
    "currency_normalize": "PASS"
  },
  "needs_review": false,
  "review_reasons": [],
  "trace": {
    "model": "NuExtract3-4B",
    "schema_hash": "8e0d...",
    "doc_hash": "a3f4...",
    "latency_ms": 11200,
    "tokens_in": 4521,
    "tokens_out": 187
  }
}
```

### Happy path (batch)
```
$ realestate-bench batch \
    --docs "invoices/*.pdf" \
    --schema vendor_invoice \
    --out out/ \
    --concurrency 4

[INFO] 47 documents matched
[INFO] Running 4 parallel workers...

  ████████████████████████████████░░░░  44/47 (00:02:13)

[INFO] 47 extractions complete
[INFO] 41 auto-accepted, 6 in review queue
[INFO] Review queue: out/review_queue.jsonl (6 items)

$ cat out/review_queue.jsonl | head -1
{
  "doc": "invoices/inv_2026_09_044.pdf",
  "needs_review": true,
  "review_reasons": ["monthly_rent.confidence 0.78", "deposit.amount validation failed"]
}
```

### Exception path: model timeout
```
$ realestate-bench extract --doc huge_500page.pdf --schema rent_roll
[ERROR] MODEL_TIMEOUT after 60s
[INFO] Try: increase --timeout, use a smaller model, or split the doc
[INFO] Cached result not available for this (doc, schema) pair
$ echo $?
1
```

### Recovery path: cache hit after retry
```
$ realestate-bench extract --doc leases/addendum_42.pdf --schema lease_addendum --no-cache
[INFO] Cache disabled, running extraction... (12.1s)
$ realestate-bench extract --doc leases/addendum_42.pdf --schema lease_addendum
[INFO] Cache hit for (doc_hash, schema_hash) → returning cached result in 0.04s
```

## 12. Screen Requirements

CLI has no GUI; the "screens" are command outputs and the (future) web UI.

### CLI surface
- `extract` — single-doc extraction (FR-001 to FR-005)
- `batch` — multi-doc extraction (FR-007)
- `test` — golden-set regression (FR-008)
- `schemas list/show` — schema introspection (FR-010)
- `cache clear/list` — cache management (FR-006)
- `config get/set` — configuration
- `init` — first-run wizard (sets model, downloads weights, runs example)

### First-run wizard (init)
States: welcome → model selection → example doc run → success / error → ready

### (Future) Web UI — minimal viable
- Upload page (drag/drop PDF, pick schema)
- Progress (real-time, with model name + ETA)
- Result page (JSON tree, color-coded confidence, validation results)
- Review queue (low-confidence flagged, click to fix, save corrections)
- History (recent extractions, with re-run / delete)
- Settings (model, threshold, API key for hosted)

## 13. UX Requirements

### Usability principles
- **CLI first, GUI second.** Operators who try the CLI convert to paid 3× more than GUI-first users (industry pattern).
- **Progressive disclosure.** Default output is clean JSON. `--verbose` adds trace. `--debug` adds everything.
- **Plain language errors.** No "AssertionError: list index out of range" — instead "MODEL_TIMEOUT: extraction took longer than 60s. Try --timeout 120 or use a smaller model."
- **Reversible actions.** Every extraction can be re-run from the trace.
- **Visible provenance.** Every result shows the model + schema version + doc hash. Users can always verify "where did this number come from."

### Accessibility
- CLI: `--help` is comprehensive and discoverable via `man realestate-bench` (generated from typer/click)
- Output: `--output-format json|yaml|text` — text mode is human-readable
- Colors: optional, auto-disabled if NO_COLOR env or non-TTY output
- Screen reader: structured output modes preserve semantic structure

### Desktop / server parity
- Same CLI runs identically on macOS (M-series), Linux (x86_64 CUDA), Linux (x86_64 CPU), and (best-effort) Windows
- Backends: vLLM (CUDA), llama.cpp (Metal/CPU/CUDA), HTTP (remote hosted)

## 14. Design System

CLI is mostly textual. Design tokens map to:

- **Color codes** (auto-disabled on NO_COLOR or non-TTY):
  - GREEN: pass / success / auto-accepted
  - YELLOW: warning / needs review
  - RED: error / failed
  - CYAN: info / status
  - DIM: secondary text (file paths, hashes)
- **Typography:** ASCII only; no Unicode boxes (CLI portability)
- **Spacing:** 2-space indent in JSON; aligned columns in tables
- **Iconography:** Emoji `[INFO]`, `[WARN]`, `[ERROR]`, `[OK]`
- **Motion:** Progress bars use `tqdm` (no custom animation)

(Future Web UI design system documented in `docs/DESIGN_SYSTEM.md`.)

## 15. Information Architecture

CLI command tree:
```
realestate-bench
├── init                 # First-run wizard
├── extract              # Single-doc extraction
│   ├── --doc <path>
│   ├── --schema <id>
│   ├── --out <file>
│   ├── --model <id>
│   ├── --confidence-threshold <float>
│   ├── --timeout <seconds>
│   ├── --no-cache
│   ├── --log-format json|text
│   └── --verbose / --debug
├── batch                # Multi-doc extraction
│   ├── --docs <glob>
│   ├── --schema <id>
│   ├── --out <dir>
│   └── --concurrency <int>
├── test                 # Golden-set regression
│   ├── --golden <dir>
│   └── --baseline <file>
├── schemas              # Schema management
│   ├── list
│   └── show <id>
├── cache                # Cache management
│   ├── list
│   └── clear
├── config               # Configuration
│   ├── get <key>
│   └── set <key> <value>
└── --version / --help
```

## 16. Data Requirements

### Entities

| Entity | Purpose | Key fields |
|---|---|---|
| `Document` | Input doc representation | id, path, mime, sha256, page_count, file_size, ingested_at |
| `Schema` | Pydantic schema + metadata | id, version, sha256, field_definitions, validators |
| `Extraction` | One extraction result | id, doc_id, schema_id, model, data, confidence, validation, trace, created_at |
| `ReviewQueueItem` | Low-confidence item | id, extraction_id, fields[], reasons[], resolved_at, resolver, correction |
| `GoldenDoc` | Hand-labeled golden example | id, doc_path, schema_id, ground_truth, labeled_by, labeled_at |
| `GoldenResult` | Regression result | id, run_id, golden_id, per_field_f1, overall_pass |
| `ModelConfig` | Backend model config | id, name, backend, weights_path, vram_gb, parameters |
| `Cache` | Idempotency cache | key (sha256 doc+schema), value (extraction JSON), created_at |

### Data dictionary
- `id`: UUID v4
- `sha256`: hex string
- `confidence`: float [0.0, 1.0], NaN not allowed
- `latency_ms`: integer ≥ 0
- `tokens_in`/`tokens_out`: integer ≥ 0
- `created_at`: ISO-8601 UTC timestamp

### Storage (Phase 1)
- SQLite at `~/.local/share/realestate-bench/state.db` (extractions, cache, configs)
- No remote DB until hosted API (Phase 3)

## 17. Database Design

### Schema (SQLite)

```sql
CREATE TABLE documents (
  id TEXT PRIMARY KEY,  -- UUID
  path TEXT NOT NULL,
  mime TEXT NOT NULL,
  sha256 TEXT NOT NULL UNIQUE,
  page_count INTEGER,
  file_size INTEGER,
  ingested_at TEXT NOT NULL
);

CREATE TABLE schemas (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  version TEXT NOT NULL,
  sha256 TEXT NOT NULL,
  source_path TEXT,  -- optional, if loaded from file
  field_count INTEGER,
  registered_at TEXT NOT NULL,
  UNIQUE (name, version)
);

CREATE TABLE extractions (
  id TEXT PRIMARY KEY,
  doc_id TEXT NOT NULL REFERENCES documents(id),
  schema_id TEXT NOT NULL REFERENCES schemas(id),
  model TEXT NOT NULL,
  data_json TEXT NOT NULL,  -- the validated JSON
  confidence_json TEXT NOT NULL,
  validation_json TEXT,
  trace_json TEXT,
  needs_review INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE INDEX idx_extractions_doc ON extractions(doc_id);
CREATE INDEX idx_extractions_schema ON extractions(schema_id);
CREATE INDEX idx_extractions_review ON extractions(needs_review) WHERE needs_review = 1;

CREATE TABLE cache (
  key TEXT PRIMARY KEY,  -- sha256(doc+schema)
  extraction_id TEXT NOT NULL REFERENCES extractions(id),
  created_at TEXT NOT NULL
);

CREATE TABLE golden_docs (
  id TEXT PRIMARY KEY,
  doc_id TEXT NOT NULL REFERENCES documents(id),
  schema_id TEXT NOT NULL REFERENCES schemas(id),
  ground_truth_json TEXT NOT NULL,
  labeled_by TEXT,
  labeled_at TEXT NOT NULL
);

CREATE TABLE golden_results (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  golden_id TEXT NOT NULL REFERENCES golden_docs(id),
  per_field_f1_json TEXT NOT NULL,
  overall_pass INTEGER NOT NULL,
  created_at TEXT NOT NULL
);
```

### Indexes / constraints
- `documents.sha256` UNIQUE (dedup)
- `schemas (name, version)` UNIQUE
- `cache.key` PRIMARY KEY
- `golden_results (run_id, golden_id)` UNIQUE

### Migration
- Initial migration is `001_initial.sql`; run on first `init`
- Schema version stored in `PRAGMA user_version`

## 18. API Requirements

### Local CLI (no HTTP)
Already documented in §15.

### Hosted API (Phase 3, FastAPI)

| Endpoint | Method | Auth | Purpose |
|---|---|---|---|
| `/v1/extract` | POST | API key | Single-doc extraction |
| `/v1/batch` | POST | API key | Multi-doc extraction |
| `/v1/schemas` | GET | API key | List available schemas |
| `/v1/schemas/{id}` | GET | API key | Show schema details |
| `/v1/usage` | GET | API key | Usage for current key |
| `/healthz` | GET | none | Liveness |

#### `POST /v1/extract`

Request:
```json
{
  "doc_url": "https://...",  // OR
  "doc_base64": "...",        // OR
  "doc_id": "...",            // (if previously uploaded)
  "schema_id": "lease_addendum",
  "confidence_threshold": 0.90,
  "review_queue": true
}
```

Response (200):
```json
{
  "extraction_id": "...",
  "data": {...},
  "confidence": {...},
  "validation": {...},
  "needs_review": false,
  "review_reasons": [],
  "trace": {"model": "...", "latency_ms": 11200, "cost_usd": 0.012}
}
```

Errors:
- 400 INVALID_REQUEST (missing field, wrong type)
- 401 UNAUTHORIZED (missing/invalid API key)
- 403 QUOTA_EXCEEDED
- 404 SCHEMA_NOT_FOUND
- 413 DOC_TOO_LARGE
- 422 VALIDATION_FAILED (Pydantic-level)
- 504 MODEL_TIMEOUT

#### Rate limits
- Default: 100 requests/min per key
- Burst: 200/min
- Per-doc size: 100 MB

#### Auth
- API keys: 32-byte random hex, prefix `rb_`
- Keys stored hashed (argon2id) in DB
- Per-key: rate limit, monthly quota, allowed schemas (optional)

## 19. AI Requirements

### Models evaluated (Phase 0)

| Model | Size | Context | Strengths | Weaknesses | Decision |
|---|---|---|---|---|---|
| NuExtract3-4B (Qwen3.5-4B) | 4B (fp16 ~8GB) | 262K | Strong on long docs, JSON-mode built-in | Higher VRAM, slower than smaller models | Strong candidate |
| ReaderLM-v2 (1.5B) | 1.5B (~3GB) | 512K | Very long context, HTML→JSON | HTML-in, JSON-out; some real-estate PDFs aren't HTML-like | Strong candidate |
| GLiNER2 (large-v1) | 486M (~1GB) | 8K | Tiny, CPU-friendly, fast | Encoder-only, no generation; entity tagging only | Fallback for entity subset |
| Needle 2 (45M) | 45M (~100MB) | 256-token sliding | Edge, on-device | Quality drop on long docs; schema-constrained only | Too small; exclude |

### Decision rule (Phase 0)
Pick the model with the best weighted score on the three eval sets:
```
score = 0.5 × ExtractBench_short_F1
      + 0.3 × invoice_extraction_benchmark_F1
      + 0.2 × real_estate_golden_set_F1
```
Subject to F1 ≥ 0.80 on `invoice_extraction_benchmark` (Phase 0 gate).

### Prompts
- Versioned prompt templates stored in `realestate_bench/prompts/` (one file per schema)
- Templates include: schema JSON, field descriptions, doc text, output-format example
- Tested against 10 injection-attack patterns (defense-in-depth)

### RAG
- Not used in Phase 1 — schemas are explicit, retrieval would add latency without value
- Considered for Phase 5 multi-doc reasoning (cross-referencing leases)

### Memory
- Not used in Phase 1 — extractions are stateless
- Considered for Phase 4 schema marketplace (preference tracking per schema author)

### Agent workflow
- Single-call per extraction in Phase 1 (no agent loop)
- Phase 2 may add a small loop for low-confidence retries

### Verification
- Deterministic checks: schema validation, business-rule validation, confidence range
- Probabilistic checks: per-field F1 on golden set (regression gate)
- Hallucination controls: schema-constrained output, JSON-mode model flag, post-hoc type checking

### Cost controls
- Default timeout per extraction: 60s
- Default max tokens out: 4,096
- Per-doc cost logged; configurable via `--max-cost-usd 0.10`

## 20. Security Requirements

| Concern | Mitigation |
|---|---|
| Untrusted PDF (malicious embedded JS) | Use `pypdf` (safe) over `PyPDF2`; sandbox in subprocess if needed |
| API key leak | Keys stored hashed (argon2id); never logged |
| PII in logs | Per-doc opt-out `--no-log-data`; redaction of common PII patterns (SSN, EIN, account #) |
| Doc exfiltration to hosted model | Local-first default; hosted only with explicit `--allow-cloud` |
| Model weight tampering | SHA-256 verification on download; signed weights from upstream (NuMind, Jina, etc.) |
| Cache poisoning | Cache key = SHA-256(doc) + SHA-256(schema); immutable entries |
| Code injection in schema | Pydantic v2 sandboxed; no `eval`/`exec`; safe by design |

Compliance posture (Phase 2+):
- **SOC2:** Phase 5 only, when an enterprise deal requires it
- **GDPR:** Local-first default; deletion via `realestate-bench cache clear` + state DB wipe
- **HIPAA:** Not designed for HIPAA workloads (out of scope)

## 21. Privacy Requirements

- **Data collection (CLI, local):** No remote calls except model weight download and (optional) hosted extraction
- **Data collection (hosted API, Phase 3+):** Extracted JSON + doc hash + per-field confidence + usage metrics; no raw doc text retained beyond 30 days
- **Consent:** First-run wizard asks: "May we collect anonymous usage metrics?" (opt-in)
- **Retention:** Raw docs cached for 30 days, then auto-deleted; user can delete via `cache clear`
- **Deletion:** `realestate-bench forget --doc <hash>` removes all traces
- **Telemetry:** Strict opt-in; default OFF; documented in `PRIVACY.md`

## 22. Performance Requirements

| Metric | Target | Measurement |
|---|---|---|
| Single-doc latency (Lease Addendum, 4 pages) | < 30s p95 on M-series Mac (local), < 5s p95 on A100 | Bench harness, repeated 30 runs |
| Batch throughput | ≥ 200 docs/hour on A100 | Bench harness, 100 docs in batch |
| Cache hit latency | < 100ms p99 | Bench harness |
| Memory (idle) | < 500 MB | `psutil` |
| Memory (extraction, A100) | < 16 GB VRAM | `nvidia-smi` |
| Memory (extraction, Mac M-series) | < 12 GB unified | `psutil` |
| Schema validation | < 50ms p99 per extraction | Bench harness |
| Confidence scoring | < 20ms p99 per field | Bench harness |
| Cold-start (model load) | < 60s | Bench harness |
| Warm-start (model already loaded) | < 1s | Bench harness |

## 23. Reliability Requirements

- **Fault tolerance:** Single-doc failure never crashes the CLI; always returns a structured response or error code
- **Retry logic:** Automatic retry (1×) on transient errors (network timeout, GPU OOM); no retry on validation failures
- **Circuit breakers:** None in Phase 1 (single backend); Phase 3 hosted may add per-region circuit breakers
- **Backup:** State DB is local; user responsible for backup. Documented in `OPERATIONS.md`
- **Recovery:** If state DB is corrupted, `realestate-bench init --recover` rebuilds from cached files

## 24. Observability Requirements

### Logs
- Structured JSON logs (default `--log-format text`)
- Levels: DEBUG, INFO, WARN, ERROR
- Per-extraction: model, schema, doc hash, latency, GPU-seconds, confidence mean/p5, errors
- Retention: rolling 7 days in `~/.local/share/realestate-bench/logs/`

### Metrics (Phase 3 hosted)
- Per-key: requests/min, errors/min, latency p50/p95/p99
- Per-model: requests/min, GPU-seconds/hour, cost-equivalent
- Per-schema: requests/min, auto-accept rate, review-queue rate

### Tracing (Phase 3 hosted)
- OpenTelemetry export
- Each extraction is a span; sub-spans for model call, validation, scoring

### Alerts (Phase 3 hosted)
- Error rate > 5% over 5 min → page on-call (PagerDuty)
- Latency p95 > 30s over 10 min → warn
- Cost > $X/day → warn

## 25. Integration Requirements

### Outbound
- Hugging Face Hub: model weight download (CLI only, first-run)
- (Phase 3) Optional: OpenTelemetry collector
- (Phase 5) Optional: AppFolio/Buildium/Yardi CSV/JSON export

### Inbound (Phase 3 hosted)
- Stripe: subscription billing + usage metering
- Auth: API keys (Phase 3); OAuth (Phase 4 if needed)

### No external services in Phase 1

## 26. Reporting Requirements

### Operator reports
- `realestate-bench report --last 30d` — number of docs extracted, auto-accept rate, review-queue size, cost total
- Output: Markdown table + JSON for piping

### Founder reports
- Weekly: GitHub stars, downloads, paying customers
- Monthly: MRR, churn, expansion, NPS (when N ≥ 10)

### Customer-facing (Phase 4)
- Per-schema leaderboard on landing page: accuracy on golden set, latency p95, cost/doc
- Updated weekly; historical trend

## 27. Analytics Requirements

### Events (CLI, opt-in)
- `extraction_completed` — model, schema, latency, confidence
- `extraction_failed` — error code
- `review_queue_item_resolved` — correction
- `schema_used` — schema_id
- `model_loaded` — model, load_time

### Funnels
- Install → init complete → first extraction → 10 extractions → Pro plan

### Retention
- Week-1 retention (returned at least once): target ≥ 60%
- Week-4 retention: target ≥ 40%
- NPS (when N ≥ 10): target ≥ 40

## 28. Testing Requirements

### Unit tests
- Schema validation (Pydantic edge cases)
- Confidence scoring math
- Validators (totals math, date sanity, currency normalize)
- Hash computation
- Cache key derivation
- CLI argument parsing

### Integration tests
- End-to-end on 3 known docs (golden set)
- Each model backend (mocked for hosted)
- Batch mode with concurrency
- Cache hit path

### Golden-set regression (the gate)
- `realestate-bench test --golden bench/golden/v1/`
- Per-field F1 thresholds (see PRD §9 FR-008)
- CI fails on regression > 2%

### Model evaluation tests
- Phase 0: full benchmark suite runs in < 2 hours on 1× A100
- Reported in `bench/results/<date>/BENCH_RESULTS.md`

### Security tests
- Malicious PDF (embedded JS, embedded scripts) → graceful failure
- Schema with malicious code → Pydantic sandbox catches
- Cache poisoning attempts → cache key verification catches

### Performance tests
- Latency benchmarks on reference hardware
- Memory profile (no leak over 100 extractions)
- Throughput (docs/hour)

## 29. Deployment Requirements

### Environments
- **Dev:** Local development (this PRD)
- **Test:** CI (GitHub Actions on PR)
- **Stage:** Phase 3 hosted (1× A100, $0.50-2/hr on Vultr/Lambda)
- **Prod:** Phase 3 hosted (1× A100 + autoscaler)

### CI/CD
- GitHub Actions on push:
  - Lint (ruff, black)
  - Unit tests (pytest)
  - Integration tests (3-doc golden)
  - Golden-set regression (per-field F1)
  - Build wheel + docker image
- On tag: publish to PyPI + push docker image to GHCR

### Rollback
- PyPI: pin version; `pip install realestate-bench==<previous>`
- Docker: previous image tag always available
- DB migrations: every migration is reversible (`alembic downgrade`)

## 30. Migration Requirements

- **Schema versioning:** `lease_addendum@1.0.0` → `lease_addendum@2.0.0`. Old extractions remain queryable.
- **Migration tool:** `realestate-bench migrate --from v1 --to v2` applies the transformation
- **User migration:** N/A (CLI tool, no user accounts in Phase 1)
- **Rollback:** Keep all schema versions; users opt in to new versions

## 31. Release Plan

### MVP (Phase 1, end of November 2026)
- CLI with `extract`, `batch`, `test`, `init`
- Lease Addendum schema (built-in)
- 30-doc Lease Addendum golden set
- Per-field F1 ≥ 0.85 on golden set
- Local-first; single-binary install
- Public GitHub repo
- `pip install realestate-bench` works

### Phase 2 (end of January 2027)
- Rent Roll schema (built-in)
- 60-doc combined golden set
- Per-field F1 ≥ 0.90 on combined set
- Confidence calibration (AUGRC)
- Batch mode with concurrency
- Regression gate in CI
- First Reddit/blog post announcing results

### Phase 3 (end of March 2027)
- Hosted FastAPI service
- 1× A100 self-hosted
- $0.05/page pricing
- Landing page with public benchmark
- 3-5 paying design partners
- $500-2,500 MRR

### Phase 4 (end of June 2027)
- Schema marketplace (GitHub repo + PR workflow)
- Web UI (upload doc + pick schema → result page)
- 10+ community schemas
- $10-25K MRR

### Phase 5 (end of Q4 2027)
- Multi-doc reasoning
- ERP/PMS export
- $1-10M ARR range (stretch)

## 32. Risk Assessment

| ID | Risk | Probability | Impact | Mitigation |
|----|------|-------------|--------|------------|
| R1 | Small models can't hit 0.80 F1 on invoice benchmark | Medium | High | Phase 0 evaluates 4 models with evidence; Plan B = route hard docs to a larger hosted model (NuExtract PRO, LlamaExtract) |
| R2 | Real-estate docs are too varied for one schema | High | Medium | Phase 1 ships one schema (Lease Addendum); Phase 2 adds Rent Roll; per-domain sub-schemas in Phase 4 |
| R3 | PDF extraction accuracy limits total accuracy | High | High | OCR fallback (PaddleOCR + Tesseract ensemble) in Phase 2; PDF text layer first, OCR fallback |
| R4 | GPU costs exceed $0.03 target at hosted scale | Medium | High | Model selection favors small/fast (NuExtract3 vs 2.0); batch size tuning; auto-scaler |
| R5 | Competitor with more resources (Reducto, NuMind) ships real-estate features | Medium | Medium | Speed-to-market + transparent benchmark + local-first; we don't try to outspend |
| R6 | Schema marketplace moderation becomes a burden | Low | Medium | Phase 4 only; Phase 1-3 curated schemas only |
| R7 | PII / tenant data leaks via logs | Low | Critical | Default no-log-data; opt-in telemetry; redaction of common PII patterns |
| R8 | Long context doesn't translate to long-doc reasoning | High | Medium | Build with chunked extraction from day 1; "model sees whole doc" is a Phase 4 nice-to-have |
| R9 | RealEstate365 timeline shifts; lose natural design partner | Medium | Low | Benchmark results + golden set are useful regardless; standalone product is viable without it |
| R11 | Legal liability for "wrong extraction" used in a deal | Low | Critical | Confidence threshold + review queue; clear disclaimer in docs; EULA excludes consequential damages |
| R12 | Vendor lock-in if NuExtract3 is the only viable choice | Medium | Medium | Architecture supports backend swapping; always test ReaderLM-v2 and GLiNER2 in CI |

## 33. Assumptions

- A1: At least one of {NuExtract3, ReaderLM-v2} can hit ≥ 0.80 F1 on invoice-extraction-benchmark without fine-tuning
- A2: 30-doc hand-labeled Lease Addendum set is sufficient to start; we can grow it incrementally
- A3: Pydantic v2 remains the dominant Python schema library
- A4: SQLite is sufficient for state until Phase 3 hosted (Postgres migration is straightforward)
- A5: Model weights fit on a single consumer GPU (≤ 24 GB VRAM) by Phase 1
- A7: RealEstate365 referral channel produces at least 1 design partner in Phase 3
- A8: A single founder (ksjswaroop) can ship Phase 0 + Phase 1 solo with AI agent assistance
- A9: vLLM remains the dominant local inference backend (or llama.cpp catches up)
- A10: Operators will tolerate a 5-minute install + first-run setup (versus SaaS signup in 30s)
- A11: PaddleOCR + Tesseract 5 + Surya remain competitive in 2026-2027

## 34. Constraints

- **Technical:** Must work on consumer hardware (M-series Mac, RTX 4090, RTX 3090); no data center GPUs for end users
- **Business:** Bootstrap-first; do not raise until $15K MRR or enterprise deal requires it
- **Legal:** Apache-2.0 for code (permissive, encourages adoption); CC-BY-4.0 for data and docs
- **Budget:** Personal bootstrap budget only; no external spend beyond domain + hosting (~$50/mo)
- **Time:** Founder time is the bottleneck; aim for 6-hour focused sessions per the user's working style
- **Compatibility:** Python 3.11+; macOS 14+, Ubuntu 22.04+, Windows 11+ (best effort)

## 35. Success Metrics

### Product metrics (12-month horizon)
- **Activated users (CLI):** ≥ 500 within 6 months of public launch
- **Week-4 retention:** ≥ 40%
- **Auto-accept rate:** ≥ 80% on golden set extractions
- **Schemas in registry:** ≥ 5 by Q4 2027

### Business metrics
- **MRR:** $500-$2,500 by Q1 2027; $10-25K by Q2 2027
- **Paying customers:** 5-25 design partners by Q1 2027
- **Churn:** < 5% monthly
- **Gross margin:** > 50% by Q2 2027

### Technical metrics
- **Per-field F1 (Lease Addendum golden set):** ≥ 0.85 in Phase 1, ≥ 0.90 in Phase 2
- **Per-field F1 (invoice-extraction-benchmark):** ≥ 0.80 (Phase 0 gate)
- **ExtractBench short F1:** Top 10 leaderboard bracket
- **p95 latency:** < 30s on M-series Mac, < 5s on A100
- **Crash-free rate:** ≥ 99.5%
- **Cost per doc:** ≤ $0.03 at hosted scale

### Adoption signals
- GitHub stars ≥ 500 within 3 months of public launch
- PyPI downloads ≥ 1,000/month within 3 months
- 3+ unsolicited community PRs in first 6 months
- 1+ RealEstate365 student/graduate referral in Phase 3

## 36. KPIs Dashboard

(Phase 3 hosted. Phase 1 = a CLI; dashboard is local Markdown report.)

### Founder weekly view
| KPI | Source | Target |
|---|---|---|
| Weekly downloads | PyPI | +10% WoW |
| GitHub stars | GitHub API | trending up |
| Golden-set F1 (latest) | bench/results | ≥ 0.85 |
| Open issues | GitHub API | < 30 |
| First-time users (CLI `init`) | local telemetry (opt-in) | +20% WoW |

### Operational view
| KPI | Source | Target |
|---|---|---|
| Per-doc latency p95 | logs | < 30s |
| Per-doc cost | logs | ≥ 0.03 |
| Review-queue rate | logs | < 25% |
| Model error rate | logs | < 1% |

### AI-specific KPIs
| KPI | Source | Target |
|---|---|---|
| Per-field F1 (golden set) | `realestate-bench test` | ≥ 0.85 (Phase 1), ≥ 0.90 (Phase 2) |
| AUGRC calibration | `bench/calibrate.py` | < 0.05 deviation |
| Hallucination rate | manual review on 50 random extractions | < 2% |
| Schema-conformance rate | logs | > 95% |

## 37. Open Questions

- OQ1: Will we need to fine-tune a model on real-estate docs, or are off-the-shelf models sufficient? (Phase 0 answer.)
- OQ2: Should the hosted API use serverless (Lambda + on-demand GPU) or always-on? (Decide before Phase 3.)
- OQ3: Schema marketplace: GitHub PR-based, or our own hosted registry? (Decide Phase 4.)
- OQ4: Do we need a real "app" (Tauri/PyQt) or is CLI + (later) web sufficient? (Defer.)
- OQ5: Web UI build vs buy? Build if Phase 4 takes off; defer otherwise.
- OQ6: Pricing: per-page, per-doc, per-month, or hybrid? (Phase 3 decision; current leaning $0.05/page + $50/mo min.)
- OQ7: Do we ship OCR bundled, or assume users have it? (Phase 2 decides; lean bundled for simplicity.)
- OQ8: Multi-language docs (Spanish lease addenda, etc.)? (Phase 4; for now, English-only with a warning.)

## 38. Appendices

### Appendix A — Dataset inventory
See `datasets/README.md` for the full manifest of benchmarks, their licenses, and what each tests.

### Appendix B — Model comparison sheet
See `bench/RESULTS.md` (populated after Phase 0 runs).

### Appendix C — Glossary
- **KIE:** Key Information Extraction (the academic term for "extract fields from docs")
- **OCR:** Optical Character Recognition (image → text)
- **IDP:** Intelligent Document Processing (the industry term for the whole category)
- **Schema:** A Pydantic v2 model declaring the fields and types of an extraction result
- **Golden set:** A hand-labeled held-out set of (doc, schema, ground_truth) used for regression testing
- **Per-field F1:** F1 score computed per field, then averaged. Different from per-document F1 (which counts a doc as right only if every field is right).
- **AUGRC:** Area Under Generalized Retention Curve (calibration metric from ExtractBench)

### Appendix D — References
- NuExtract blog: https://numind.ai/blog/nuextract3
- ReaderLM-v2 paper: https://arxiv.org/abs/2503.01130
- GLiNER2 repo: https://github.com/fastino-ai/gliner2
- ExtractBench: https://github.com/run-llama/extract-bench
- drew-ipp invoice benchmark: https://github.com/DrewKraken/invoice-extraction-benchmark
- CORD dataset: https://github.com/clovaai/cord
- SROIE dataset: https://github.com/rcabench/sroie
- FUNSD dataset: https://guillaumejaume.github.io/FUNSD/
- RealEstate365 (umbrella business): ksjswaroop bootstraps RealEstate365 (Avenue One IP → education → SaaS)

### Appendix E — Revision history
| Date | Ver | Author | Notes |
|------|-----|--------|-------|
| 2026-10-13 | 1.0 | ksjswaroop + Hermes | Initial PRD |

## 39. AI / Agent Architecture

(Phase 1 uses no agent; this section documents the design space for future phases.)

### Agent catalog (Phase 4+)
- **Extractor agent:** Runs the model, parses output, returns structured data
- **Validator agent:** Runs business rules (totals math, date sanity)
- **Confidence agent:** Computes per-field confidence (currently deterministic; could be LLM-based in Phase 5)
- **Review agent (Phase 5):** Suggests corrections for low-confidence fields (always human-approves)
- **Schema-evolution agent (Phase 5):** Detects new fields in docs that the schema doesn't have; suggests schema updates (requires PR + review)

### Memory model
- Phase 1: stateless extraction (no memory)
- Phase 3 (hosted): per-API-key usage history; per-schema accuracy history
- Phase 5: corrections accumulate into a per-(user, schema) memory used to fine-tune confidence calibration

### Tool registry
- `extract_pdf`, `extract_image`, `validate_totals`, `validate_dates`, `score_confidence`, `write_to_pms`

### Typed inter-agent events
- `ExtractionRequested`, `ExtractionCompleted`, `ValidationFailed`, `ConfidenceBelow`, `SchemaEvolutionSuggested`

### Resumability
- Phase 1: not needed (single-call extractions)
- Phase 5: long-running multi-doc jobs checkpoint to SQLite; resumable on crash

### Human approval callbacks
- Review queue items: human approves via CLI (`realestate-bench review resolve <id>`) or (Phase 4) web UI
- Schema-evolution suggestions: human approves via PR review
- Destructive schema changes: human approves explicitly

### Channel notifications
- (Phase 3) Optional Slack/Discord webhook for batch completion
- (Phase 3) Email digest for weekly usage summary

### Structured-output validation
- Pydantic v2 enforced at every agent boundary
- JSON-mode model flag ensures model output is parseable JSON
- Post-hoc JSON schema check before validation passes

### Provider/model abstraction
- Backend interface: `class ModelBackend` with `def extract(doc, schema) -> Extraction`
- Implementations: `VLLMBackend`, `LlamaCppBackend`, `HTTPBackend` (hosted)
- Swap via `--model <id>` or `realestate-bench config set model <id>`

### Trust/safety/cost controls
- Per-extraction cost cap: `--max-cost-usd 0.10`
- Per-extraction time cap: `--timeout 60s`
- Default deny for hosted API (no raw doc retention beyond 30 days)
- Review queue required for confidence < 0.70
- Schema-evolution suggestions require human PR review

---

*PRD v1.0 — Phase 0 gate: per-field F1 ≥ 0.80 on drew-ipp invoice-extraction-benchmark*
*RealEstate-Bench © 2026 ksjswaroop — Apache-2.0 (code), CC-BY-4.0 (data/docs)*
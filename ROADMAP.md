# realestate-bench - Roadmap

Document-extraction benchmarking and the extractor we'll build on top of it. Goal: a small model (or pipeline of small models) that turns pricing pages, contracts, leases and statements into validated structured JSON, accurate enough to trust in RealEstate365 workflows.

## Phase 0: Benchmark-driven model selection (October 2026)

### v0.1 - Pick a model with evidence
**Target: October 2026** *(in progress)*

**Done:**
- [x] 7 benchmark datasets pulled (3.5 GB): ExtractBench, SROIE, CORD, FUNSD, invoice-extraction-benchmark, invoice-extraction-dataset-v2, plus the NuExtract GitHub repo
- [x] README + manifest at `datasets/README.md`
- [x] Models shortlisted: NuExtract3 (4B VLM, 262K ctx), ReaderLM-v2 (1.5B, 512K ctx, HTML→JSON), GLiNER2 (486M, CPU, entity tagging), Needle 2 (45M, edge)

**Open:**
- [ ] Score all 4 candidate models on ExtractBench **short split (252 docs)** using their existing eval harness
- [ ] Score the same models on `01_invoice_extract_benchmark` (181 docs with answer keys) — this is the one that actually measures pricing/dates/line-items
- [ ] Score on 20 held-out docs from your own RealEstate source corpus (real contracts, lease addenda, statements)
- [ ] Pick the model with best weighted score: `0.5 × ExtractBench_F1 + 0.3 × invoice_F1 + 0.2 × your_data_F1`, with cost and latency as tiebreakers

**Success metrics:**
- A `BENCH_RESULTS.md` ranking the 4 models on per-field F1 (vendor, invoice_no, date, total, line_items) and cents/page
- A `MODEL_PICK.md` with a one-paragraph justification + reproducer script (`bench/run.py`)
- Decision logged: which model(s) ship, which are cut

**Out of scope for this phase:**
- Production API, UI, billing, auth, multi-tenancy

---

## Phase 1: Pipeline MVP (November 2026)

### v1.0 - End-to-end extraction on one document type
**Target: November 2026**

**Features:**
- [ ] CLI: `extract --schema <pydantic> --doc <pdf> --out <json>` — input is a PDF or image, a Pydantic schema, output is validated JSON
- [ ] One schema first: **lease addendum** (dates, rent, deposit, escalation clauses, renewal terms). RealEstate365's first concrete use case.
- [ ] Validation: Pydantic schema-enforced; post-extraction rules (date sanity, currency normalize, totals-math check, line-items-sum)
- [ ] Confidence per field; below threshold → flag for review
- [ ] Local-first (no API keys required); self-hosted model weights; single-binary install
- [ ] 30 example docs from one RealEstate source, hand-labeled, used as the held-out test set
- [ ] Per-field F1 ≥ 0.85 on the 30-doc set (vs 96.1% IPP claims on synth; beat your own baseline, not theirs)

**Stack choices to lock in this phase:**
- Model: whichever won Phase 0 (NuExtract3 vs ReaderLM-v2 favored)
- OCR (if image input): PaddleOCR or Tesseract 5; benchmark both
- Schema: Pydantic v2 (works natively with NuExtract, ReaderLM-v2)
- Storage: SQLite for eval results, no DB for prod yet
- Packaging: `uv` + single wheel; `pip install realestate-bench`

**Success metrics:**
- Per-field F1 ≥ 0.85 on 30-doc held-out set
- p95 latency < 30 sec/doc on M-series Mac (local) and < 5 sec/doc on 1× A100 (server)
- Zero silent failures (every extraction returns a structured response or an explicit error)

**Out of scope for this phase:**
- Long-running API, multi-tenant auth, web UI, paid plans

---

## Phase 1.5: Desktop App — local extraction, no cloud (November 2026)

### v1.0-desktop — Tauri 2 app with on-device model

**Target: November 2026** *(in progress — runs in parallel with Phase 1 CLI)*

**Why now:** every real-estate professional we talked to handles documents on their laptop. A hosted API asks them to upload contracts to a third party — that is a non-starter for leases, NDAs, and pre-NDA financial statements. The desktop app ships the same Phase 1 extractor as a local-first app that **never leaves the machine**.

**The stack (locked in this phase):**
- **App shell:** Tauri 2 (OS webview, not bundled Chromium) + React + TypeScript + Vite
- **Model:** [Needle 2/3 by Cactus Compute](https://huggingface.co/Cactus-Compute/needle2) — 45M parameters, **single 14 MB binary, 28 MB RAM, 500 tok/s on a Raspberry Pi 5**, grammar-constrained JSON output via a byte-level decoder grammar compiled from Pydantic schemas. Apache-2.0, on-device, no GPU.
- **Extraction layer:** Python sidecar using `cactus-needle` PyPI package (`pip install cactus-needle`); Tauri Rust core spawns it as a child process for each extraction.
- **Validation:** same Pydantic schemas + post-extraction rules as the CLI (`realestate_bench.schemas`).
- **Local DB:** SQLite via `rusqlite[bundled]` for the extracted-records store, FTS5 for search.
- **Export:** Excel (`.xlsx` via `rust_xlsxwriter`), JSON, CSV, plus the local SQLite DB.
- **First schema:** lease addendum (matches the Phase 1 schema — same code path).

**The on-device bet:** the PRD's 0.90 F1 gate on `ExtractBench` is not what the desktop app targets. The desktop app targets **user-perceived correctness on real lease documents**, with confidence flags below 0.7. We will publish the F1 on a held-out 20-doc lease set as the *desktop* quality bar (target: 0.75 per-field F1 — enough to be a 10× productivity multiplier for lease abstracting, not enough to skip human review). Phase 0/1's higher F1 thresholds apply to the hosted API track.

**Features:**
- [ ] Drag-and-drop a PDF / image / scanned photo into the app
- [ ] Pick a schema (lease addendum, invoice, custom) — declared in YAML or Python
- [ ] Run extraction locally; show the JSON with **per-field confidence coloring** (green ≥ 0.7, yellow 0.4-0.7, red < 0.4)
- [ ] Edit any field inline; edits are stored alongside the extraction
- [ ] Export to **Excel (one sheet per document, one row per field)**, JSON, CSV, or the local SQLite DB
- [ ] Bundle the Python sidecar (`realestate-extract`) into the Tauri app via Tauri's `externalBin` / `resources` so users get a single `.app` / `.msi` / `.AppImage` install
- [ ] No network calls at runtime; no telemetry; works fully offline

**Success metrics:**
- App launches and extracts a 5-page lease in < 30 seconds on M-series Mac
- 0.75 per-field F1 on a 20-doc held-out lease set (vs ground truth)
- Bundle size < 50 MB (Tauri shell + Needle binary + Python sidecar)
- Zero network egress verified in Wireshark during a full extraction

**Risks specific to this phase:**
| Risk | Mitigation |
|---|---|
| Needle 2 is small (45M) — extraction accuracy on messy real-world scans may be < 0.70 | Grammar-constrained decoding guarantees well-formed output; Pydantic validators catch bad values; confidence score below 0.7 forces review. |
| Tauri compile is slow (4-5 min first build, 30s incremental) | Manual scaffold (no `tauri init`); stub `dist/index.html` before `cargo check`. |
| Cactus-needle binary is x86_64 / ARM64 only — no Apple Silicon universal build? | Verify cross-arch download on first install; fall back to source build if needed. |
| Python sidecar adds 200 MB to the bundle | Use `uv` to ship a static-linked minimal env; pin to CPython 3.12; exclude `pip`/`setuptools`. |

**Out of scope for this phase:**
- Multi-user, sync, cloud backup
- Phase 1.5 is the **standalone desktop product**; Phase 2+ reliability work applies to both tracks

---

## Phase 2: Reliability + second use case (December 2026 → January 2027)

### v1.1 - Production-hardened on two schemas
**Target: January 2027**

**Features:**
- [ ] Second schema: **rent roll** (units, sqft, rent/sqft, lease dates, escalation). Add it without breaking the lease-addendum code path.
- [ ] Field-level confidence calibration on the AUGRC metric from ExtractBench — necessary if you're going to do auto-accept / review-queue routing
- [ ] OCR fallbacks: detect low-confidence text regions, re-run with the other OCR engine, vote
- [ ] Resume / retry on long docs (chunked extraction, partial progress saved)
- [ ] PDF + image + scanned-photo input support
- [ ] Golden-set regression: every commit must pass on the 60-doc golden set (30 lease + 30 rent roll). Block merge on regressions.
- [ ] Logs to local file with extraction trace (which model, which OCR, latency, cost-equivalent)

**Success metrics:**
- 95% schema-conformance rate (Pydantic validation passes)
- 5% review-queue rate (low-confidence flagged, rest auto-accepted)
- < 1 manual fix per 10 docs on the golden set
- Run on the actual ExtractBench leaderboard (submit scores; gain credibility)

**Out of scope for this phase:**
- API monetization, marketplace of community schemas, multi-tenant, enterprise SSO

---

## Phase 3: API + first external user (Q1 2027)

### v2.0 - Hosted API, one paying design partner
**Target: March 2027**

**Features:**
- [ ] FastAPI service: POST `/v1/extract` with schema + doc, returns validated JSON + per-field confidence
- [ ] Self-hosted on a single GPU box ($0.50-$2/hr on Vultr/Lambda)
- [ ] Auth: API keys only; per-key rate limit; usage metering
- [ ] Pricing: $0.05/page, $50/mo minimum (target: 5 design partners × $100/mo = $500 MRR)
- [ ] Landing page (`realestate.bench` or similar) with the ExtractBench leaderboard showing our model beating NuExtract3
- [ ] One RealEstate industry design partner on a paid pilot ($100-$500/mo, LOI in writing)
- [ ] Public docs site with the schemas, recipes, accuracy claims

**Success metrics:**
- 3-5 paying design partners
- $500-$2K MRR
- < 3% churn / month
- 1 customer references us in their own marketing

**Out of scope for this phase:**
- Marketplace, community schemas, SOC2, multi-region

---

## Phase 4: Schema marketplace + community (Q2 2027)

### v2.1 - Open schemas, community contributions
**Target: June 2027**

**Features:**
- [ ] Public schema registry: GitHub repo `realestate-bench/schemas` with PR-friendly reviews
- [ ] 10+ community-contributed schemas (PMS exports, bank statements, credit reports, 1099s, vendor invoices)
- [ ] Per-schema leaderboard (accuracy on golden set, latency, cost)
- [ ] Web UI: upload doc + pick schema → see extracted JSON with confidence coloring
- [ ] Human-in-loop store: corrections become eval data, eval data trains future models (with permission)

**Success metrics:**
- 20+ schemas in registry
- $10-25K MRR
- 50+ active users (mix of free + paid)
- 5+ community PRs merged

**Out of scope:**
- Enterprise SSO, SOC2, custom model fine-tuning as a service

---

## Phase 5: Enterprise + multi-document reasoning (H2 2027)

### v3.0 - Portfolio-scale workflows
**Target: Q4 2027**

**Features:**
- [ ] Multi-doc reasoning: extract from N docs in one workflow (rent rolls across all properties of a portfolio, reconciliation across statements)
- [ ] State-of-the-art accuracy on ExtractBench long split (currently 39.3% NuExtract3, 80.5% Reducto, 95.6% LlamaExtract Agentic Plus — gap is the prize)
- [ ] ERP/PMS integrations: AppFolio, Buildium, Yardi CSV/JSON export
- [ ] SOC2 Type I (only if enterprise pipeline requires it)
- [ ] Pricing tiers: Starter $99/mo, Pro $499/mo, Enterprise custom

**Success metrics:**
- $1-10M ARR
- 50+ paying customers
- Top 3 on ExtractBench leaderboard

**Out of scope (probably forever):**
- Replacing Yardi/AppFolio, full property-management platform

---

## Phase 6: Optional — wrap into RealEstate365 (any time)

If RealEstate365 (the Avenue One IP → education → SaaS play) wants to use this as a building block:

- [ ] Add lease-extraction as a paid add-on to RealEstate365 education graduates
- [ ] Embed in the data platform layer (per your "Bootstrap: education → Data Platform → PropTech SaaS" plan)
- [ ] Use the same extractor for the Avenue One doc corpus (8,110 docs, $100M+ SFR IP)

This is not a phase on the critical path. It is the upside if the standalone product grows slower than RealEstate365.

---

## Risks and how we mitigate

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Small models can't hit 0.85 F1 on real docs | Medium | High | Phase 0 picks the best model with evidence, not vibes. Plan B: route hard docs to a larger hosted model (NuExtract 2.0 PRO, LlamaExtract). |
| ExtractBench leaderboard is dominated by well-funded players | High | Low | We don't have to top the board; we have to beat the threshold our customers care about (pricing/dates/line items on real docs). |
| Long context doesn't translate to long-form reasoning | High | Medium | Build with chunked extraction from day 1; treat "model sees the whole doc" as a Phase 4 nice-to-have. |
| Schema marketplace becomes moderation hell | Medium | Medium | Phase 4 only. Phase 1-3 ship with curated schemas only. |
| RealEstate365 timeline shifts and we lose the natural design partner | Medium | Low | The 30-doc golden set + benchmark results are useful regardless. Standalone product is viable without RealEstate365 as anchor. |

---

## What this roadmap is NOT

- Not a 30-page strategy doc (the commercial-roadmap skill can produce that, but it would be premature — at week 1 we don't know if the model hits 0.85 F1)
- Not committed to specific quarters — they slide based on Phase 0 results
- Not a fundraising deck (raise at $15K MRR, after Phase 3, not now)

*Roadmap v0.1*
*Created: 2026-10-13*
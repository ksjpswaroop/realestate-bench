# RealEstate-Bench — Prompt Templates for Iterative Doc Generation

> 20 copy-paste prompts that regenerate (or audit) the PRD sections. Generated alongside the PRD so future updates stay grounded in evidence, not vibes. Methodology: `iterative-document-generation` skill — generate → verify (deterministic) → correct → finalize → stitch.

---

## How to use

Each prompt below targets one section (or section-cluster) of `PRD.md`. Run them with:

```bash
# Single-section regeneration
hermes prompt "PRD.md §2 Executive Summary ..." --attachment PRD.md

# Full regeneration (YOLO mode)
hermes batch docs/PROMPT_TEMPLATES.md --output PRD.md
```

After running, run the verification checks at the bottom of this file to confirm the regenerated PRD is internally consistent with `ARCHITECTURE.md`, `BACKLOG.md`, and `datasets/README.md`.

---

## Prompt Template 1 — Document Control (PRD §1)

```
Regenerate PRD §1 (Document Control) for RealEstate-Bench.

Required fields:
- Product name, codename, version, status, author, stakeholders
- Created + updated dates
- Repository path, license (code + data split)
- Revision history table (initially 1 row)
- Supersedes pointer to BACKLOG.md / ROADMAP.md

Format: Markdown with `##` headers, no filler. Match the existing PRD tone (terse, factual, no marketing language).

Source of truth (do not contradict):
- Phase 0 gate = per-field F1 ≥ 0.80 on drew-ipp invoice-extraction-benchmark
- License = Apache-2.0 code, CC-BY-4.0 data/docs
- Author = ksjswaroop (use this name; not "Swaroop")

Output: the §1 markdown only. Include a 1-line "ready to paste" summary at the top.
```

## Prompt Template 2 — Executive Summary (PRD §2)

```
Regenerate PRD §2 (Executive Summary) for RealEstate-Bench.

Required sub-sections (with subsection headers):
- Vision (1 paragraph)
- Problem (1 paragraph; reference data-entry cost $4-12/doc)
- Solution (1 paragraph; CLI + Python lib + future API)
- Business value (1 paragraph; value-prop for property managers)
- Success definition (12-month horizon; MRR + accuracy + LOI targets)

Hard numbers to include verbatim:
- $500-2,500 MRR by Q1 2027
- Per-field F1 ≥ 0.85 on lease_addendum golden set
- 1 signed design-partner LOI
- 30-doc held-out golden set for Lease Addendum + Rent Roll

Style: 1-2 sentences per paragraph. No filler. No "we believe" or "we hope."

Output: §2 markdown + a 1-line summary.
```

## Prompt Template 3 — Strategic Context (PRD §3)

```
Regenerate PRD §3 (Strategic Context) for RealEstate-Bench.

Required sub-sections:
- Market analysis (TAM/SAM/SOM with citations)
- Competitor analysis (table with Strength/Weakness columns)
- Positioning (1 paragraph)
- Differentiators (long-form, 5 numbered points)

Required citations (must be in §38 References too):
- TAM: $2.5B IDP market 2025 → $10-13B by 2030, CAGR 25-30%. Source: Grand View Research, Markets and Markets 2025.
- Real-estate slice: ~5-8% of horizontal SaaS doc processing
- US SAM: ~60,000 PM firms + 25,000 brokerages + 5,000 lenders (NAA 2024)

Required competitors (in this exact order):
NuMind, LlamaExtract, Reducto, Landing AI, Google DocAI, AWS Textract, Manual data entry, AppFolio/Buildium/Yardi internal tools.

Output: §3 markdown.
```

## Prompt Template 4 — Goals & Objectives (PRD §4)

```
Regenerate PRD §4 (Goals & Objectives) for RealEstate-Bench.

Three buckets, each 3-5 bullet points:
- Business goals (B1-B5)
- User goals (U1-U5)
- Technical goals (T1-T7)

Hard constraints:
- B1: $500-2,500 MRR by Q1 2027
- B2: $10-25K MRR by Q2 2027
- B3: Bootstrap until $15K MRR or enterprise deal
- T1: per-field F1 ≥ 0.80 on drew-ipp invoice-extraction-benchmark (Phase 0 gate)
- T2: per-field F1 ≥ 0.85 on lease_addendum golden set (Phase 1 gate)
- T4: p95 latency < 30s on M-series Mac, < 5s on A100
- T5: cost ≤ $0.03/doc at hosted scale

Style: terse. Each bullet is a single concrete claim, not a paragraph.

Output: §4 markdown.
```

## Prompt Template 5 — Personas (PRD §5)

```
Regenerate PRD §5 (Personas) for RealEstate-Bench.

Three personas, exact IDs and order:
1. Primary: Sarah — Operations Manager, 200-unit property management firm, Atlanta GA
2. Secondary: David — Solo Broker / Investor, 12-unit portfolio, Phoenix
3. Tertiary: Priya — Data Engineer at small PropTech startup

Each persona gets: Demographics (3-5 bullets), Goals (2-3 bullets), Pain points (2-3 bullets), Success metrics (2 bullets), Buying behavior (1 bullet).

Style: factual, specific (names, numbers, locations). No vague traits.

Output: §5 markdown.
```

## Prompt Template 6 — User Research (PRD §6)

```
Regenerate PRD §6 (User Research) for RealEstate-Bench.

Required sub-sections:
- Hypotheses (H1-H5; mark as to-validate)
- Validation plan (V1-V5)
- Key findings (placeholder: "Will populate in subsequent PRD versions")

Validation plan must include:
- 15 user interviews (3 PMs + 5 solo brokers + 2 data engineers + 5 tech-curious)
- 7-day diary study with 3 operators
- Landing-page test
- 100-person survey via BiggerPockets / r/realestate
- 1 paid design-partner pilot

Style: hypothesis-driven, falsifiable.

Output: §6 markdown.
```

## Prompt Template 7 — Scope Definition (PRD §7)

```
Regenerate PRD §7 (Scope Definition) for RealEstate-Bench.

Four sub-sections, in order:
- In Scope (Phase 1: CLI MVP, Lease Addendum schema, 30-doc golden set, F1 ≥ 0.85)
- In Scope (Phase 2: Rent Roll schema, 60-doc combined set, F1 ≥ 0.90)
- In Scope (Phase 3: FastAPI hosted, $0.05/page, 1 design partner)
- Out of Scope (multi-tenant, schema marketplace, multi-doc reasoning, ERP integration, SOC2, mobile, tenant-facing, replacing PMS)

Cross-reference: this section MUST align with BACKLOG.md phases exactly. Do not invent features not in BACKLOG.

Output: §7 markdown.
```

## Prompt Template 8 — User Journey (PRD §8)

```
Regenerate PRD §8 (User Journey) for RealEstate-Bench.

Primary persona: Sarah (Ops Manager, 200-unit property management firm).

Stages (in order, with brief descriptions):
Awareness → Landing Page Visit → Install → First-Run Onboarding → Try on Real Doc → Weekly Use → Scaling → Referral

Activation target: first useful extraction within 5 minutes of install.
Retention target: week-4 retention ≥ 40%.

Format: ASCII flow diagram + table of stages.

Output: §8 markdown.
```

## Prompt Template 9 — Functional Requirements (PRD §9)

```
Regenerate PRD §9 (Functional Requirements) for RealEstate-Bench.

10 functional requirements (FR-001 to FR-010) with format:
- ID, Priority (Must/Should/Could)
- User story (As a / I want to / So that)
- Acceptance criteria (3-7 specific, testable criteria each)
- Cross-refs to BACKLOG story IDs

Required FRs in order:
FR-001 Document ingestion (Must)
FR-002 Schema declaration (Must)
FR-003 Extraction execution (Must)
FR-004 Business-rule validation (Must)
FR-005 Confidence scoring + routing (Must)
FR-006 Caching + idempotency (Should)
FR-007 Bulk + batch execution (Should)
FR-008 Golden-set regression testing (Should)
FR-009 Logging + observability (Could)
FR-010 Schema introspection (Could)

Every acceptance criterion must be observable (button click, file write, latency <Xms). No "looks good" or "feels fast."

Output: §9 markdown.
```

## Prompt Template 10 — Feature Specifications (PRD §10)

```
Regenerate PRD §10 (Feature Specifications) for RealEstate-Bench.

Each feature gets: Overview, User stories (FR ref), Workflow, Inputs, Outputs, Errors, Edge cases, Permissions.

Required features (4):
1. Document Ingestion (FR-001)
2. Schema-driven Extraction (FR-002 + FR-003)
3. Validation + Confidence Routing (FR-004 + FR-005)
4. Golden-set Regression (FR-008)

Style: dense, technical. No marketing language.

Output: §10 markdown.
```

## Prompt Template 11 — User Flows (PRD §11)

```
Regenerate PRD §11 (User Flows) for RealEstate-Bench.

Required flows:
- Happy path (CLI single-doc)
- Happy path (CLI batch)
- Exception path (model timeout)
- Recovery path (cache hit)

Each flow: terminal-style output (with `[INFO]`/`[OK]`/`[WARN]`/`[ERROR]` markers) + brief prose explanation.

The CLI happy path MUST include a realistic LeaseAddendum extraction with 12 fields and per-field confidence.

Output: §11 markdown + a mermaid diagram for the happy path.
```

## Prompt Template 12 — Screen Requirements (PRD §12)

```
Regenerate PRD §12 (Screen Requirements) for RealEstate-Bench.

CLI has no GUI; "screens" = commands.

Required:
- CLI surface (table: command, purpose, phase)
- First-run wizard states
- (Future) Web UI minimal viable screens

CLI commands to include (in this order):
init, extract, batch, test, schemas list/show, cache list/clear, config get/set, review list/resolve.

Web UI screens (Phase 4, future):
Upload, Progress, Result, Review queue, History, Settings.

Output: §12 markdown.
```

## Prompt Template 13 — UX Requirements (PRD §13)

```
Regenerate PRD §13 (UX Requirements) for RealEstate-Bench.

Required sub-sections:
- Usability principles (5 bullets)
- Accessibility (4 bullets; CLI-specific + future Web)
- Desktop/server parity (table)

Usability principles (in this order):
1. CLI first, GUI second
2. Progressive disclosure
3. Plain language errors
4. Reversible actions
5. Visible provenance

Output: §13 markdown.
```

## Prompt Template 14 — Design System (PRD §14 + docs/DESIGN_SYSTEM.md)

```
Regenerate PRD §14 (Design System) for RealEstate-Bench. (For full design system, see docs/DESIGN_SYSTEM.md.)

PRD §14 is a 1-paragraph summary pointing to docs/DESIGN_SYSTEM.md.

Must mention:
- Color codes for CLI (with NO_COLOR respect)
- ASCII only (no Unicode boxes)
- Plain-text tables
- (Future) Web UI design tokens

Output: PRD §14 markdown (terse; details in docs/DESIGN_SYSTEM.md).
```

## Prompt Template 15 — Information Architecture (PRD §15)

```
Regenerate PRD §15 (Information Architecture) for RealEstate-Bench.

CLI command tree with arguments.

Format:
```
realestate-bench
├── init
├── extract
│   ├── --doc <path>
│   ├── --schema <id>
│   ├── --out <file>
│   ├── --model <id>
│   ├── --confidence-threshold <float>
│   ├── --timeout <seconds>
│   ├── --no-cache
│   ├── --log-format json|text
│   └── --verbose / --debug
├── batch
│   ├── --docs <glob>
│   ├── --schema <id>
│   ├── --out <dir>
│   └── --concurrency <int>
├── test
│   ├── --golden <dir>
│   └── --baseline <file>
├── schemas (list, show)
├── cache (list, clear)
├── config (get, set)
├── review (list, resolve)
└── --version / --help
```

All flags must be reflected in BACKLOG.md acceptance criteria (e.g. US-144 mentions --no-cache, --timeout, etc.).

Output: §15 markdown.
```

## Prompt Template 16 — Data + DB Design (PRD §16-17)

```
Regenerate PRD §16 (Data Requirements) and §17 (Database Design) for RealEstate-Bench.

§16 entities: Document, Schema, Extraction, ReviewQueueItem, GoldenDoc, GoldenResult, ModelConfig, Cache.

§17 SQL DDL for SQLite (Phase 1):
- documents (id, path, mime, sha256 UNIQUE, page_count, file_size, ingested_at)
- schemas (id, name, version, sha256, source_path, field_count, registered_at; UNIQUE(name, version))
- extractions (id, doc_id FK, schema_id FK, model, data_json, confidence_json, validation_json, trace_json, needs_review, created_at)
- cache (key PK = sha256(doc+schema), extraction_id FK, created_at)
- golden_docs (id, doc_id FK, schema_id FK, ground_truth_json, labeled_by, labeled_at)
- golden_results (id, run_id, golden_id FK, per_field_f1_json, overall_pass, created_at)

Indexes: idx_extractions_doc, idx_extractions_schema, idx_extractions_review (partial WHERE needs_review=1).

Output: §16 + §17 markdown with embedded SQL block.
```

## Prompt Template 17 — API + AI Requirements (PRD §18-19)

```
Regenerate PRD §18 (API Requirements) and §19 (AI Requirements) for RealEstate-Bench.

§18 covers both:
- Local CLI (table of commands)
- Hosted API Phase 3 (table of endpoints)

Required Phase 3 endpoints:
- POST /v1/extract (single-doc)
- POST /v1/batch (multi-doc)
- GET /v1/schemas (list)
- GET /v1/schemas/{id} (detail)
- GET /v1/usage (per-key)
- GET /healthz (liveness)

§19: AI requirements with model evaluation table.

Required models evaluated (in order):
1. NuExtract3-4B (Qwen3.5-4B, ~8GB, 262K ctx)
2. ReaderLM-v2 (1.5B, ~3GB, 512K ctx)
3. GLiNER2-large-v1 (486M, ~1GB, CPU-friendly)
4. Needle 2 (45M, ~100MB, on-device)

Decision rule (Phase 0): `score = 0.5 × ExtractBench_short_F1 + 0.3 × invoice_benchmark_F1 + 0.2 × real_estate_golden_F1`, gate = invoice F1 ≥ 0.80.

Output: §18 + §19 markdown.
```

## Prompt Template 18 — Security, Privacy, Performance, Reliability (PRD §20-23)

```
Regenerate PRD §20-23 for RealEstate-Bench:
- §20 Security Requirements (table)
- §21 Privacy Requirements
- §22 Performance Requirements (table with targets)
- §23 Reliability Requirements

Performance targets (verbatim, do not change):
- Single-doc latency < 30s p95 on M-series Mac, < 5s p95 on A100
- Batch throughput ≥ 200 docs/hour on A100
- Cache hit < 100ms p99
- Schema validation < 50ms p99
- Confidence scoring < 20ms p99
- Cold-start < 60s
- Memory idle < 500MB, extraction < 16GB VRAM (A100), < 12GB unified (M-series)

Compliance posture: SOC2 only at Phase 5 enterprise deal; GDPR via local-first + cache clear; HIPAA explicitly out of scope.

Output: §20-23 markdown.
```

## Prompt Template 19 — Observability + Integration + Reporting + Analytics + Testing (PRD §24-28)

```
Regenerate PRD §24-28 for RealEstate-Bench.

§24 Observability (logs, metrics, tracing, alerts)
§25 Integration (outbound: HF Hub; Phase 3: Stripe, PagerDuty)
§26 Reporting (operator report CLI; Phase 4 leaderboard)
§27 Analytics (events, funnels, retention; CLI opt-in)
§28 Testing (5 layers: unit, integration, golden-set regression, model eval, security)

Testing layers (in order):
1. Unit tests (pytest, 100% of extract/schemas/state/golden)
2. Integration tests (3 known docs, mock model)
3. Golden-set regression (30/60 docs, per-field F1, CI gate)
4. Benchmark suite (Phase 0 only, full eval on 7 datasets)
5. Property-based tests (Phase 4, Hypothesis-generated docs)

Output: §24-28 markdown.
```

## Prompt Template 20 — Release Plan + Risks + Success + KPIs + Open Questions + AI Architecture (PRD §31-39)

```
Regenerate PRD §31-39 for RealEstate-Bench.

§31 Release Plan (MVP → Phase 5 with quarterly targets)
§32 Risk Assessment (12 risks: R1-R12, with Probability × Impact × Mitigation)
§33 Assumptions (A1-A11)
§34 Constraints (technical, business, legal, budget, time, compatibility)
§35 Success Metrics (Product + Business + Technical + Adoption)
§36 KPIs Dashboard (founder/operational/AI-specific)
§37 Open Questions (OQ1-OQ8)
§38 Appendices (datasets, model comparison, glossary, references, revision history)
§39 AI / Agent Architecture (Phase 4+ agent catalog, memory, tools, events, resumability, approvals, channels, validation, abstraction, trust/safety/cost)

Phase 5 (Enterprise + multi-doc reasoning):
- Multi-doc reasoning (13pt story)
- Top-3 ExtractBench long split
- ERP/PMS export (AppFolio, Buildium, Yardi)
- SOC2 Type II

Required risks (do not omit): R1 (small models don't hit 0.80 F1), R7 (PII leak), R11 (legal liability).

Required open questions: OQ1 (fine-tune or not), OQ2 (serverless or always-on), OQ6 (pricing model).

Output: §31-39 markdown.
```

---

## Verification Checklist

After regenerating any PRD section, run these deterministic checks:

```bash
# 1. All 39 sections present
grep -c "^## " PRD.md   # should be 39

# 2. No TBD / TODO / placeholder text
grep -cE "TBD|\[placeholder\]|\[TODO\]" PRD.md   # should be 0

# 3. Cross-refs to BACKLOG story IDs are valid
for id in $(grep -oE "US-[0-9]+" PRD.md | sort -u); do
  grep -q "^### $id" BACKLOG.md || echo "MISSING: $id"
done

# 4. Cross-refs to ARCHITECTURE sections exist
for section in $(grep -oE "ARCH §[0-9]+" PRD.md | grep -oE "[0-9]+" | sort -u); do
  grep -q "^## $section\\." ARCHITECTURE.md || echo "MISSING ARCH §$section"
done

# 5. FR IDs in BACKLOG match FRs in PRD
grep -oE "FR-[0-9]+" PRD.md | sort -u > /tmp/prd_frs.txt
grep -oE "FR-[0-9]+" BACKLOG.md | sort -u > /tmp/backlog_frs.txt
diff /tmp/prd_frs.txt /tmp/backlog_frs.txt   # should be empty

# 6. Required competitor names appear in §3
for c in "NuMind" "LlamaExtract" "Reducto" "Landing AI" "Google DocAI" "AWS Textract"; do
  grep -q "$c" PRD.md || echo "MISSING competitor: $c"
done

# 7. F1 targets verbatim
grep -F "per-field F1 ≥ 0.80" PRD.md
grep -F "per-field F1 ≥ 0.85" PRD.md
grep -F "per-field F1 ≥ 0.90" PRD.md

# 8. MRR targets verbatim
grep -F "\$500-2,500 MRR" PRD.md
grep -F "\$10-25K MRR" PRD.md
```

If any check fails, fix the offending section **before** commit — never ship a regenerated PRD that fails the cross-doc contract.

---

## Notes for future iterations

- Each prompt template can be re-run independently — sections don't depend on each other for regeneration, only for cross-reference verification.
- Memory store (per `iterative-document-generation` skill) lives at `docs/.doc-memory.json` — populated after each section-finalize.
- If the project pivots (e.g. "drop schema marketplace, focus on hosted API"), regenerate §7, §9, §15, §31, §35 and run the verification checklist.

---

*Prompt Templates v1.0 — RealEstate-Bench © 2026 ksjswaroop*
*Methodology: iterative-document-generation skill (generate → verify → correct → finalize → stitch).*
*Deterministic verification only — no LLM judgment on quality.*
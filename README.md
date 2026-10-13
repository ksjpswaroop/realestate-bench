# RealEstate-Bench

Honest document extraction for real estate. Pick the best small model, ship accurate JSON, beat the hosteds on cost.

**Status:** Phase 0 in progress · v0.1 · `2026-10-13`

## What this is

A CLI + Python library (and eventually a hosted API) that turns leases, rent rolls, vendor invoices, lender statements, and other real-estate documents into validated structured JSON. Built around three principles:

1. **Transparent benchmark.** Every model claim has a published number on ExtractBench + the drew-ipp invoice benchmark + our own real-estate golden set.
2. **Real-estate-first schemas.** Lease Addendum, Rent Roll, Vendor Invoice, Lender Statement — not generic.
3. **Local-first by default.** Self-hosted model weights, no API key required.

## Documentation

- [`PRD.md`](./PRD.md) — Full 39-section Product Requirements Document
- [`ARCHITECTURE.md`](./ARCHITECTURE.md) — System architecture, components, data flow
- [`BACKLOG.md`](./BACKLOG.md) — Sprint backlog with stable story IDs (US-001..US-640)
- [`ROADMAP.md`](./ROADMAP.md) — Phase-by-phase timeline (commercial)
- [`datasets/README.md`](./datasets/README.md) — Benchmark dataset manifest
- [`docs/adr/`](./docs/adr/) — Architecture Decision Records
- [`docs/DESIGN_SYSTEM.md`](./docs/DESIGN_SYSTEM.md) — CLI + (future) Web UI design tokens
- [`docs/WIREFRAMES.md`](./docs/WIREFRAMES.md) — CLI screens + future UI mockups
- [`docs/PROMPT_TEMPLATES.md`](./docs/PROMPT_TEMPLATES.md) — 20 iterative-regeneration prompts

## Quickstart (Phase 1, not yet shipping)

```bash
# Install
pip install realestate-bench

# First-run wizard
realestate-bench init

# Extract from a PDF
realestate-bench extract \
    --doc leases/addendum_42.pdf \
    --schema lease_addendum \
    --out out/addendum_42.json

# Run the golden-set regression
realestate-bench test --golden bench/golden/v1/
```

## Desktop app (Phase 1.5 — shipped ✅)

A local Tauri 2 desktop app that runs the Phase 1 extractor on-device.
No network at runtime. No hosted service. The 45M-parameter cactus-needle
model is bundled; documents never leave your machine.

```bash
cd desktop/
pnpm install
pnpm tauri build           # → 6.1 MiB .app bundle in target/release/bundle/macos/
open target/release/bundle/macos/RealEstate-Bench.app
```

The window has three pages:

- **Extract** — drag-and-drop or file-pick a PDF / image / text, choose a
  schema (Invoice, LeaseAddendum, or RentRoll), click *Extract*. Per-field
  confidences are color-coded; click any field to edit it (your edit is
  recorded in the `edits` table and overrides the model value in exports).
- **Extractions** — list, search, and re-open every extraction. Full-text
  search runs over the FTS5 virtual table.
- **Settings** — DB path, size, row counts, and a wipe button.

Exports: **Excel (.xlsx)**, **JSON**, **CSV**. All open in any spreadsheet
or text editor. Edited values win over model values in Excel/CSV exports.

Tests passing: 6/6 pytest (sidecar) + 2/2 cargo integration (sidecar
smoke + DB flow) + `tsc --noEmit` clean + `vite build` clean.

> **Why no LLM-agent orchestrator (no Hermes, no LangChain) inside the
> .app?** The desktop app's orchestration is deterministic Rust
> (`commands.rs` → sidecar → SQLite). Adding an LLM-agent framework
> would break FR-001 (latency), FR-011 (no network at runtime), and
> the "double-click and it works" UX. The decision is documented in
> [`docs/adr/0001-no-hermes-in-desktop-runtime.md`](./docs/adr/0001-no-hermes-in-desktop-runtime.md)
> and the rationale is summarized in `ARCHITECTURE.md` §3.7. Hermes
> Agent is the AI tool we use to *build* the app, not a component the
> app ships with.

## Phase 0 Gate

Before Phase 1 ships, we need per-field F1 ≥ 0.80 on drew-ipp's invoice-extraction-benchmark for the chosen model. Models being evaluated: NuExtract3, ReaderLM-v2, GLiNER2, Needle 2.

See [`ROADMAP.md`](./ROADMAP.md) for the full timeline.

## License

Apache-2.0 for code · CC-BY-4.0 for datasets and documentation

## Author

ksjswaroop · `~/projects/realestate-bench/`
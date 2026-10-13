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
- [`BACKLOG.md`](./BACKLOG.md) — Sprint backlog with stable story IDs (US-001..US-530)
- [`ROADMAP.md`](./ROADMAP.md) — Phase-by-phase timeline (commercial)
- [`datasets/README.md`](./datasets/README.md) — Benchmark dataset manifest
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

## Phase 0 Gate

Before Phase 1 ships, we need per-field F1 ≥ 0.80 on drew-ipp's invoice-extraction-benchmark for the chosen model. Models being evaluated: NuExtract3, ReaderLM-v2, GLiNER2, Needle 2.

See [`ROADMAP.md`](./ROADMAP.md) for the full timeline.

## License

Apache-2.0 for code · CC-BY-4.0 for datasets and documentation

## Author

ksjswaroop · `~/projects/realestate-bench/`
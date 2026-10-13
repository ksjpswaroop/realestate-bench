# RealEstate-Bench Desktop App

Local Tauri 2 desktop app that runs the Phase 1 extractor on-device.
**No network at runtime. No hosted service. No LLM-agent orchestrator.**

## Quickstart

```bash
pnpm install
pnpm tauri build           # → 6.1 MiB .app bundle in src-tauri/target/release/bundle/macos/
open src-tauri/target/release/bundle/macos/RealEstate-Bench.app
```

For development with hot-reload:

```bash
pnpm tauri dev
```

## Architecture

```
React (src/)                  Tauri webview, UI
   │ tauri.invoke
   ▼
Rust core (src-tauri/)        commands.rs → db.rs + extract.rs
   │ tauri-plugin-shell::sidecar
   ▼
Python sidecar (sidecar/)     realestate_extract/cli.py → cactus-needle 45M
   │ JSON over stdin/stdout
   ▼
SQLite (~/Library/...)        documents + extractions + fts_extractions + edits
```

Each Tauri command is a typed async function in `src-tauri/src/commands.rs`.
Sidecar invocation is in `src-tauri/src/extract.rs`. There is **no
LLM-agent framework in the runtime** — orchestration is deterministic Rust.

## Why no Hermes / LangChain / smolagents inside the app?

**Hermes Agent** (and other LLM-agent frameworks) is the AI tool that
*built* this codebase, not a component the app ships with. Embedding an
agent framework in the `.app` would:

- break FR-001 (5-second first extraction) by adding an LLM hop
- break FR-011 (no network at runtime) — every LLM framework needs a provider
- break the "double-click and it works" UX — providers need API keys
- add 200+ MB to the bundle
- make extraction non-reproducible (sampling LLM in the loop)

The full decision record with alternatives considered:
[`../docs/adr/0001-no-hermes-in-desktop-runtime.md`](../docs/adr/0001-no-hermes-in-desktop-runtime.md).
A short summary is in `../ARCHITECTURE.md` §3.7.

## Layout

```
desktop/
├── src/                        React app (DropPage, ExtractionsPage, SettingsPage,
│                               ExtractionView, lib/ipc.ts, styles/globals.css)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs             Tauri entry; wires up plugins and setup
│   │   ├── lib.rs              Public re-exports
│   │   ├── commands.rs         IPC commands (extract, list, get, delete, edit, export...)
│   │   ├── db.rs               SQLite + FTS5 + migrations
│   │   ├── extract.rs          Sidecar spawn + JSON I/O
│   │   ├── errors.rs           AppError enum (Io, Db, Sidecar, Schema, Excel, Zip, Json)
│   │   └── paths.rs            OS-appropriate app data dir
│   ├── bin/                    Sidecar binaries (gitignored; built by scripts/build_sidecar.sh)
│   ├── tests/                  Integration tests (sidecar_smoke, db_flow)
│   ├── capabilities/           Least-privilege Tauri capabilities
│   ├── icons/                  App icons
│   └── tauri.conf.json         Build config (beforeBuildCommand = vite build)
├── sidecar/
│   ├── realestate_extract/     Python package (cli.py, schemas/, runner.py, ...)
│   ├── bin/realestate-extract  Sidecar entry script (absolute shebang, clears PYTHONPATH)
│   ├── .venv/                  Python 3.12 + cactus-needle + pydantic + pypdf + pillow
│   └── README.md               Sidecar hardening details
├── schemas/                    JSON schemas for the 3 built-in types (invoice, lease_addendum, rent_roll)
├── tests/                      pytest fixtures + 6 tests
├── package.json                pnpm scripts
├── vite.config.ts              Vite config
└── tsconfig.json               TypeScript strict mode
```

## Tests

```bash
# Python sidecar (pytest)
cd sidecar/
.venv/bin/pytest ../tests/ -v

# Rust integration tests
cd src-tauri/
cargo test

# TypeScript / build
pnpm exec tsc --noEmit
pnpm exec vite build
```

Currently passing: 6/6 pytest + 2/2 cargo + `tsc --noEmit` + `vite build`
+ `tauri build` (full bundle).

## Build artifact

After `pnpm tauri build`:

```
src-tauri/target/release/bundle/macos/RealEstate-Bench.app
  → 6.1 MiB, launches via `open`, no install required
```

The .app bundles:
- The Rust binary (`realestate-bench-desktop`, 6.0 MiB optimized)
- The Python sidecar (`bin/realestate-extract-aarch64-apple-darwin`, ~3 MiB)
- The cactus-needle model + Python 3.12 venv (~12 MiB in `sidecar/.venv/`)
- A `.icns` icon (Resources/AppIcon.icns)

## Privacy

Zero network calls at runtime. The Tauri CSP is
`default-src 'self'; connect-src 'none'`. The sidecar clears
`PYTHONPATH`, `NEEDLE_TELEMETRY`, `HF_HUB_DISABLE_PROGRESS_BARS`, and
`TRANSFORMERS_VERBOSITY`. Tauri capabilities grant only the minimum
permissions (`core:default`, `core:window:default`, `dialog:*`).
Verified by `pnpm tauri build && open ...app` + Wireshark capture during
Phase 1.5 acceptance.

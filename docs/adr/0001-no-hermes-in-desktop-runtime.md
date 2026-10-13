# ADR-001: No Hermes-Agent in the desktop app's runtime

**Status:** Accepted (2026-10-13)
**Deciders:** ksjpswaroop
**Relates to:** [ARCHITECTURE.md §3](ARCHITECTURE.md), [PRD §FR-011](PRD.md), [BACKLOG §Phase 1.5](BACKLOG.md)

## Context

The Phase 1.5 desktop app (`desktop/`) is a Tauri 2 + Rust + React + Python
sidecar application that runs a 45M-parameter cactus-needle extraction model
on-device. The user is considering whether to embed Hermes Agent as the
in-app orchestrator that drives the sidecar.

The same user (ksjpswaroop) is also the maintainer of Hermes Agent, and
Hermes is the AI tool that wrote the implementation, so the question is
natural. The answer is: no, Hermes has no role in the desktop app's
runtime. The reasons follow.

## Decision

The desktop app's runtime orchestration is implemented in Rust
(`desktop/src-tauri/src/commands.rs` + `extract.rs`), not via Hermes.
Hermes was used **during development** (writing code, running tests,
committing) and is not bundled, linked, or invoked at runtime by the
`.app`.

## Rationale

### 1. Different problem class

Hermes Agent is a **developer / user-facing LLM agent framework**:
skills, persistent memory, scheduled jobs, multi-platform messaging
gateway, sub-agent delegation, and tool-calling driven by an LLM that
reasons about user prompts.

The desktop app is a **deterministic document extraction tool**:
drag a PDF, pick a schema, click Extract, get JSON in ~600 ms, save to
SQLite, export to Excel. The "decision tree" is:

```
on drag-drop OR file-pick
  → compute SHA-256
  → upsert into `documents` table
  → spawn sidecar process
  → write extraction + FTS5 row
  → render result in React

on "Export Excel" click
  → query `extractions` + `edits`
  → build .xlsx in memory
  → write to user-chosen path
```

There is **no LLM reasoning loop, no tool-calling, no multi-step plan**.
A 45M-parameter extraction model is not capable of reliable tool use
anyway — it exists to map `(document, JSON schema) → JSON`, nothing else.

### 2. Hard runtime constraints forbid it

The desktop app must satisfy:

- **FR-011 (PRD §6):** *No network calls at runtime.* Documents never
  leave the device. The sidecar already runs with
  `PYTHONPATH=""`, `NEEDLE_TELEMETRY=0`, `HF_HUB_DISABLE_PROGRESS_BARS=1`,
  and `TRANSFORMERS_VERBOSITY=error` to enforce this.
- **FR-001:** First extraction under 5 seconds on a warm cache.
  Hermes would add at minimum one network round-trip to an LLM
  provider (it is provider-agnostic, but it always uses one), pushing
  the path from 600 ms (Rust IPC → sidecar → model) to 1-3 seconds
  (Rust IPC → Hermes → provider → sidecar → model). Even a tiny local
  model via Ollama would add 200-500 ms of tokenizer + prompt-build
  overhead for a reasoning step that has nothing to reason about.
- **US-601 (BACKLOG):** Tauri capabilities must be least-privilege
  and sidecar-scoped. Adding Hermes would require granting the
  shell plugin access to `hermes` itself — Hermes's gateway, MCP,
  and skills subsystems are far broader than `bin/realestate-extract`.
- **US-612 (BACKLOG):** Schemas are deterministic Pydantic v2 classes
  + JSON Schema, validated at extract time. The model returns the
  shape we asked for, or the JSON parser rejects it. There is no
  place for an LLM orchestrator to make a useful decision.

### 3. Orchestration is already Rust-native

The current chain is:

```
React (DropPage, ExtractionView)
   │   tauri.invoke("extract_document", { ... })
   ▼
Tauri command (commands.rs::extract_document)
   │   db::upsert_document, then
   │   tauri_plugin_shell::ShellExt::sidecar("realestate-extract")
   ▼
Python sidecar (realestate_extract/cli.py)
   │   cactus-needle 45M, ~600 ms
   ▼
SQLite (extractions + fts_extractions + edits)
   │
   ▼  ipc response
React re-renders ExtractionView
```

This is fully deterministic, fully testable, runs without an LLM, and
exits in under a second on a warm cache. Replacing any link in this
chain with an LLM agent would *remove* functionality (determinism,
offline operation, sub-second latency) without adding any.

### 4. Where Hermes *would* fit — and why it's separate

Hermes makes sense when the task requires **reasoning across many
files, scheduling, or natural-language goals**. Concretely, it would
fit:

| Use case | Why Hermes helps | Status |
|----------|------------------|--------|
| "Extract rent rolls from every PDF in `~/Documents/2025-12-leases/`" | Multi-step plan, batch dispatch, resumable | Not built. Would live in a separate `realestate-bench-agent/` CLI, not in the desktop `.app`. |
| "Run the golden-set regression on the new model and report the diff" | Cron + tool calls + memory of past runs | Not built. Lives in CI, not the desktop app. |
| "Watch this folder; when a new lease appears, auto-extract it" | Cron + filesystem watch + the sidecar | Not built. Would be a separate `realestate-bench-watcher` daemon. |

The common thread: **all of these are developer/operator workflows,
not in-app workflows.** They live outside the `.app`, and none of
them would benefit from being bundled *into* the desktop app's
runtime. If we ever build them, they'd be `pip install
realestate-bench[agent]` extras that use the same Python sidecar CLI
and the same `runner.py` — they'd just be driven by Hermes on a
developer machine, not by an LLM agent in a Tauri webview.

### 5. Cost

Hermes is a substantial dependency:

- `~/.hermes/` is hundreds of MB; bundling it would bloat the 6.1 MiB
  `.app` to >200 MiB.
- It depends on a working LLM provider configuration
  (`~/.hermes/config.yaml`, API keys in `~/.hermes/.env`). A user
  who downloads the `.app` from GitHub and double-clicks it must be
  able to extract a document *without ever touching a config file*.
  That's the whole point of the Phase 1.5 product.
- It uses tool-calling and JSON-mode responses; the 45M cactus-needle
  model does not, and forcing it to would be slower and more brittle
  than the current direct-Pydantic path.

## Consequences

Positive:

- The `.app` stays 6.1 MiB, fully offline, no provider config, no
  API key, no first-run wizard.
- Deterministic extraction pipeline; same input + same schema = same
  output. Testable with golden sets.
- Build pipeline is `pnpm tauri build` and nothing else; no
  Python-side agent dependency to install, lock, or upgrade.

Negative:

- The desktop app does not benefit from Hermes' skills/memory/
  scheduling features. If a user wants "extract from a whole
  folder", they need to use the Python CLI or wait for a future
  watcher daemon (out of scope for Phase 1.5).
- Future in-app features that *would* benefit from LLM reasoning
  (e.g. "summarize the diffs between this lease and the standard
  template") will need to add a *narrow* LLM hook, not the full
  Hermes runtime. This is a deliberate trade — we want the LLM hook
  to be opt-in, scoped to the specific feature, and lazy-loaded, not
  always-on.

## Alternatives considered

| Option | Why rejected |
|--------|--------------|
| Embed full Hermes in the desktop app | Adds 200+ MB, requires provider config, violates FR-011 and FR-001. |
| Use Hermes' sub-agent system to dispatch multiple sidecars in parallel | Sidecar already parallelises across processes; tokio on the Rust side gives finer control. The "many sub-agents" pattern is for LLM reasoning, not for "run the same model 200 times." |
| Add Hermes as a developer tool that *builds* the .app, not as runtime | Already doing this. Documented in the "Author" line of the PRD and in this ADR. |

## References

- PRD §6 (Non-Functional Requirements) — FR-001 (latency), FR-011 (no network)
- BACKLOG §Phase 1.5 — US-601, US-612, US-618
- ARCHITECTURE.md §3 — Sidecar invocation
- `desktop/sidecar/README.md` — Sidecar environment hardening
- `desktop/src-tauri/src/commands.rs` — Rust orchestration chain
- `desktop/src-tauri/src/extract.rs` — Sidecar spawn logic

# RealEstate-Bench — Design System

> **Note:** RealEstate-Bench is primarily a CLI in Phase 1-3. This document covers CLI text design tokens (Phase 1) and the future Web UI design system (Phase 4). When the Web UI ships, the CLI tokens become a separate "terminal design system" sub-page.

## 1. Principles

1. **CLI first, GUI second.** The CLI is the primary surface; the GUI serves users who can't or won't use a terminal.
2. **Plain language over jargon.** Errors explain what to do, not what broke.
3. **Reversible actions.** Every command can be re-run; nothing is destructive without `--force`.
4. **Visible provenance.** Every output shows where it came from (model + schema + doc hash).
5. **Color is additive.** Color codes status; meaning is also in words. Works in greyscale.

## 2. CLI Color Tokens (Phase 1)

Auto-disabled when `NO_COLOR` env var is set, or stdout is not a TTY.

| Token | ANSI | Use |
|---|---|---|
| `pass` | green (32) | Validation pass, auto-accepted |
| `warn` | yellow (33) | Needs review, low confidence |
| `error` | red (31) | Hard failure, exit code != 0 |
| `info` | cyan (36) | Status, progress |
| `dim` | dim (2) | File paths, hashes, secondary info |
| `accent` | magenta (35) | Schema names, model names |
| `bold` | bold (1) | Headings, key values |

Hex equivalents (for the future Web UI):

| Token | Hex |
|---|---|
| `--pass` | `#16a34a` |
| `--warn` | `#ca8a04` |
| `--error` | `#dc2626` |
| `--info` | `#0891b2` |
| `--dim` | `#6b7280` |
| `--accent` | `#a855f7` |
| `--fg` | `#0f172a` |
| `--fg-muted` | `#475569` |
| `--bg` | `#fafafa` |
| `--border` | `#e5e7eb` |

Dark theme:

| Token | Hex |
|---|---|
| `--fg` | `#f8fafc` |
| `--fg-muted` | `#cbd5e1` |
| `--bg` | `#0f172a` |
| `--border` | `#1e293b` |

## 3. CLI Typography

ASCII only. No Unicode boxes (CLI portability).

| Element | Style | Example |
|---|---|---|
| Heading | Bold, all caps | `EXTRACT` |
| Subheading | Bold | `Pipeline:` |
| Field label | Bold | `tenant:` |
| File path | Dim | `~/.cache/realestate-bench/models/nuext3/` |
| Hash | Dim, monospace | `a3f4e2c1...` |
| Number | Default | `42` |
| Currency | Default | `$4,500.00` |
| Date | Default | `2026-01-01` |

JSON output is indented 2 spaces. Tables are aligned columns.

## 4. CLI Spacing

- 2-space indent in JSON
- 1 blank line between sections
- 4-space column gap in summary tables

```
[INFO] Extraction complete in 11.2s

[OUTPUT]
{
  "data": {...},
  "confidence": {...}
}

[INFO] Wrote out/addendum_42.json (1.2 KB)
```

## 5. Iconography

Emoji-as-iconography, prefixed with bracket:

| Icon | Use |
|---|---|
| `[INFO]` | Informational status |
| `[OK]` | Success |
| `[WARN]` | Warning |
| `[ERROR]` | Error |
| `[DEBUG]` | Debug (only with `--verbose`) |
| `[PROGRESS]` | Progress bar updates |

## 6. Motion (CLI)

- Progress bars use `tqdm` (no custom animation)
- Spinner on long model calls
- No blinking or animation in non-progress output

## 7. Web UI Design System (Phase 4, future)

When the Web UI ships, this section becomes the source of truth.

### 7.1 Color palette

Same hex tokens as §2 (light + dark). All colors paired with a non-color cue (icon or text) for accessibility.

### 7.2 Typography

- **Font stack:** `Inter, system-ui, -apple-system, sans-serif`
- **Mono:** `'JetBrains Mono', 'Fira Code', ui-monospace, monospace`
- **Sizes:**
  - `xs` 12px · `sm` 14px · `base` 16px · `lg` 18px · `xl` 20px · `2xl` 24px · `3xl` 30px · `4xl` 36px

### 7.3 Spacing

4px base scale: `0, 4, 8, 12, 16, 24, 32, 48, 64` (px)

### 7.4 Components

| Component | Used in | Notes |
|---|---|---|
| `UploadDropZone` | Upload page | Drag/drop + click to upload |
| `SchemaPicker` | Upload page | Card grid; each card shows schema name + field count |
| `ProgressBar` | Result page, batch | Real-time progress with ETA |
| `JsonTree` | Result page | Expand/collapse nested objects; confidence-colored leaves |
| `ConfidenceBadge` | Result page | Green ≥ 0.9, yellow 0.7-0.9, red < 0.7 |
| `ValidationPill` | Result page | Green = PASS, red = FAIL, gray = SKIPPED |
| `ReviewQueueRow` | Review queue | Inline editor for corrections |
| `HistoryRow` | History page | Doc name + status + re-run button |
| `ApiKeyCard` | Settings | Show/hide toggle, copy button |

### 7.5 States

Every component has these states:

- **Empty:** helpful empty-state message + CTA
- **Loading:** spinner or skeleton
- **Error:** error message + retry button
- **Success:** confirmation message
- **Review-needed:** confidence warning + review CTA

### 7.6 Keyboard map (Web UI)

| Key | Action |
|---|---|
| `u` | Upload |
| `s` | Open schema picker |
| `r` | Open review queue |
| `h` | Open history page |
| `?` | Show keyboard help |
| `Cmd/Ctrl + Enter` | Run extraction |

### 7.7 Brand identity (Web UI / desktop app)

**Mark.** A house silhouette (roof + walls) in forest green, with a
JSON brace `{` in paper-white cut into the wall, and a terracotta
ground line. The mark is the **only** hand-drawn icon — every other
icon comes from `lucide-react`.

**Color narrative.** The palette is drawn from physical materials
that exist in real estate: forest (evergreen landscaping), slate
(blueprint paper), terracotta (clay roof tiles), moss (mature
shrubbery), saffron (autumn leaves), brick (masonry), paper (the
warm off-white of a printed lease), graphite (pencil).

**Why no violet/purple.** Violet is the SaaS-default brand color;
RealEstate-Bench wants to read as a tool for the *physical* world of
property, not a generic web app. The brand-mark uses forest +
terracotta specifically because neither has been over-marketed in
PropTech.

**Wordmark.** `RealEstate` in forest, `·Bench` in terracotta. The
midpoint dot is the only decorative element. System sans-serif only
(no webfonts shipped in the offline desktop app).

**Color tokens.** The full set lives in
`desktop/src/styles/globals.css`. Key tokens:

```css
--color-primary:   #0d6b5e;   /* forest-500 */
--color-secondary: #1e4a6b;   /* slate-500 */
--color-accent:    #b06a3a;   /* terracotta-400 */
--color-success:   #2f7a3a;   /* moss-500 */
--color-warning:   #c47800;   /* saffron-500 */
--color-danger:    #a83232;   /* brick-500 */
--bg-page:         #fafaf8;   /* paper-50 */
--bg-surface:      #ffffff;
--fg-default:      #1a1a1a;   /* graphite-300 */
```

**Industry-pack overrides.** A future industry pack overrides the
accent color on its own container:

```css
.pack-medical    { --color-accent: var(--color-terracotta-500); }
.pack-legal      { --color-accent: var(--color-slate-500); }
.pack-finance    { --color-accent: var(--color-moss-500); }
.pack-logistics  { --color-accent: var(--color-saffron-500); }
.pack-hr         { --color-accent: var(--color-forest-400); }
```

Components that reference `var(--color-accent)` (the wordmark "Bench",
the dropzone hover state, `.card--accent`, `.badge--accent`) all
recolor automatically. **No per-pack stylesheet needed.**

**Component library.** The full CSS-only component set ships in
`globals.css`:

- `.btn` — primary, secondary, accent, ghost, danger × sm/md/lg
- `.card` — base, elevated, accent, primary
- `.input`, `.select`, `.textarea` — focus rings use forest-50 soft
- `.badge` — 5 variants
- `.confidence` — engine-specific, three-tier (high/mid/low)
- `.schema-pill` — industry-pack chooser
- `.dropzone` — drag-and-drop target
- `.app-shell`, `.app-header`, `.app-main` — layout primitives

**Assets.** All brand assets live in `desktop/src-tauri/icons/`:

| File | Use |
|---|---|
| `icon.png`, `icon-1024.png` | Tauri bundle base + store |
| `icon.icns`, `icon.ico` | macOS + Windows bundle (multi-size) |
| `Square*.png` | Windows Store tiles |
| `wordmark-*.png` | Header / docs |
| `og-1200x630.png` | Social share |

Regenerate with `python3 desktop/scripts/regen_icons.py`.

**Full brand guide.** See §11 below for the complete brand story
(mark anatomy, real-estate color logic, accessibility checklist).

## 8. Accessibility

### CLI
- `--help` discoverable via `man realestate-bench` (generated from Typer)
- `--output-format json|yaml|text` — text mode is human-readable
- Color auto-disabled on NO_COLOR / non-TTY
- Screen reader: structured output modes preserve semantic structure

### Web UI (future)
- WCAG 2.2 AA target
- Contrast: ≥ 4.5:1 for normal text, ≥ 3:1 for large text
- Focus visible on all interactive elements
- Motion reduced: respect `prefers-reduced-motion`
- Color + icon/text labels for status (not color alone)
- **Forest-on-paper = 6.0:1** (AA pass at all sizes)
- **Terracotta-on-paper = 3.7:1** (AA pass for large text + graphical
  elements; not for body text — that's why terracotta is *only* used
  for the wordmark accent and large CTAs)

## 9. Implementation

### CLI
- `typer` for CLI + auto-help
- `rich` for color + tables (auto-detects TTY)
- `tqdm` for progress bars
- Python `logging` for logs; `rich.logging.RichHandler` for color in TTY mode

### Web UI (Phase 4, future)
- TBD — likely Tauri (desktop) or Next.js (web)
- Tailwind for CSS (with custom property tokens)
- shadcn/ui or similar for components

## 10. Asset Checklist

- [x] Logo (CLI + landing page) — `desktop/src-tauri/icons/wordmark-*.png`
- [x] App icon, macOS bundle (`.icns`) + Windows bundle (`.ico`)
- [x] Square tiles for Windows Store (Square30 → Square310)
- [x] Open Graph card (1200×630)
- [x] Brand palette + tokens (`desktop/src/styles/globals.css`)
- [ ] CLI demo GIF
- [ ] Web UI mockups (Phase 4)
- [ ] Architecture diagrams
- [x] Icon set (Phase 4 — lucide-react)

## 11. Brand Guide (summary)

**Mark.** House silhouette + JSON brace, three colors only. The brace
is the "extraction" half; the house is the "real estate" half. The
terracotta line at the base reads as a ground plane or a foundation.

**Palette logic.** Colors are physical, not synthetic — they exist in
the real estate built environment (forest, slate, terracotta, moss,
saffron, brick, paper, graphite). None of them are violet or purple.

**Voice.** Direct, technical, warm. The doc strings and CLI help
read like a senior engineer explaining a system to a peer. No
marketing copy.

**Typography.** System sans (offline-friendly, zero webfont weight).
Monospace for extraction values and confidence scores.

**Layout.** 4px grid, 1180px max content width, 220px sidebar
(when shipped), 64px header. Generous whitespace; no tight crammed
grids.

**Logo variations.**

- Full-color on light (default)
- Full-color on dark (inverts the brace to paper)
- Monochrome forest (one-color mark for stamps, watermarks)
- Monochrome paper (inverse, for dark backgrounds)

**Regeneration.** `python3 desktop/scripts/regen_icons.py` from the
repo root. Pure Pillow; no external services, no API calls. Safe to
run in CI.

---

*Design System v1.0 — RealEstate-Bench © 2026 ksjswaroop*
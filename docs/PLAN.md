# SLP — Simple Landscape Planner: Rust Rebuild Plan

A ground-up rebuild of the two interactive spikes in `spike/` (Deck Planner +
Paver Planner), **unified into one Leptos client-side WASM app**, delivered as
**vertical slices** (thin end-to-end features), using the toolchain and
conventions established across the sibling repos (`t2t`, `playwright-rust`,
`rust-project-template`, `panschema`).

## 1. Decisions (locked)

| Decision | Choice | Why |
|---|---|---|
| Backend scope | **CSR/WASM only** | The planner is client-side: drawing, drag/rotate, BOM math, JSON save/load, print. No server/DB for MVP. Mirrors `playwright-rust/crates/site`. |
| App shape | **Unified planner** | Both spikes share the feet coordinate system, deck/house geometry, catalogs, and interaction primitives. One canvas with a *deck layer* + a *yard layer*. |
| Plan-file format | **LinkML + panschema** | Versioned, validated `.slp.json`; generated `serde` types; dogfoods the toolchain (t2t convention). |
| Rendering surface | **2D SVG via Leptos RSX**, authoritative | Direct port of the spikes; vector + print-friendly; reactive nodes. |
| 3D | **Deferred view, designed-in now** | Landscape is 2.5D (footprint + height). 3D is an *extruded* second renderer over `slp-core`, not a parallel model. Schema carries height/elevation + material-ref from slice 1. 3D **view** lands ~slice 6; 3D **designer** is optional/later. |
| 3D engine | **Deferred** | `three-d` (viewer-friendly) default; `bevy` if the 3D designer becomes central; `wgpu` if we need low-level control. All WASM + Rust-only. Decide before slice 6. |
| Materials | **Manifest-driven catalog** | A material = {image(s), real dimensions, unit price, provenance}. Shapes reference a material by id (not inline color/price). Feeds both 2D tiling and 3D albedo. |
| Material ingestion | **Per-source adapters + manifest** | `slp-ingest` crate, one adapter per manufacturer site, pulls image-ref + dims + price from a product URL. |
| Ingested-asset IP | **Manifest committed, assets gitignored** | Commit only metadata + provenance + `license` + price + dims. Binaries live in a gitignored local cache (`materials/cache/`), never committed, never in a hosted bundle. Adapters respect robots.txt/ToS. **Never redistribute scraped/manufacturer binaries.** |
| E2E | **playwright-rust** | Copy `crates/site-e2e`: serve `dist/` via Axum `ServeDir`, drive with `playwright-rs`, trace + screenshots. |
| Scaffolding | **rust-project-template** | CI, dependabot, prek hooks, cargo-deny/audit/vet, mutants, edition 2024, MSRV pin, `unsafe_code = "forbid"`. |

## 2. Persona & primary goal

- **DIY homeowner couple** (the only real persona) — Paul + wife laying out
  their own backyard. The point of the tool is **deciding what to buy**:
  comparing products by *both* look and cost, on an accurate to-scale layout.
- **Estimate and visualization are co-primary.** Neither subordinate to the
  other — the purchase decision needs both an honest budget and a believable
  picture.
- **No handoff persona.** No contractor/dealer to satisfy, so **print/export
  (G2) is nice-to-have**, demoted to polish. Accurate take-off still matters —
  but for *our* budget, not a quote.

## 3. Crate layout

```
slp/
├── Cargo.toml                 # workspace (members: slp-core, slp-ui, dokime, theoria)
├── schema/slp.yaml            # LinkML: shapes, materials, settings, plan file
├── materials/
│   ├── manifest.toml          # COMMITTED: material metadata + provenance only
│   └── cache/                 # GITIGNORED: ingested image binaries, never committed
├── crates/
│   ├── slp-core/              # headless, WASM-free: geometry + take-off math
│   │   └── src/{generated/, geom.rs, takeoff.rs, material.rs}
│   ├── slp-ui/                # Leptos components, runtime-agnostic (member)
│   │   └── src/canvas/{grid,scale_bar,yard}.rs   (3D renderer later)
│   ├── dokime/                # DOGFOOD: component testing for Leptos (member, lib)
│   ├── theoria/               # DOGFOOD: component explorer UI for Leptos (member)
│   ├── slp-ingest/            # per-source material adapters -> manifest (later)
│   ├── slp-app/               # Leptos CSR app (Trunk target; selects csr; excluded)
│   │   └── src/{app.rs, gallery.rs (?gallery → theoria), panels/, io.rs}
│   └── slp-e2e/               # playwright-rust tests (excluded; serves dist/)
├── docs/PLAN.md               # this overview
└── docs/stories/              # one detailed doc per user story (slice breakdown)
```

`slp-core` is **renderer-agnostic** (SVG and the future 3D view are both
consumers) and **headless** so geometry + take-off math get fast native
unit/mutation tests, independent of the UI. Biggest correctness win over the
spikes, where math and DOM are entangled.

**dokime** and **theoria** are incubated **as crates in this workspace** and
dogfooded against `slp-ui` — the most efficient way to start (one build graph;
the real consumer drives the API) — then **spun out to their own repos** via
`git subtree split` once their APIs stabilize. Each is a TDD/component-driven
sub-project that follows this **same `PLAN.md` + `docs/stories/` convention** in
its own tree (`crates/<crate>/docs/`). theoria has a UI; dokime is a library.
dokime tests itself; theoria is tested *by* dokime (never theoria-in-theoria).

**Demand-driven:** theoria/dokime stories are **pulled by slp's needs** — a
capability is built only when an slp slice requires it (or dokime when slp *or*
theoria requires it), never speculatively.

### 3.1 Frontend: component-driven from the start

The UI is a tree of small, single-purpose Leptos `#[component]`s, each
developable and testable in isolation — never the spikes' one giant script.

- **Canvas tree (today):** `App → Yard → { Grid, ScaleBar }`, with a `Transform`
  (feet→px) passed as a prop. Grows with `Ground`, `Shape`, `DeckLayer`,
  `SelectionHandles`, then `Toolbar`, `SidePanel → { Estimate, SelectionCard,
  Legend }`.
- **Isolation tooling (dogfooded, in-repo crates):** **theoria** component
  explorer (preview a component without the whole app — served separately, *not*
  bundled into the production app), **dokime** for native component tests. dokime
  is in use today; theoria's gallery is served on demand (the Storybook-style
  `theoria serve` CLI is a theoria backlog story; a thin gallery bin is the
  interim stand-in).
- **State:** Leptos signals; the `Plan` (slp-core types) is the single reactive
  source the components render from.

### 3.2 Manual local testing

`trunk serve` from `crates/slp-app` gives a hot-reloading dev server at
`http://localhost:8080` (the planner) — edit a component, the browser refreshes.
That's the primary manual loop. The **theoria** gallery (for working a single
component in isolation) is served *separately* on demand, never bundled into the
app. See `CLAUDE.md` for all commands.

## 4. Domain model (LinkML → panschema)

- **Plan** — `name`, `yard {w,d}`, `deck_x`, `settings`, `shapes[]`,
  `furniture[]`, material-price overrides. **Clean-sheet format** designed
  around this schema and the 2.5D model — *no coupling to the spike JSON*
  (the two sample files are trivial to recreate, so no importer).
- **Shape** (tagged union by `kind`), **every shape carries `elevation` +
  `height` (default flat) and a `material_ref`** so 2D→3D is additive:
  - `Polygon` — paver area / mulch bed: `pts[]`, `material_ref`, `border`, …
  - `Polyline` — wall / edging: `pts[]`, `height`, `material_ref`
  - `Point` — step / tree / equipment: `x`, `y`, `rot`, `height`, type-specific
- **Material** (catalog record) — `id`, `name`, `category`, `dimensions`,
  `unit_price`, `price_unit`, `tile_ft`, `source_url`, `source`, `license`,
  `fetched_at`, `checksum`, `asset` (gitignored path). 2D tiles the albedo;
  3D uses it as the albedo map (normal/roughness optional later).
- **Settings** — gravel/sand/mulch depths, wall/edge $/ft², etc.

`panschema verify` in CI guards schema/codegen drift (t2t convention).

## 5. Backlog

The story catalog (epics → stories) and one detailed, slice-by-slice document per
story live in **[`docs/stories/`](stories/README.md)** — the single source of
truth for the backlog. This plan does not repeat it; §6 below only sequences
*which* stories ship together. The dogfood sub-projects keep their own
`PLAN.md` + `docs/stories/` at `crates/dokime/docs/` and `crates/theoria/docs/`.

## 6. Delivery order (milestones)

This only **orders the stories** — which ship together, in what sequence. Each
story's acceptance criteria and **vertical-slice breakdown live in its
`docs/stories/<ID>.md`**; this section deliberately does not repeat them.

Architectural note: milestones 1–5 do **nothing 3D** but stay **3D-ready**
(every shape carries height + material-ref; `slp-core` is renderer-agnostic), so
milestone 6 is an additive renderer, not a rewrite.

| # | Milestone | Stories |
|---|---|---|
| 0 | Walking skeleton (scaffold, CI/CD, yard renders, first e2e) ✅ | — |
| 1 | Draw a paver area & see cost | A1, B1, B2 |
| 2 | Edit & keep | F1, G1 |
| 3 | Materials, ingestion & comparison | M1–M5, B4 |
| 4 | Vertical hardscape | C1 |
| 5 | Objects | D1 |
| 6 | 3D view | R2 |
| 7 | Deck layer | E1, E2 |
| 8 | 3D designer + polish | R3, B5, G2 |

## 7. E2E approach (playwright-rust)

Copy `crates/site-e2e/tests/landing_page.rs`: build with `trunk build`, serve
`dist/` via Axum `ServeDir` on `127.0.0.1:0`, drive with `playwright-rs`
locators (auto-waiting, no sleeps). Because the app is CSR/WASM, the assertions
prove the bundle boots and widgets react. Step screenshots + `trace.zip` are
byproducts; assertions are the gate. Skips gracefully when `dist/` is absent.
CI adds `npx playwright@<ver> install chromium --with-deps`.

## 8. What carries over vs. rebuilt

**Carried over (as data/spec only):** feet coordinate system, deck/house
geometry constants, catalogs/prices (→ manifest), take-off formulas (yd³ =
sqft·in/324, face-area pricing, soldier course). **Not** the spike JSON format —
the new `.slp.json` is clean-sheet and owes it nothing.

**Rebuilt better:** tested headless core; reactive SVG vs string `innerHTML`;
one unified layered app vs two copy-pasted files; manifest-driven materials with
provenance vs inline JS dicts; typed/validated/versioned plan files; 2.5D data
model that extrudes to 3D.

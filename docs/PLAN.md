# SLP — Simple Landscape Planner: Rust Rebuild Plan

A Leptos client-side WASM app to plan a backyard landscape — a single unified
planner (a deck layer + a yard layer), delivered as **vertical slices** (thin
end-to-end features), using the toolchain and conventions established across the
sibling repos (`t2t`, `playwright-rust`, `rust-project-template`, `panschema`).

## 1. Decisions (locked)

| Decision | Choice | Why |
|---|---|---|
| Backend scope | **CSR/WASM only** | The planner is client-side: drawing, drag/rotate, BOM math, JSON save/load, print. No server/DB for MVP. Mirrors `playwright-rust/crates/site`. |
| App shape | **Unified planner** | The deck and yard share a feet coordinate system, geometry, catalogs, and interaction primitives. One canvas with a *deck layer* + a *yard layer*. |
| Plan-file format | **LinkML + panschema** | Versioned, validated `.slp.json`; generated `serde` types; dogfoods the toolchain (t2t convention). |
| Rendering surface | **2D SVG via Leptos RSX**, authoritative | Vector + print-friendly; reactive nodes. |
| 3D | **Deferred view, designed-in now** | Landscape is 2.5D (footprint + height). 3D is an *extruded* second renderer over `slp-core`, not a parallel model. Schema carries height/elevation + material-ref from slice 1. 3D **view** lands ~slice 6; 3D **designer** is optional/later. |
| 3D engine | **Deferred** | `three-d` (viewer-friendly) default; `bevy` if the 3D designer becomes central; `wgpu` if we need low-level control. All WASM + Rust-only. Decide before slice 6. |
| Materials | **Manifest-driven catalog** | A material = {image(s), real dimensions, unit price, provenance}. Shapes reference a material by id (not inline color/price). Feeds both 2D tiling and 3D albedo. |
| Material ingestion | **Per-source adapters + manifest** | `slp-ingest` crate, one adapter per manufacturer site, pulls image-ref + dims + price from a product URL. |
| Ingested-asset IP | **Manifest committed, assets gitignored** | Commit only metadata + provenance + `license` + price + dims. Binaries live in a gitignored local cache (`materials/cache/`), never committed, never in a hosted bundle. Adapters respect robots.txt/ToS. **Never redistribute scraped/manufacturer binaries.** |
| E2E | **playwright-rust** | Copy `crates/site-e2e`: serve `dist/` via Axum `ServeDir`, drive with `playwright-rs`, trace + screenshots. |
| Scaffolding | **rust-project-template** | CI, dependabot, prek hooks, cargo-deny/audit/vet, mutants, edition 2024, MSRV pin, `unsafe_code = "forbid"`. |

## 2. Persona & primary goal

- **DIY homeowner** (the target user; Paul + wife are the first users and the
  impetus) — laying out *their own* backyard to **decide what to buy**, comparing
  products by *both* look and cost on an accurate to-scale layout.
- **General-purpose — nothing is hardcoded to one property.** The user *draws and
  saves* their own yard, house (walls + doors + windows), deck(s), and everything
  else. No fixed measurements or specific house/yard/deck baked into the app; it
  works for any property.
- **Estimate and visualization are co-primary.** The purchase decision needs both
  an honest budget and a believable picture.
- **No handoff persona.** No contractor/dealer to satisfy, so **print/export
  (G2) is nice-to-have**, demoted to polish. Accurate take-off still matters —
  for the homeowner's budget, not a quote.

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
unit/mutation tests, independent of the UI — keeping the math decoupled from the
DOM.

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
developable and testable in isolation — never one monolith.

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

- **Plan** — `name`, `yard {w,d}`, `structures` (house + deck(s)), `shapes[]`,
  `objects[]`, materials/price overrides, `settings`. **Clean-sheet format**;
  **everything is user-drawn — nothing is property-specific.** (Started minimal —
  `name` + yard size — and grows per slice.)
- **Structures (drawn and saved, like everything else)** —
  - **House** — an outline drawn as wall segments, with **doors** and **windows**
    placed along the walls (position + width). Context for planning around
    entries and sightlines; not costed.
  - **Deck(s)** — a footprint polygon (optionally multi-level, with stairs and
    railing), drawn by the user.
  These are normal plan entities (drawn, saved, edited) — *not* hardcoded
  geometry — and carry a **status** (below), defaulting to **existing**.
- **Item status — two independent axes** (nothing is hardcoded; both are
  per-item flags):
  - **status** (`planned` | `existing`): `planned` is to buy/build, `existing`
    is already owned/built. Both are *real*.
  - **virtual** (`is_virtual`, objects only): a what-if ghost duplicate at an
    *alternate* position — never a second real item. Structures are always real
    (no virtual variant); an object crosses the two axes freely (planned/
    existing × real/virtual).
  Take-off counts an object only when **planned *and* real** — existing (already
  owned) or virtual (a ghost) is excluded regardless of the other flag. On the
  canvas the axes read separately: line count encodes status (single = planned,
  double = existing) and line style encodes realness (solid = real, dashed =
  virtual). *(Structure status is carried but not yet cost- or render-differentiated.)*
- **Shape** (tagged union by `kind`), **every shape carries `elevation` +
  `height` (default flat) and a `material_ref`** so 2D→3D is additive:
  - `Polygon` — paver area / mulch bed: a ring of nodes, `material_ref`,
    `border`, …. Each *edge* (node→next) carries its own kind — straight
    (default), **arc** (a signed bulge factor), or **bezier** (two control
    points) — so one boundary can freely mix straight/curved/arced edges (see
    [F3](stories/F3-draw-edit-shapes.md))
  - `Circle` — a round area or footprint: `center`, `radius_ft`,
    `material_ref` — a plain primitive, not a polygon trick
  - `Polyline` — wall / edging: `pts[]`, `height`, `material_ref`
  - `Point` — step / tree / equipment: `x`, `y`, `rot`, `height`, type-specific
    *(discrete objects like furniture/fire-pit/trees are actually `Object` +
    `CatalogItem`, generated today, not this `Point` shape — see the
    `CatalogItem` note below for the same kind of drift)*
- **Material** (catalog record) — `id`, `name`, `category`, `dimensions`,
  `unit_price`, `price_unit`, `tile_ft`, `source_url`, `source`, `license`,
  `fetched_at`, `checksum`, `asset` (gitignored path). 2D tiles the albedo;
  3D uses it as the albedo map (normal/roughness optional later).
  *(`CatalogItem`, generated today, is this record's discrete-object subset —
  `id`/`name`/`category`/`shape`/`width_ft`/`depth_ft`/`height_ft`/
  `unit_price`/`clearance_ft`/`trunk_diameter_ft` — grown ad hoc per slice
  (E1, D1, D2). It's missing `price_unit` and every provenance field; those
  land with [M4](stories/M4-M5-material-ingestion.md), which is also where the
  two names converge — `CatalogItem` may just become `Material`, or gain these
  fields under its own name, whichever the schema work there decides.)*
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

Architectural note: every milestone before the **3D view** does nothing 3D but
stays **3D-ready** (every shape carries height + material-ref; `slp-core` is
renderer-agnostic), so the 3D view is an additive renderer, not a rewrite.

The order follows the **way a yard is actually built up** — the fixed structures
first (house, then deck), then the things you place into it, roughly in the
user's purchase-decision order: furniture, fire pit, trees, mulch, bushes,
pavers, grills, hot tubs. Two cross-cutting **enablers** are folded in (not part
of that list, but required by it): once you've drawn the house & deck you need to
**edit/save** them (F1, G1), and the first *costed catalog object* (furniture)
forces the **materials/catalog + cost engine** (M1–M3) to land with it.

| # | Milestone (what you can do) | Stories |
|---|---|---|
| 0 | Walking skeleton (scaffold, CI/CD, yard renders, first e2e) ✅ | — |
| 1 | Set the yard + **draw the house** (walls, doors, windows) | A1, H1 |
| 2 | **Draw the deck** (footprint, stairs, railing) | H2 |
| 3 | *Enabler:* edit what you drew + save/load the plan file | F1, G1 |
| 4 | *Enabler + first object:* catalog & cost engine, **place furniture** | M1–M3, E1 |
| 5 | **Fire pit** | D2 |
| 6 | **Trees** | D1 |
| 7 | *Enabler:* freeform shapes (boundaries, arcs, curves, circles) + **mulch beds** (volume & cost) | F3, B4 |
| 8 | **Bushes / shrubs** | D3 |
| 9 | **Paver areas** (ft² & cost) | B1, B2 |
| 10 | **Grills** | D4 |
| 11 | **Hot tubs** | D5 |
| later | 3D view, material ingestion & swap-&-compare, per-area composition & material images, vertical hardscape, soldier-course border, deck presets, print | R2–R3, M4–M5, B3, C1, B5, E2, G2 |

All of items 4–11 (furniture, fire pit, trees, bushes, grills, hot tubs, and
paver/mulch areas) share one capability — **place/draw an item from the catalog
and see its look + cost** — so each later milestone is mostly a new catalog
category + any item-specific geometry (point, footprint, or area), not new
machinery.

## 7. E2E approach (playwright-rust)

Copy `crates/site-e2e/tests/landing_page.rs`: build with `trunk build`, serve
`dist/` via Axum `ServeDir` on `127.0.0.1:0`, drive with `playwright-rs`
locators (auto-waiting, no sleeps). Because the app is CSR/WASM, the assertions
prove the bundle boots and widgets react. Step screenshots + `trace.zip` are
byproducts; assertions are the gate. Skips gracefully when `dist/` is absent.
CI adds `npx playwright@<ver> install chromium --with-deps`.

## 8. Key design choices

- **Feet coordinate system** throughout; SVG is a pure view transform over it.
- **Take-off math in `slp-core`** (headless, unit/mutation-tested): yd³ =
  ft²·in/324, face-area pricing, soldier-course borders.
- **Manifest-driven materials** with provenance (`materials/manifest.toml`);
  binaries never committed.
- **Reactive SVG** (Leptos nodes), not string templating.
- **Typed, validated, versioned plan files** via the LinkML schema + panschema.
- **2.5D data model** (footprint + height + material-ref) so the 2D plan
  extrudes to a 3D view without a rewrite.

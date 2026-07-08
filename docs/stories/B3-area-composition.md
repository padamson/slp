# B3 — Compose an area from real materials (per-area build-up)

*Epic B — Hardscape areas · yard layer. Extends [B2](B2-area-cost.md): B2.2 gave
a paver its base + bedding courses, but fixed **per paver type** on the catalog
item. B3 moves the build-up **per area** — each patio picks its own materials and
thicknesses — and pairs it with real, pictured materials from
[M4](M4-M5-material-ingestion.md) (add-a-material + `asset` images, whose
visualization/tiling lives in [M4.4](M4-M5-material-ingestion.md)).*

## Story

As a DIY homeowner, I want to build each area up from the exact materials I'll
buy — this patio on 6″ of one gravel, that patio on 4″ of another — choosing from
materials I've added myself (with a photo), so the estimate matches my real cart
and each area is composed independently, not locked to one recipe per paver type.

## Vertical slices

- **B3.0 — add & author materials** ✅ *(prereq; advances [M4.2](M4-M5-material-ingestion.md)'s
  manual-authoring path)*
  - [x] an **"+ Add"** button in the catalog inspector hand-adds a new catalog
        item (first free `material-N` id, then edited like any item) — so there's
        a *second* gravel / an alternate paver to compose with; persists with the
        plan (e2e-covered), no ingestion/network needed
  - [x] a **`price_unit` control** (the `SelectField` from B3.3, pulled forward)
        so a hand-added material can be per-item / per-ft² / per-yd³ /
        per-linear-ft — a gravel is per-yd³, a paver per-ft² ([M4.3](M4-M5-material-ingestion.md)'s
        deferred bit, now done)
- **B3.1 — per-area composition (schema + core)** ✅
  - [x] `Shape`/`Circle` gain `courses: Vec<Course>` (`Course = {material_ref,
        depth_in}`) — the ordered sub-layers beneath the surface `material_ref`
  - [x] `take_off` costs each area's **own** courses per material
        (`yd³ = ft²·in/324`); an area with empty `courses` falls back to the
        catalog paver's base/bedding template (B2.2), so existing plans keep
        costing correctly — unit-tested (two patios on different gravels;
        courses override the template; circle areas too) + mutation-tested
- **B3.2 — seed a default composition on draw**
  - [ ] drawing a paver area copies the catalog paver's default courses (gravel
        base 4″ + bedding sand 1″) into the area's `courses`, so it starts right
        and is then edited independently
- **B3.3 — `SelectField` primitive** ✅ *(built with B3.0)*
  - [x] a controlled labeled dropdown (`(value, label)` options, selected value
        server-rendered) — the string-choice counterpart of `NumberField`/
        `TextField`, reused by B3.0's `price_unit` control and (next) the
        composition editor's material picker
- **B3.4 — composition editor in the area inspector**
  - [ ] a paver area's inspector shows its **course list** — one row per layer
        (material `Select` + thickness `NumberField`), with add/remove-layer
  - [ ] editing a course recomputes that area's cost and the estimate live
        (extends [B2.3](B2-area-cost.md)); replaces B2.2's catalog-level base/
        bedding fields
- **B3.5 — e2e + verify**
  - [ ] two paver areas set to different gravels/thicknesses → the estimate
        itemizes each gravel separately with the right volume, and editing one
        area's composition leaves the other untouched

## Notes / refs

- **This is the per-area generalization of [B2.2](B2-area-cost.md).** B2.2's
  catalog-level `base_material_ref`/`base_depth_in`/`bedding_*` become the
  *default template* a new area copies from; the area then owns its `courses`.
  Catalog-vs-instance mirrors how a tree's canopy is a catalog default with a
  per-object override — editing the catalog default doesn't rewrite existing
  areas (they hold their own copy), which is the intended behavior.
- **Cost math stays in `slp-core::takeoff`** — `material_volume` already sums a
  per-yd³ material's volume across areas; B3.1 just sources the layers from each
  area's `courses` instead of the catalog item. Mulch beds stay single-layer
  (their `depth_in`); a general `courses` list could later subsume that, but B3
  doesn't force it.
- **Lighter alternative** if the general layer stack is more than wanted: keep
  the fixed base+bedding model and make just those two materials + depths
  *per-area overrides* (no add/remove rows). Covers the two-patio example with
  less UI; `courses` is the future-proof version and the recommended one.
- **Real, pictured materials** — adding a material with a photo, showing it as a
  swatch in the picker/inspector, and tiling it across the drawn surface — is the
  Materials side, storied in [M4.4](M4-M5-material-ingestion.md) (the catalog
  record's `asset`, per `PLAN.md` §2 "a material = {image(s), dims, price,
  provenance}", feeding 2D tiling now and 3D albedo later). B3.0 is where a
  hand-added material first gets its image attached.
- **2.5D-ready.** A course's `depth_in` is the layer's extruded thickness and a
  surface's tiled texture its face — both already feed the future 3D view, no
  rework.

# E1 — Place deck furniture (look + cost)

*Epic E — Objects placed from the catalog. Folds in the **M1–M3** enabler
(materials/catalog + cost engine): furniture is the first costed catalog object,
so the catalog + take-off machinery lands with it. Reuses the
[node-placement engine](H1-draw-house.md) for placement.*

## Story

As a DIY homeowner, I want to place furniture from a catalog onto my plan and see
its footprint to scale and its cost, so that I can decide what to buy — for **any**
product, since the catalog and placements are saved in the plan, not hardcoded.

## Vertical slices

- **E1.0 — catalog + cost engine (headless, `slp-core`)** ✅ *done*
  - [x] schema grows a `CatalogItem` (id, name, category, unit_price, footprint
        `width_ft`/`depth_ft`, `height_ft`) and an `Object` (catalog_ref, x, y,
        rot, status), wired into `Plan` as `catalog[]` + `objects[]`; generated
        into `slp-core` (the `virtual` status escapes to `r#virtual`, wire name
        unchanged)
  - [x] `ItemStatus` enum (planned / existing / virtual); take-off counts only
        **planned** (excludes existing + virtual), per the domain rule
  - [x] `takeoff::take_off(&Plan)` returns a bill of materials (per catalog item:
        qty, unit_price, line total) + grand total — a pure fn, unit + mutation
        tested; unresolved `catalog_ref`s are excluded
- **E1.1 — place + render furniture** ✅ *done*
  - [x] a furniture `Tool` places an `Object` at a clicked point (catalog item
        chosen from a picker); the footprint renders to scale inside `Yard`
  - [x] the placement is saved to the `Plan` and survives a reload
  - [x] an object whose footprint isn't fully inside a single deck/paver surface
        is outlined in red, so it's obvious what doesn't fit (paver surfaces join
        the check when that slice lands)
- **E1.2 — estimate panel**
  - [ ] a side panel shows the BOM (line items + total), reacting live as
        furniture is placed/removed
- **E1.3 — status + rotation + e2e**
  - [ ] per-object **existing/virtual** toggle removes it from the estimate
  - [ ] rotate a placed object; footprint reflects `rot`
  - [ ] e2e: place furniture → footprint renders + estimate updates

## Notes / refs

- **Catalog lives in the plan** (`Plan.catalog[]`); objects reference items by
  `id`. Keeps `.slp.json` self-contained (save/load just works) and honors
  "nothing hardcoded". The committed `materials/manifest.toml` + `slp-ingest`
  become an *import-into-the-plan's-catalog* path later (M4–M5), not a competing
  source of truth.
- **Starter catalog:** a small furniture catalog is seeded into the plan the
  first time a deck is drawn (the surface furniture sits on) — plan data the user
  can place, ignore, or replace, like the default yard size, not hardcoded
  geometry. Catalog *authoring* (add / edit / import) is its own later slice.
- **Cost model:** furniture is **count × unit_price**. Later categories add
  area/volume pricing (pavers ft², mulch yd³) — same BOM/total shape, new
  per-category math in `slp-core`.
- Footprint dims + `height_ft` are carried now (unused by cost) so the 2D render
  and the future 3D view need no schema churn — 2.5D from the start.
- Enablers F1 (select/move/delete) and G1 (save/load `.slp.json`) are sequenced
  near here in PLAN §6 but are separable; placement reuses the existing
  reload-persistence the house/deck already use.
- **`status` default:** schema-encoded — the `status` slot declares
  `ifabsent: ItemStatus(planned)`, so an absent status deserializes to `planned`
  from a single source of truth (`status` is a non-`Option` field with a serde
  default; the cost engine needs no None-means-planned convention).

# D3 — Bushes / shrubs (a planted-on-ground catalog object)

*Epic D — placed catalog objects. Reuses everything E1/F1/F2/D1 built (catalog,
palette, placement, inspector, estimate, status/virtual, move/delete, the
round-object + category-look + on-ground machinery). A bush is the simplest
member of the family: a round mass of foliage with **no trunk** (unlike a
tree), placed on **open ground** — the yard or a mulch bed, not the house, a
deck, or a paver. It needs **no new schema and no new core geometry** — just a
`bush` category, a distinct green look, and a `bush` arm on the shared
category-aware placement-validity rule.*

## Story

As a DIY homeowner, I want to place shrubs — round and to scale, a leafy green
that reads as a plant, and flagged if I've dropped one onto hardscape it
shouldn't sit on — so that I can see how much space they fill, budget for them,
and put them in the beds/ground where they belong before I buy.

## Vertical slices

- **D3.0 — bushes as round, on-ground catalog objects** ✅
  - [x] a few starter shrubs seeded into the catalog (category `bush`; round
        footprint, spread as `width_ft`, a representative price) — they get
        their own **palette group** ("Bush") for free (the palette groups by
        `category`, humanized), and seed on load like every other starter item
  - [x] a bush renders as a single **filled green canopy** (distinct from a
        tree's translucent canopy, and with **no trunk** — a shrub is a solid
        mass of foliage); the fill/stroke live in `slp-ui/src/style.rs`
  - [x] status/virtual/selection still read through (a virtual bush is a dashed
        ghost, an existing one keeps its treatment) — the green look is a
        *category* layer under those independent channels, exactly as a tree's
        canopy is
  - [x] a bush belongs on **open ground**: its **whole footprint** turns red
        when it overlaps the **house**, a **deck**, or a **paver** (hardscape a
        shrub shouldn't be planted on); it's quiet on the bare yard or a mulch
        bed. This is the shared `furnishings.rs::category_ground_invalid` model
        (a `bush` arm) reusing `slp_core::circle_overlaps_polygon` — no new core
        geometry, and no per-object trunk (unlike a tree, whose *trunk* is the
        checked disk)
  - [x] placing/costing is the existing machinery: click a palette tile to arm,
        click the yard to place; the estimate gains a per-item line;
        move/delete/select/status all work as for any placed object
  - [x] dokime: a bush renders a green canopy and **no trunk** (a tree renders a
        trunk, a bush does not); the whole footprint is flagged when on a
        surface. e2e: place a bush from the palette → a round green footprint +
        the estimate updates; dragged onto the deck it goes red; back on bare
        ground it clears

## Notes / refs

- **No new schema.** A bush reuses `CatalogItem`'s existing fields — `category`
  (`bush`), `shape` (`circle`), `width_ft` (spread), `height_ft`, `unit_price`.
  Per-object resize (the way a tree's canopy/trunk are adjustable, D1.2) is
  **not** part of D3.0 — a bush is sized by its catalog item (editable in the
  catalog inspector, which already propagates to every placement); add a
  per-bush size slice later only if a real need shows up.
- **Category-aware placement validity is the shared enabler** (D1.3, D2.2). Each
  category's rule is a plain, unit-tested `slp-ui` function
  (`category_ground_invalid`) over whether the object's disk overlaps the house
  or a deck/paver surface (`slp_core::point_in_polygon`/`circle_overlaps_polygon`,
  already mutation-tested). D3 adds one arm:
  - *bush* — its **whole footprint** must **not** overlap the house, a deck, or
    a paver (OK on the yard or a mulch bed). Mirrors a fire pit's whole-footprint
    flag, but extended to the deck/paver too (a fire pit only forbids the house).
- **Look.** Denser and more opaque than a tree's translucent canopy, so a shrub
  reads as solid foliage rather than a tree's airy crown; a distinct green from
  the tree canopy so the two don't blur together on a busy plan.

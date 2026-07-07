# M4–M5 — Material ingestion, catalog inspector, swap-&-compare

*Enablers M4–M5 — growing the catalog beyond the hand-seeded starter data
([`starter_catalog()`](../../crates/slp-ui/src/components/planner.rs)) into
something the user builds up themselves: pull real products in from
manufacturer/retailer sites, review and edit what came in before it's usable,
and compare alternatives against what's already placed. Per `PLAN.md` §2, this
is a separate `slp-ingest` crate (per-source adapters), never `slp-core`/
`slp-ui` — ingestion is a data-gathering concern, not planner logic.*

## Story

As a DIY homeowner, I want to pull real products (with real prices, dimensions,
and photos) into my catalog from the sites I actually shop at, review and correct
what came in, and swap one item for an alternative to compare cost and fit —
so that my estimate reflects things I can actually buy, not placeholder data.

## Vertical slices

- **M4.0 — `CatalogItem` grows into a `Material` record**
  - [ ] schema gains the provenance/measure fields `PLAN.md` §4 already
        anticipates: `price_unit` (per-item vs per-ft²/yd³/linear-ft — needed
        for [B2](B2-area-cost.md)'s area/volume costing, not just discrete
        objects), `source_url`, `source`, `license`, `fetched_at`, `checksum`,
        and `asset` (a path into gitignored `materials/cache/`, never
        committed)
  - [ ] existing categories (furniture, fire pit, tree) are unaffected — the
        new fields are all optional, same pattern as `clearance_ft`/
        `trunk_diameter_ft`
- **M4.1 — `slp-ingest`: per-source adapters (headless)**
  - [ ] a new crate, one adapter per manufacturer/retailer site: given a product
        URL, pull name/category/dimensions/price/image-ref
  - [ ] each source's `robots.txt`/ToS is respected before fetching; a source
        that disallows it is refused, not worked around
  - [ ] the fetched image is cached to gitignored `materials/cache/`, never
        committed or redistributed; metadata (+ provenance: `source_url`,
        `fetched_at`, `checksum`) is written to the committed
        `materials/manifest.toml`
- **M4.2 — human-in-the-loop review (nothing ingested is live automatically)**
  - [ ] an ingested item lands in a **staging** list, separate from the plan's
        live catalog — pulling it in never silently changes an existing plan
  - [ ] the user reviews each staged item (photo, parsed metadata) and either
        **approves** it (moves into the plan's catalog, placeable like any
        starter item) or **rejects** it (discarded, cache entry cleaned up)
  - [ ] a parse that comes back incomplete (missing price/dimensions) is
        flagged for the user to fill in during review, not silently guessed
- **M4.3 — catalog inspector (edit any catalog item's metadata)** ✅ *partly done*
  - [x] a catalog-browsing/editing panel (`CatalogPanel`) — the catalog-side
        counterpart to [`ObjectInspector`](E1-place-furniture.md) (which edits a
        *placed* object): a toolbar-toggled drawer listing every item in the
        plan's catalog, select one, edit its name/category/price and footprint
        dimensions (width/depth/height) directly. Built on a new reusable
        `TextField` primitive (the string counterpart of `NumberField`).
  - [x] `price_unit` is now editable via a `SelectField` dropdown
        ([B3.0](B3-area-composition.md)); image editing is still open (M4.4)
  - [x] editing a catalog item's footprint/price updates every object already
        placed from it (they reference it by `catalog_ref`, not a copy) — the
        estimate reprices and the footprint re-renders live (e2e-covered)
  - [ ] deleting a catalog item the plan has no objects placed from removes it
        outright; one that's in use is blocked (or asks to remove the
        placements first) — never a dangling `catalog_ref` (M4.3b, next)
  - [ ] **add** a new catalog item by hand (name/category/price/`price_unit`/
        dimensions) — the manual-authoring entry point (also
        [B3.0](B3-area-composition.md)); pairs with the `price_unit` `Select`
        from [B3.3](B3-area-composition.md)
- **M4.4 — material images (visualization + surface tiling)** — *`PLAN.md` §2:
  "a material = {image(s), real dimensions, unit price, provenance}", feeding
  "both 2D tiling and 3D albedo".*
  - [ ] a material carries an image: the `asset` ref (M4.0) plus a **real-world
        tile size** (how many feet the image spans, for scale). When adding/
        editing a material, attach an image — a user upload or a URL/cache path.
        [design fork: a small thumbnail inline (data-URI, portable through
        `localStorage`/export) vs. a `materials/cache/` path or IndexedDB blob
        for large/ingested assets — recommend inline thumbnail + cache-path for
        full-res; never commit the binary]
  - [ ] **thumbnail**: the flat color swatch in the catalog panel, the Area
        tool's material picker, and the area inspector becomes the actual
        material photo when one is present (flat color as fallback)
  - [ ] **surface tiling**: a drawn area (paver/mulch) fills with its surface
        material's image tiled as an SVG `<pattern>` at real-world scale
        (`patternUnits="userSpaceOnUse"`, tile = tile-size-ft × px_ft), so a
        2×2 ft sample repeats to scale across the polygon; flat color when there's
        no image. 2.5D-ready — the same texture is the 3D surface albedo later.
- **M5.0 — swap & compare**
  - [ ] from a selected placed object, browse the catalog for an alternative in
        the same category and preview the swap (footprint + cost delta) before
        committing
  - [ ] confirming re-points that object's `catalog_ref` (position/rotation/
        status untouched) instead of delete-and-replace, so it stays the same
        plan entity
  - [ ] e2e: swap a placed chair for a pricier one → the footprint updates to
        the new item's size and the estimate reflects the new price

## Notes / refs

- **`slp-ingest` is its own crate, not `slp-core`/`slp-ui`.** It fetches from
  the network and writes to `materials/`; `slp-core` stays headless/
  network-free and `slp-ui` stays a pure render/interaction layer. Ingestion
  populates the same `CatalogItem`/`Material` data both already consume — no
  new consumption-side machinery.
- **Nothing ingested is live until approved (M4.2).** This is the whole point
  of the staging step: a scrape can misparse a price or dimension, and this
  app's estimate is the thing the user is trusting to decide what to buy —
  wrong data landing silently in a live plan is the failure mode to design
  against.
- **The catalog inspector (M4.3) is also where hand-authored items belong** —
  a user typing in a product they measured themselves rather than any site's
  listing. Ingestion (M4.1) and manual authoring both feed the same staging →
  approve → catalog pipeline; M4.3 doesn't care which one produced an item.
- **The custom catalog needs no new persistence.** `Plan.catalog` already
  round-trips through `localStorage` and (once [G1](G1-save-load-plan.md)
  lands) the exported file — an ingested/edited catalog is saved and loaded
  with everything else automatically.
- **Category-driven look/behavior should stay data-first as categories grow.**
  `Furnishings`' category dispatch (a tree's canopy+trunk render, a fire pit's
  silver fill and ground rule) is currently a `match` on category strings in
  `slp-ui`. Every new ingested category that wants anything other than the
  generic look needs a Rust change today; before M4 lands enough categories to
  make that painful, consider whether render/placement-rule hints belong on
  the catalog item itself (data) rather than in component code.
- Respect each source's `robots.txt`/ToS; never redistribute scraped images —
  already stated in the repo's top-level `CLAUDE.md` and unchanged here.

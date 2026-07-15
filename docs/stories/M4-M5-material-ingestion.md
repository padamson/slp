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

- **M4.0 — `CatalogItem` grows into a `Material` record** ✅
  - [x] schema gains the provenance/measure fields `PLAN.md` §4 already
        anticipates: `price_unit` (per-item vs per-ft²/yd³/linear-ft — needed
        for [B2](B2-area-cost.md)'s area/volume costing, not just discrete
        objects; landed with B4), `source_url`, `source`, `license`,
        `fetched_at`, `checksum`, and `asset` (a path into gitignored
        `materials/cache/`, never committed — the inline swatch stays `image`)
  - [x] existing categories (furniture, fire pit, tree) are unaffected — the
        new fields are all optional, same pattern as `clearance_ft`/
        `trunk_diameter_ft` (whole workspace stays green). A plan round-trip
        test (`plan_file.rs`) proves an ingested item's provenance persists
        through save→load and that an item without it omits the keys from the
        file.
- **M4.1 — screenshot ingestion (vision extraction, in the browser app)**
  - *Design pivot (2026-07-14):* the original plan was a headless `slp-ingest`
    crate with per-site HTML adapters. Reality killed it: the target sites are
    Cloudflare-protected and **block headless automation**
    ([[techo-bloc-cloudflare-blocks-scraping]] — the `/all-products` listing
    loaded once, but every leaf product page returned Cloudflare's "you have
    been blocked"). So acquisition moves to **the user's own browser**: they
    screenshot a product page (⌘⇧4 → paste) and a **vision model reads the
    image** into a draft. This is **site-agnostic** — one extractor for every
    site — so it also deletes the per-site adapter + synthetic-fixture +
    weekly-contract-test machinery entirely. Extraction lives in `slp-ui`
    (browser), not a headless crate.
  - **B1 — API-key config + gating** ✅ *(in progress)*
    - [ ] the vision feature gates on a user-supplied Anthropic API key, stored
          in **localStorage** as app config (`slp.anthropicKey`) — deliberately
          **not** in the `Plan`/`.slp.json`, so a shared plan file never leaks a
          billable secret. No key → the extract affordance is disabled with an
          "add your key" note (mirrors the Save-As-gates-on-FSA pattern).
    - [ ] a **dev-only** `option_env!("SLP_ANTHROPIC_KEY")` seeds localStorage
          when empty, for a local `trunk serve` (gitignored `.env`); **never**
          set in the hosted/CI build, or the key ships in the public WASM.
  - **B2 — clipboard paste** — paste a screenshot into the catalog inspector
    (Clipboard API, `csr`-gated) → a `data:` URI draft image, the analogue of
    the existing `FileInput` file-read.
  - **B3 — vision extract call** — browser-direct Anthropic request
    (`anthropic-dangerous-direct-browser-access`) with the pasted image →
    a structured **draft product** JSON: the shared fields (name, category,
    price_unit, thickness→`depth_in`, tile size) plus the **full variant
    matrix** — every color × texture × size the page shows, with **availability**
    (greyed = unavailable). Prompt encodes the real-page lessons: **prices are
    usually absent** (manufacturer→dealer) so never guess one; prefer
    **imperial** dims → feet. Default a cheap vision model (Haiku 4.5 / Sonnet
    5), configurable; the user pays via their own key.
    - *Modeling decision (2026-07-15): **multi-select**, not one-item.* A page is
      a configurator (color × texture × size), and the user wants to add several
      combinations to the catalog to **compare in the viz**. So extraction
      returns the whole matrix and curation (M4.2) lets the user tick multiple
      combos → **one `CatalogItem` per ticked combo** (distinct id/name/`image`
      swatch, shared base fields), each a placeable/tileable material.
- **M4.2 — human-in-the-loop curation (nothing ingested is live automatically)**
  - [ ] the extracted draft product lands in a **staging** review, separate from
        the plan's live catalog — extracting never silently changes a plan
  - [ ] the user **multi-selects** which variant combos to keep (tick colors ×
        sizes), edits shared/per-variant fields, and **approves** — each ticked
        combo becomes its own catalog `CatalogItem` (provenance stamped),
        placeable/tileable to compare in the viz; or **rejects** (discarded)
  - [ ] a draft that comes back incomplete (**missing price** is the norm, or
        missing dimensions) is flagged for the user to fill in during review,
        **never silently guessed** — the estimate has to stay trustworthy; a
        "same price for all variants" shortcut saves re-typing a dealer quote
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
  - [x] a material carries an image: `CatalogItem.image` (a `data:` URI or URL)
        plus `tile_width_ft` + `tile_depth_ft` (the photo's **real-world E–W ×
        N–S span**, so a rectangular unit like a paver tiles undistorted). The
        catalog editor has an Image field + Tile W/D fields and previews the set
        image; it persists with the plan (e2e-covered). Storage decision:
        user-added images are stored **inline as a data-URI** (round-trips
        through `localStorage`/export); the M4.0 `asset` cache-path stays for
        *ingested* full-res binaries (M4.1), never committed.
  - [x] attach via **file upload**: a reusable `FileInput` primitive reads the
        picked file to a data-URI (browser `FileReader`, `csr`-gated; a no-op on
        SSR) and sets the image — so you pick a photo file rather than pasting a
        URI. e2e-covered via an in-memory `FilePayload` (no fixture on disk).
  - [x] **thumbnail**: a reusable `MaterialSwatch` (photo when the material has
        an image, else a flat square in its category color — the same
        `area_style` mapping the canvas fill uses) now appears in the catalog
        list row, the Area tool's material picker (live from the catalog
        signal), and the area inspector's material row. e2e: set a material's
        photo → its swatch becomes that photo in all three places.
  - [x] **surface tiling**: a drawn area (paver/mulch) fills with its surface
        material's image tiled as an SVG `<pattern>` at real-world scale
        (`patternUnits="userSpaceOnUse"`, tile = tile-width-ft × tile-depth-ft ×
        px_ft, default via `slp_core::tile_size_ft` so a future 3D albedo
        resolves the same size), so a 2×2 ft sample repeats to scale across the
        polygon *and* the round `Circle` areas. One pattern per **material** (not
        per area — a photo's data-URI is embedded once however many areas share
        it), anchored at the world origin so tiles stay glued to world
        coordinates when the yard resizes. Flat category color when there's no
        image; selection tint (translucent) overrides the texture while
        selected. Shared `texture_patterns`/`surface_fill` helpers back both
        `Shapes` and `Circles`; one e2e draws a textured polygon *and* circle.
        Textured surfaces render opaque by design (a real material occludes the
        grid/deck beneath; select the area to see through it while editing).
- **M5.0 — swap & compare** — *nice-to-have; deferred. Not on the critical
  path to "decide what to buy": the estimate already reprices live when a
  catalog item is edited (M4.3), so comparing alternatives works today by
  editing the item or drawing a second option. Pick this up only if in-place
  swap-and-preview earns its keep.*
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

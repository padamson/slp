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
  - **B1 — API-key config + gating** ✅
    - [x] the vision feature gates on a user-supplied Anthropic API key, stored
          in **localStorage** as app config (`slp.anthropicKey`) — deliberately
          **not** in the `Plan`/`.slp.json`, so a shared plan file never leaks a
          billable secret. No key → the extract affordance is disabled with an
          "add your key" note (mirrors the Save-As-gates-on-FSA pattern).
          E2e-covered: gating, persistence across reload, key absent from the
          plan autosave.
    - [x] a **dev-only** `option_env!("SLP_ANTHROPIC_KEY")` seeds localStorage
          when empty, for a local `trunk serve` (gitignored `.env`); **never**
          set in the hosted/CI build, or the key ships in the public WASM.
  - **B2 — clipboard paste** ✅ — paste a screenshot into the catalog inspector
    (Clipboard API, `csr`-gated) → a `data:` URI draft image, the analogue of
    the existing `FileInput` file-read; previewed with a Clear action.
  - **B3 — vision extract call** ✅ — browser-direct Anthropic request
    (`anthropic-dangerous-direct-browser-access`) with the pasted image, forcing
    a **tool call** whose `input` is validated against a **JSON Schema** (so the
    model returns structured data, not free-form text to parse). The schema is
    derived from the `ExtractedProduct` Rust type via `schemars` (one source of
    truth); its **field descriptions carry the rules** and `price_unit` is a
    closed **enum**. Extracts the shared fields (name, category, price_unit) plus
    the **full variant matrix** — colors, textures, and **sizes as purchasable
    formats** (60 MM / 6×13 / Grande), each carrying the installed pattern's
    tile `width_ft`×`depth_ft`×`thickness_in`, so a chosen color × format becomes
    a catalog item with real tile geometry. A format is a *system* laid as a
    pattern, so it stays ONE item (you tile the color swatch at the pattern
    repeat) and its included pieces (A/B/C) are recorded in `includes` as
    metadata, not split out. Every option carries **availability** (greyed =
    unavailable). The real-page lessons live on the schema: **prices are usually
    absent** (manufacturer→dealer) so never guess one; prefer **imperial** dims →
    feet. A Rust **guard pass** drops any non-positive price / absurd dimension
    the model slips through (softer JSON-Schema constraints aren't API-enforced).
    Default a cheap vision model (Haiku 4.5), editable; the user pays via their
    own key.
  - **B4 — per-color swatch images** ✅ — the extractor also returns a **bounding
    box** per color swatch (`Variant.bbox`, normalized fractions), and the app
    **crops each swatch out of the pasted screenshot** client-side (a `<canvas>`
    via `window.slpVision.crop`) into a per-color `data:` URI. The curation step
    shows the cropped thumbnails, and each approved color × format item carries
    its swatch as `image` — so the drawn area tiles with the real paver look in
    the viz. Best-effort (vision boxes are approximate; the catalog editor lets
    you replace an image).
    - *Modeling decision (2026-07-15): **multi-select**, not one-item.* A page is
      a configurator (color × texture × size), and the user wants to add several
      combinations to the catalog to **compare in the viz**. So extraction
      returns the whole matrix and curation (M4.2) lets the user tick multiple
      combos → **one `CatalogItem` per ticked combo** (distinct id/name/`image`
      swatch, shared base fields), each a placeable/tileable material.
- **M4.2 — human-in-the-loop curation (nothing ingested is live automatically)** ✅
  - [x] the extracted draft lands in a **staging** review (the `IngestDraft`
        component), separate from the plan's live catalog — extracting never
        silently changes a plan
  - [x] the user **multi-selects** which combos to keep (tick colors × sizes —
        available ones start ticked), edits the shared category / price, and
        **approves**: each ticked combo becomes one `CatalogItem` (a color's look
        at a size's geometry) appended to the catalog; **Discard** drops the
        draft. e2e-covered end to end (paste → extract → approve → catalog row).
  - [x] a **missing price** (the norm) is left blank for the user to fill (0 →
        no price set), **never guessed**; the shared price applies to every
        approved combo — the per-item price stays editable in the catalog editor.
- **B5 — adjustable swatch crop in curation** (refine B4's auto-crop) ✅
  - [x] in the "Add to catalog" dialogue, a color's swatch is a button — click
        it to open a `CropEditor`: the pasted screenshot with a **box overlay**
        at the current bounding box, plus **X/Y/W/H % inputs** to nudge/resize
        it; "Use crop" re-crops the region (via `vision::crop`) and updates that
        color's `swatch` + `bbox` live (reactive, so the thumbnail refreshes and
        the approved item carries the new swatch). Vision boxes are usually close
        but not exact, so this lets the user tighten the crop before adding.
        Component-driven: dokime (the overlay is positioned from the bbox, the
        inputs render) + a theoria story + e2e (open → adjust W → "Use crop" →
        the swatch re-crops to a new image). The catalog editor still lets you
        replace an image after the fact.
  - [x] **pointer-drag** of the box: drag the box to move it, its corner handle
        to resize, with the pointer captured (`setPointerCapture`) so the drag
        survives outrunning the box. The pixel→percent geometry is a pure
        function (`drag_box`) unit-tested natively (move/resize deltas, clamping
        to the image, the 2% minimum); the e2e drives the real held-button
        gesture via playwright's `drag_to` + `target_position` against a
        canvas-generated screenshot and asserts the box moved.
  - [x] the editor is a **modal**: a dimmed full-viewport backdrop (click to
        close; clicks inside stay put) centering the dialog, with the stage
        sized to the viewport (`min(86vw, 940px)` × `72vh`) instead of the
        catalog panel's narrow column — you can see what you're cropping.
  - [x] the crop bridge **trims near-white borders**: product pages wrap color
        chips in white cards and vision boxes include a sliver of that margin,
        which tiled as an area texture rendered as white grid lines between
        tiles. `slpVision.crop` now scans the cropped pixels and trims
        near-white/transparent border rows and columns (capped at 40% per axis
        so a light material survives). Approving a draft now also **replaces**
        catalog items with matching ids instead of skipping them, so
        re-extracting a product refreshes its swatches. E2e drives the *real*
        bridge (not the stub): white-margined, margin-free, and all-white crops.
- **M4.5 — catalog-driven area material picker** (the ingestion payoff) ✅
  - [x] the Area tool's picker is **driven by the plan's catalog**, not a
        hardcoded Mulch/Pavers pair. A new `MaterialPicker` component lists every
        *area material* — a catalog item priced per ft²/yd³/linear-ft and not a
        sub-base (`is_aggregate`) — **grouped by category** so the toolbar stays
        compact: one armable button per category (its selected type's swatch +
        a prettified name) plus a **type dropdown** when a category has more than
        one material (a dozen ingested paver colors → one "Slab" button + a
        dropdown, not a dozen buttons). Arm a category, draw an area, and it's
        **priced** (take-off already costs by `material_ref`) and **tiled** (the
        SVG `<pattern>` already tiles the material's image) with no new
        pricing/tiling code. Component-driven: dokime tests (grouping, the
        dropdown appears only for multi-material categories, the armed category
        is active) + a theoria story + e2e (the existing paver/mulch picks move
        to `area-mat-cat-<category>`; an ingested slab's category becomes
        armable). This answers "how do I select my paver for an area?".
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
  - [x] deleting a catalog item nothing references removes it outright; one
        that's in use is **blocked** with an "In use — N references" note —
        never a dangling ref (M4.3b). "In use" counts every reference kind:
        placed objects (`catalog_ref`), area materials (`material_ref`), area
        courses, and other items' base/bedding layers — `slp_core::
        reference_count`, unit- and mutation-tested. E2e: a hand-added
        material deletes (row + editor gone); the starter paver's base gravel
        is blocked with the note.
  - [x] **add** a new catalog item by hand (name/category/price/`price_unit`/
        dimensions) — the manual-authoring entry point, shipped with
        [B3.0](B3-area-composition.md)'s "+ Add" + `price_unit` `SelectField`
        in the catalog inspector (a hand-added material is catalog-only, not a
        placeable object)
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

# F2 — Place objects from a palette (placement UX)

*Enabler — a simpler, scalable way to place *any* catalog object. The
dropdown-plus-"Place furniture"-button flow was fine for one furniture category,
but it's two steps to arm and doesn't scale to fire pits, grills, trees, and
shrubs. This enabler serves every catalog-object story (E1, D1–D5): pick an item
from a **palette** and place it, with modifier keys for the common variations.
Reuses the [node-placement engine](H1-draw-house.md) unchanged — it only changes
how placement is *armed* and *previewed*.*

## Story

As a DIY homeowner laying out my yard, I want to click an object in a palette and
drop it on the plan — placing several in a row when I want, or a what-if ghost
when I'm trying an alternate spot — with a live outline of what I'm about to
place following my cursor, so that furnishing the plan feels direct instead of
clunky.

## Vertical slices

- **F2.0 — object palette (click-to-place)** ✅ *done*
  - [x] a **palette** replaces the catalog dropdown + the "Place furniture"
        button: catalog items as compact tiles **grouped by `category`**
        (Furniture, Fire pits, …), each showing the item's name, price, and a
        **mini-icon drawn from `style.rs`** (a small square or circle in the
        item's shape, so the tile matches the canvas)
  - [x] clicking a tile **arms** placement of that item (the tile highlights);
        clicking the armed tile again cancels without placing (matches how the
        House/Deck tool buttons already toggle; Esc-to-cancel arrives with the
        F2.1 modifiers, which add the keydown handler)
  - [x] rename `Tool::Furniture` → `Tool::Object` in `slp-core` (it places any
        category now), updating the placement engine + call sites
  - [x] dokime: the palette renders a tile per catalog item, grouped, with the
        armed tile flagged; e2e: arm a tile → click canvas → the object is
        placed and the estimate updates (folds in D2.0's deferred fire-pit e2e:
        place the fire pit from the palette → a round footprint renders + the
        inspector shows its diameter)
  - [x] *(incidental fix)* the canvas metrics now re-measure via a
        `ResizeObserver` on the yard, not just on window resize — the palette +
        estimate appearing reflow the canvas, which previously left the object
        inspector positioned against a stale top, floating over the toolbar
- **F2.1 — placement modifiers (Shift = sticky, Option = virtual)** ✅ *done*
  - [x] **Shift held** makes placement sticky: after a drop, the tool stays
        armed while Shift is down, so Shift-click… places a row; releasing Shift
        (a keyup) ends the run and disarms. A plain click places one and disarms.
  - [x] **Option/Alt-click** places the object as a **virtual** what-if ghost
        (dashed, per E1.6) instead of a real one; composes with Shift
        (Shift+Option = a row of ghosts). Reads `shiftKey`/`altKey` off the
        pointer event (threaded through `Yard`'s `on_commit` as a `Modifiers`
        struct) — one path, Option on macOS / Alt elsewhere.
  - [x] **Escape cancels** the armed tool without placing (the promised
        keyboard equivalent of clicking the armed tile again — lands here
        alongside the other keyboard handling)
  - [x] the hint line spells the modifiers out while armed ("Click to place the
        armed item (its tile again, or Esc, to cancel) · hold Shift to place
        several · ⌥/Alt to place a what-if ghost.")
  - [x] e2e: Shift-click places several without re-arming, releasing Shift ends
        the run, Option-click places a virtual object, Shift+Option places a
        row of ghosts, Escape cancels without placing
  - **Note on testing held modifiers:** `Locator::click`'s `.modifiers()` option
    is *transient* — Playwright presses the key, clicks, then explicitly
    restores (releases) it afterward, which fires a real `keyup` on the page.
    That's indistinguishable from a genuinely released key, so it correctly
    (if confusingly, the first time) triggers our own Shift-release listener
    after every single such click. Not a Playwright defect — it's documented
    behavior for a *single-action* modifier. For a key truly *held* across
    several actions, use `page.keyboard().down()/.up()` (real, persistent
    browser key state) with `page.mouse().click()` (which reflects ambient
    key state, unlike a `force: true` `Locator::click`, which doesn't).
- **F2.2 — placement preview ghost** ✅ *done*
  - [x] while an item is armed, a **faint (~50% transparent) outline of the
        object's footprint** — the actual shape (rect or circle), to scale and
        at the snapped position — follows the cursor, so you see *what* and
        *exactly where* it'll land, not just a center dot
  - [x] the ghost reuses the shape (a new shared `Footprint` — moved out of
        `Furnishings` so `Placement` resolves a catalog item's footprint the
        same way) and a fixed faint group opacity (`PREVIEW_OPACITY` in
        `style.rs`); falls back to the plain center-node marker when no
        catalog item is armed (house/deck/steps drawing is unchanged)
  - [x] dokime: an armed round item previews a translucent `<circle>`, a
        rectangular item a `<rect>`, both at the right scale; no shape preview
        without an armed item. e2e: hovering with nothing armed shows no shape
        preview; arming the (round) fire pit shows a circular preview that
        moves to a new `cx` when the pointer moves
  - **Deferred, not done:** reflecting the armed item's eventual status/virtual
    look live during hover (e.g. dashed while Option is held) — that needs
    modifiers threaded through `on_hover` too (currently only `on_commit`
    reads them), a bigger change for a preview-only nicety. The preview always
    shows the plain real/planned look regardless of hover-time modifier state.

## Notes / refs

- **No new placement machinery.** Arming still sets `selected_id` + the tool;
  the palette and modifiers just change *how* that happens. Placement, snapping,
  commit, and the object model are all reused from E1/F1/E1.6.
- **Palette vs. drag-and-drop.** Click-to-arm-then-place (not HTML→SVG drag) is
  simpler, touch-friendly, and fits the existing arm→click engine. Drag-drop is
  a possible later refinement, not this enabler.
- **Modifiers are per-click, not a mode** (except Shift's held-run): each click
  reads its own `shiftKey`/`altKey`, so there's no hidden sticky/ghost state to
  get stuck in. The one persistent-ish bit is the Shift-held run, which ends the
  instant Shift comes up.
- **Preview ghost is shape-aware by construction** — it shares
  `Furnishings`'s footprint rendering, so it can't drift from how the object
  actually draws (same discipline as the legend ↔ canvas via `style.rs`).

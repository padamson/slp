# F1 — Select / move / reshape / delete

*Enabler E — direct manipulation of what's already on the plan. The
[node-placement engine](H1-draw-house.md) draws things; F1 lets you **change
your mind** about them: pick a placed thing up, move it, or remove it. Builds on
the object **selection + inspector + rotate-handle** that landed with
[E1.3](E1-place-furniture.md) (`slp_core::pick::object_at`, the `selected`
index, the floating inspector, and the drag-gesture pattern).*

## Story

As a DIY homeowner laying out my yard, I want to grab a placed item and drag it
to a better spot, or delete it if I change my mind, so that I can iterate on the
layout without redrawing — the same way I'd shuffle real furniture around a deck.

## Vertical slices

- **F1.0 — move a placed object (pick-and-drag)** ✅ *done*
  - [x] pressing an object's body selects it and starts a drag; pointer-move
        repositions its center via `slp_core::snap::dragged_center` (adds the grab
        offset, snaps to the foot grid when "Snap to grid" is on); release ends
        the drag
  - [x] the fit check re-runs live — dragging a piece off its deck/paver surface
        turns its outline red as it leaves, back to normal when it lands
  - [x] a tool armed for drawing takes precedence — pressing on an object while
        placing still places, so drawing is never hijacked by a stray object
  - [x] e2e: drag a placed chair across the yard; its position (read from the
        inspector) updates to the drop point
- **F1.1 — delete the selected object** ✅ *done*
  - [x] the inspector gains a **Remove** button that deletes the selected object,
        clears the selection, and updates the estimate
  - [x] pressing **Delete**/**Backspace** with an object selected removes it
        (ignored while a text field / picker is focused, so it can't eat a keypress)
  - [x] e2e: place → select → Remove → the footprint is gone and the estimate
        line drops
- **F1.2 — select drawn outlines (house, deck levels)** ✅ *partly done*
  - [x] clicking a house or deck level's filled body selects it (mirrors how
        clicking a drawn area or a tree selects it — no `Selection` enum;
        each selectable thing is its own parallel signal, cleared by a shared
        `clear_selection` helper so picking one always deselects the rest)
  - [ ] step runs aren't yet selectable (no story has needed it yet)
  - [x] selecting the house or a deck level floats the shared `AreaInspector`
        in structure mode — its footprint (ft²), a build-status control
        (existing/planned), a deck level's elevation, and **Remove** ([B2](B2-area-cost.md))
  - [x] Remove deletes the corresponding geometry (a deck level, or the house
        outline + its openings); step-run delete is still unbuilt
- **F1.3 — move a drawn outline**
  - [ ] drag a whole deck level or step run (translate all its vertices),
        grid-snapped, with the same press-to-grab gesture
- **F1.4 — reshape (drag a vertex)** ✅ *partly done*
  - [x] drag a single house or deck-level corner (reuses the shape node-drag
        gesture); dependent geometry re-derives live where it's a lookup, not
        a stored reference — a house corner move just changes wall geometry,
        and each `Wall`/opening already reads its position from the corners +
        wall index every render, so doors/windows follow with no extra code.
        A deck level's `StepRun`s store baked coordinates (captured at draw
        time), so a level's corner move has no dependent geometry at all.
  - [ ] node **insert**/**delete** on house/deck outlines — built for freeform
        shapes ([F3.1](F3-draw-edit-shapes.md), reusable `slp-core::boundary`
        primitives), **not yet wired to House or Deck**: a deck level could
        reuse it as-is (no dependent geometry), but a house corner insert/
        delete would renumber wall indices and could silently reassign an
        opening to the wrong wall — needs its own design before it's safe

## Notes / refs

- **One gesture vocabulary.** Press-to-grab → move-to-drag → release-to-drop is
  shared with the rotate handle (`rotating`) and, later, outline move/reshape —
  each is an `RwSignal` gesture flag that short-circuits `on_hover`/`on_commit`.
  Adding a new manipulable thing means a new flag, not a new event pipeline.
- **Objects first, geometry later.** F1.0/F1.1 need no new selection model — the
  `selected` object index from E1.3 is enough. F1.2 introduces the `Selection`
  enum only when drawn outlines actually need selecting (demand-driven), since
  that's the real refactor; F1.3/F1.4 build on it.
- **No new take-off math.** Move/delete reuse the existing BOM: deleting an
  object drops its line, moving one doesn't change cost. Reshape (F1.4) is the
  first that changes an *area/volume*, but that math already lives in `slp-core`.
- Grid snapping reuses `snap_to_grid` (feet) via `dragged_center`, so a dragged
  object lands on the same foot grid as a freshly-drawn node — one source of
  truth for "on grid". `dragged_center` is unit + mutation tested in `slp-core`
  alongside the other snap helpers.

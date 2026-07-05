# D1 — Trees (round canopy + trunk)

*Epic D — placed catalog objects. Reuses everything E1/F1/F2 built (catalog,
palette, placement, inspector, estimate, status/virtual, move/delete). Adds
what's tree-specific: a **canopy over a trunk** (two concentric circles), a
**per-tree adjustable** canopy and trunk size, placement **any time** (a tree
goes in the yard, so it doesn't wait for a deck), and a **trunk-on-the-wrong-
surface** warning — a tree belongs on ground (yard or a mulch bed), not on the
house, a paver, or the deck.*

## Story

As a DIY homeowner, I want to place a tree — canopy and trunk round and to
scale, sized to this particular tree, and flagged if its trunk sits somewhere a
tree shouldn't — so that I can see how much space it takes, budget for it, and
put it somewhere sensible before I buy.

## Vertical slices

- **D1.0 — trees as round catalog objects** ✅ *done*
  - [x] a few starter trees seeded into the catalog (category `tree`; round
        shape, canopy diameter as `width_ft`, a representative price)
  - [x] trees get their own **palette group** ("Tree") for free — the palette
        already groups by `category` (F2.0)
  - [x] the rotation handle no longer shows on a selected **round** item —
        rotating a circle is a visual no-op (rectangular items are unaffected)
  - [x] dokime: a round item shows no rotate handle when selected; e2e: place a
        tree from the palette → a round footprint renders + the estimate updates

- **D1.1 — canopy + trunk render** ✅ *done*
  - [x] a tree draws as **two concentric circles**: a light, translucent-green
        **canopy** (diameter = `width_ft`) with a small, dark-brown **trunk** at
        its center — so it reads as a tree at a glance, not a generic disk
  - [x] the fill/stroke live in `slp-ui/src/style.rs` (canopy + trunk colors)
  - [x] status/virtual/selection still read through (a virtual tree is a dashed
        ghost, an existing one keeps its treatment) — the trunk/canopy split is
        a *category* look, layered under those independent channels
  - [x] dokime: a tree renders two circles (canopy + a smaller central trunk);
        e2e: the placed tree shows a trunk inside its canopy

- **D1.2 — adjustable canopy + trunk size** ✅ *done*
  - [x] a tree's **canopy diameter** and **trunk diameter** are per-tree
        adjustable (fork A: `Object` gains `canopy_diameter_ft` /
        `trunk_diameter_ft`, each falling back to the catalog default when
        unset) — so one oak can be a sapling and another mature
  - [x] the inspector shows two number inputs (canopy Ø, trunk Ø) for a selected
        tree; editing one updates that tree live and the render follows
  - [x] a selected tree also gets two **drag handles**, one on the canopy's
        edge and one on the trunk's — dragging either in/out resizes that
        circle directly on the canvas (the new diameter is twice the distance
        from the tree's center to the cursor, rounded to the nearest tenth of
        a foot, same "point toward the cursor" simplicity as the rotate
        handle); a second, on-canvas way to reach the same adjustment the
        inspector's number inputs give
  - [x] dokime: the inspector renders the two size inputs for a tree (not for a
        rectangular item); a selected tree renders both drag handles (a fire
        pit or furniture gets neither); e2e: bump a tree's canopy diameter via
        the number input, and via dragging each handle

- **D1.3 — place anywhere; trunk-on-the-wrong-surface warning** ✅ *done*
  - [x] trees are placeable **any time** (fork B: the starter catalog seeds on
        load, so the palette is available before a deck is drawn — a tree lives
        in the yard, not on the deck)
  - [x] a tree's **trunk** turns red when it overlaps a surface a tree shouldn't
        stand on — the **house** or the **deck** (a paver, once B1 lands); it's
        fine on the yard (bare ground) or a mulch bed, once B4 lands. The canopy
        may overhang any of those — only the trunk's position is checked
  - [x] the rule is the shared **category-aware placement-validity** model (see
        Notes) — plain, unit-tested `slp-ui` logic reusing `slp-core`'s already
        mutation-tested `point_in_polygon`, not new core geometry
  - [x] e2e: a tree trunk on the yard is quiet; dragged onto the deck its trunk
        goes red; back on bare ground it clears

## Notes / refs

- **Category-aware placement validity (shared enabler, also used by D2.2).**
  Each category's rule is a plain function in `slp-ui`
  (`furnishings.rs::category_ground_invalid`), driven by whether an object's
  center sits inside the house outline or a deck surface (via `slp_core::
  point_in_polygon`, already mutation-tested — no new core geometry needed):
  - *furniture* — must be **contained** in a deck or paver (today's fit-check,
    unchanged).
  - *tree* — its **trunk**'s center must **not** be on the house or the deck
    (OK on yard or a mulch bed).
  - *fire pit* — its center must **not** be on the house (OK on yard, a paver,
    or the deck).
  Only house + deck exist today; a paver/mulch-bed case slots into the same
  `on_house`/`on_deck` inputs once B1/B4 land — not faked here.
- **Reuses the round-object machinery** (rendering, hit-test, the circle bounding
  square) built for D2. A tree needed new render (canopy+trunk), the two schema
  fields in D1.2, and the placement rule — no new placement/estimate plumbing.
- **No clearance ring for trees.** `clearance_ft` stays absent on tree catalog
  items (that's a fire-pit safety concept). A tree's mature-size projection, if
  ever wanted, is a separate future story.

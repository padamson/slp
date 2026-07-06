# F3 — Draw & edit freeform shapes (boundaries, arcs, curves, circles)

*Enabler F — one shared way to draw and edit a **filled area** whose boundary
can mix straight edges, arcs, and curves, plus a standalone circle. Pulled by
[B4 mulch beds](B4-draw-mulch-beds.md) (the user's current design has curved
and circular beds), and reused unchanged by every later area story — pavers
([B1](B1-draw-paver-area.md)/[B2](B2-area-cost.md)) and mulch/gravel/bed areas
alike. The boundary is a `Shape`; area (for volume/cost take-off) is a headless
`slp-core` function over it, so the same geometry serves the 2D fill today and a
3D extrusion later (every shape carries height + material-ref, per PLAN.md §4).*

## Story

As a DIY homeowner, I want to draw an area with straight sides, arcs, or smooth
curves — or a plain circle — and then adjust it by moving, inserting, or
deleting the points along its edge, so that a bed or patio on my plan matches the
real curved shape I'm laying out, not just a boxy approximation.

## Vertical slices

- **F3.0 — draw a straight-edged boundary** ✅ *done*
  - [x] a shape tool drops boundary **nodes** on click, with a live preview edge
        to the cursor; clicking the first node closes the boundary (matches
        House/Deck's own shipped behavior — Enter-to-close isn't implemented
        for any outline tool yet, this doc's own aspiration included), Esc
        cancels an in-progress draft (already tool-agnostic)
  - [x] a closed boundary persists as a `Shape` in the plan and renders as a
        filled area; it survives a reload
  - [x] a `Shape` carries an **elevation** (feet above grade, default `0.0`),
        editable the same way a deck level's already is — same field every
        area-like entity carries ([H1.4](H1-draw-house.md) adds it to `House`
        too, not yet built)
  - [x] `slp-core` reports the boundary's **area** (ft²) — the value later
        stories cost against; `slp_core::geom::area` already existed
        (unit+mutation tested) — reused as-is, no new core geometry needed
  - [x] dropped nodes snap to the grid (reusing the same snap as every other
        drawing tool)
  - [x] dokime: `Shapes` renders a polygon + corner markers + an area (and,
        when non-zero, elevation) label, skips a degenerate (<3-corner) shape;
        `slp-core`: `Tool::Shape` behaves exactly like `Tool::House`/
        `Tool::Deck` (0 mutants missed on the changed `place.rs` logic); e2e:
        draw a 4-corner area, close it, reload, confirm it persists

- **F3.1 — edit the nodes of a boundary** ✅ *done*
  - [x] select a boundary node and drag to **move** it; the two edges meeting
        there follow, and any dependent readout (area) updates live — a shape
        is selected by clicking its filled body (no dedicated tool/button;
        mirrors how clicking a tree selects it and reveals its canopy/trunk
        handles), and a selected shape's corners become interactive node
        handles the same way
  - [x] select two adjacent nodes and **insert** a new node on the edge between
        them (it lands on that edge; the edge's shape is preserved) — press
        one node, then an *adjacent* one, to arm a floating Insert/Cancel
        popup near their midpoint; pressing a non-adjacent node (or a third,
        once a pair is armed) resets the selection to just that node instead
  - [x] select a node and **delete** it; its two neighbors reconnect directly
        — Delete/Backspace with exactly one node selected (reuses the
        existing object-delete keyboard handler's pattern)
  - [x] a boundary can't be reduced below a drawable minimum (deleting the last
        node that would leave under 3 is refused, not left degenerate)
  - [x] e2e: draw a boundary, move a node and watch the area change, insert a
        node, delete a node
  - [x] `slp-core`: `are_adjacent`/`insert_node_between`/`delete_node` in the
        new `boundary` module — entity-agnostic (operate on a plain
        `&[Coord]`), so `House`/`DeckLevel` node editing can reuse them
        unchanged when that lands; 0 mutants missed

- **F3.2 — arc edges** ✅ *done (move); node insert/delete-on-arc deferred*
  - [x] any boundary edge can be an **arc** (a circular bulge to one side)
        instead of a straight line, set/adjusted by a per-edge handle at the
        edge's apex; dragging it perpendicular to the chord sets the bulge
        (rounded to a tidy step, clamped to a major-arc range). The arc renders
        true-to-scale — an all-straight boundary stays a `<polygon>`, once any
        edge bows the whole boundary becomes a `<path>` with `A` arc commands.
        Bulge follows the CAD/DXF convention (`tan(θ/4)`, positive bows left of
        the edge's travel direction) in `slp_core::arc`
  - [x] the reported **area** accounts for each arc's bulge (`boundary_area` =
        signed shoelace + a signed circular-segment correction per arc, taken
        absolute) — a bed with a bowed-out edge reads more ft², bowed-in reads
        less; unit + mutation tested (0 missed on `arc.rs`, one equivalent
        `sweep` comparison excluded with rationale)
  - [x] moving an endpoint keeps the edge's bulge, so the arc's curvature stays
        sensible as its chord changes (the bulge is chord-relative)
  - [ ] **deferred:** inserting a node *on an arc* splitting it into two arcs
        (a half-angle split), and delete-merges-to-straight for arced
        neighbors. Node insert/delete (F3.1) changes the edge count, which
        would misalign the parallel `bulges` array; arc-aware re-indexing is
        deferred, so as a safe interim **editing the node ring on a bowed
        boundary resets its edges to straight** (never a misrendered arc).
        Straight-boundary insert/delete is unaffected. Tracked for a follow-up.

- **F3.3 — bezier (smooth-curve) edges** ✅ *done (curve + move); insert/delete-on-curve deferred*
  - [x] any boundary edge can be a **smooth curve** (cubic Bézier) with two
        draggable control handles, for an S-curve or free-form bed edge. A
        selected shape shows both controls per straight/curved edge (on a
        straight edge they sit at the chord's thirds; dragging one promotes the
        edge to a Bézier); a curved edge draws a guide line from each control
        to its anchor node and drops the arc apex handle. Renders true-to-scale
        as an SVG `C` path command; stored as a sparse `Shape.curves`
        (`CurveEdge{edge, control1, control2}`), curve winning over a bulge on
        the same edge
  - [x] the reported **area** accounts for curved edges — `boundary_area` adds
        each Bézier's signed segment area (`bezier_segment_area`, the analytic
        `½∮` of a cubic minus its chord); unit + mutation tested (0 missed on
        `bezier.rs` + `arc.rs`)
  - [x] moving an endpoint **carries its control handles** (same delta) so the
        curve's tangent doesn't kink
  - [ ] **deferred:** inserting a node *on a curve* splitting it into two
        curves (De Casteljau at t=½), and delete-merges-to-straight for curved
        neighbors. Node insert/delete changes the edge count, so — as for arcs
        — it currently **resets a curved boundary's edges to straight** rather
        than misalign the edge-indexed `curves`. Tracked for a follow-up.

- **F3.4 — standalone circle shape** ✅ *done*
  - [x] a circle tool draws a **circle** (center + radius): click the center,
        click again to set the radius — a new `Tool::Circle` gesture (Add
        then FinishWith, like a door/window span, but snapping freely like an
        object instead of snapping to a wall) — persists and renders filled;
        its radius is adjustable by a drag handle on its edge (reusing the
        round-object resize gesture from [D1](D1-trees.md): drag toward/away
        from center, rounded to the nearest tenth of a foot)
  - [x] a `Circle` shape carries **elevation** too, same as `Shape` (F3.0) —
        its own top-level `Plan.circles` list (not folded into `Shape`, since
        a circle has no corners/nodes to edit)
  - [x] `slp-core` reports the circle's **area** (`circle_area`, πr²) — so a
        circular bed or patio costs the same way any other area does; unit
        tested (and the `Tool::Circle` gesture is mutation-tested, 0 missed)
  - [x] the circle's own label reads its size as a **diameter** (⌀), matching
        how a round object's does — selecting it (click its body, like a
        shape/tree) is what reveals the resize handle; no dedicated inspector
        window was needed for this

- **F3.5 — a shape can be a step target**
  - [ ] once a `Shape` (boundary or circle) has an elevation (F3.0/F3.4), it's
        detectable as a step run's `to_elevation` the same way a deck level or
        the house already is ([H2.3](H2-draw-deck.md)) — steps from a deck (or
        the house) down to a paver area use *that area's* elevation instead of
        assuming grade
  - [ ] e2e: steps drawn from a deck toward a paver area land on the paver's
        elevation, not grade

## Notes / refs

- **Answers "can a bed/patio be circular?" — yes, two ways.** A true round bed
  is the **circle shape** (F3.4). A mostly-round-but-not-perfect bed (or a
  rounded-corner rectangle) is a **boundary with arc edges** (F3.2). Cost math
  doesn't care which — it asks the shape for its area, and `πr²` vs. an
  arc-corrected polygon area are both just "the area."
- **Per-edge kind, not per-shape.** A boundary is a ring of nodes; each *edge*
  (node→next) carries a kind — straight, arc, or curve — so one bed outline can
  freely mix all three. Straight is the default; F3.2/F3.3 add the other two as
  the same `Shape` gains an optional per-edge descriptor (the additive-schema
  pattern used for `clearance_ft`/`trunk_diameter_ft`).
- **Geometry decisions (implementation, for reference — behavior is pinned by
  the tests):** arcs use a signed **bulge factor** (the CAD/DXF convention:
  `bulge = tan(¼·included-angle)`, 0 = straight); curves are **cubic beziers**
  (two control points per edge, matching SVG and every vector tool). Area over
  a mixed boundary is the shoelace sum with a **circular-segment correction**
  per arc and an **analytic cubic-area term** per bezier. Inserting a node
  splits an arc by the half-angle and a bezier by **De Casteljau at t=½** —
  both reproduce the original edge exactly.
- **Deleting a node merges its two edges to a straight line**, deliberately:
  there's no unambiguous single arc/curve through the surviving neighbors, so a
  predictable straight merge beats a guessed curve. Documented, not accidental.
- **Node editing generalizes F1.4.** [F1](F1-select-move-delete.md)'s "reshape
  (drag a vertex)" slice is this, for freeform shapes — the same press-to-grab
  gesture vocabulary (an `RwSignal` gesture flag short-circuiting
  `on_hover`/`on_commit`) the object move/rotate/resize handles already use.
- **Curves are 3D-ready like everything else.** A `Shape` carries elevation +
  height + material-ref, so a curved bed extrudes to a curved 3D volume without
  a model change — same additive-renderer story as the rest of the plan.
- **No new take-off machinery beyond area.** F3 produces the area; volume/cost
  ($/ft², or depth→yd³ for mulch/gravel) is [B2](B2-area-cost.md)/
  [B4](B4-draw-mulch-beds.md), reusing `slp-core::takeoff` with the
  `price_unit` field M4 adds.
- **Elevation is now a shared "area-like entity" field** — `House` ([H1.4](H1-draw-house.md)),
  `DeckLevel` (already had it), and `Shape` (F3.0/F3.4) all carry it, which is
  what lets steps target any of them ([H2.3](H2-draw-deck.md), F3.5). **Doors
  and windows stay house-only** — nothing about openings generalizes to
  `Shape`; that's a House-specific concept, not an area-drawing one.

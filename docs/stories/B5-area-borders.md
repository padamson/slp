# B5 — Area edge borders (border courses + edging stones)

*Epic B — costed areas. A real paver install usually runs one or more **border
courses** around the field — a contrasting paver laid as a soldier/sailor ring,
a small cobble band, or a dedicated **edging stone** product (its own catalog
category, often priced per linear foot). The border changes both the look
(contrasting ring(s) hugging the boundary) and the buy list (border pieces are
bought separately from the field, and the field's ft² shrinks by the border
band). An area gets an ordered list of border rings, outermost first, each a
catalog material + a ring width.*

## Story

As a DIY homeowner, I want to edge a paver area with one or more border courses
— a contrasting paver or an edging stone — so the plan looks like the patio I'm
designing and the estimate prices the border pieces separately from the field.

## Vertical slices

- **B5.0 — schema: border rings on an area** ✅
  - [x] `Border` entity: `material_ref` (a catalog item) + `width_ft` (the
        ring's laid width; defaults from the material's tile/piece size,
        editable). `Shape.borders` + `Circle.borders`: ordered outermost-first.
        Regenerated via panschema. Full-perimeter rings only for now — per-edge
        borders (e.g. none against the house wall) are a deferred follow-up.
- **B5.1 — core: border take-off (`slp-core`, mutation-tested)** ✅
  - [x] boundary perimeter for every area kind (`boundary_perimeter`: straight
        chords, arc lengths from bulges, sampled Bézier lengths; circle 2πr)
  - [x] each ring's quantity: a per-ft² material costs `ring area =
        centerline perimeter × width_ft` (inner rings use the perimeter
        shrunk by `2π × cumulative offset` — the rounded-offset approximation,
        documented); a per-linear-ft material (edging stones) costs
        `centerline perimeter` linear ft
  - [x] the **field shrinks**: the surface material's ft² is the area minus
        the border rings' area (never below 0) — border and field are separate
        estimate lines, so the buy list matches how the pieces are ordered
- **B5.2 — ui: render the rings** ✅
  - [x] each ring paints as the boundary path stroked with the ring material's
        texture pattern (or flat color), clipped to the area: painter's
        algorithm with cumulative stroke widths (innermost ring drawn first,
        outermost last) so ring *i* shows as a band `width_ft` wide just inside
        the previous one. Works identically for polygons, arcs, curves, and
        circles because it reuses the boundary path. dokime: an area with two
        borders renders two clipped ring strokes in order; the field pattern
        still fills beneath.
- **B5.3 — ui: borders editor + starter edging + estimate lines** ✅
  - [x] the area inspector gains a **Borders** section (sibling of the course
        composition editor): an ordered row per ring — material select (any
        per-ft² / per-linear-ft catalog item, so pavers *and* edging stones
        qualify), width, remove — plus "+ Add border"
  - [x] starter catalog seeds an **edging stone** (category `edging`, priced
        per linear ft) so the feature works before any ingestion; ingested
        edging-stone products (the vision extractor already reads any product
        page) slot in with category `edging`
  - [x] the estimate shows each border as its own line under the area (like
        the gravel/sand sublines): ring material, quantity in its unit, cost;
        `reference_count` treats a border's `material_ref` as a reference
        (deletion stays blocked)
  - [x] e2e: draw a paver area → add a contrasting-paver border → a ring
        renders + a border estimate line appears and the field ft² shrinks →
        add a second (edging stone, per linear ft) ring → both lines price
- **B5.4 — per-edge border spans** ✅
  - [x] `Border.start_node`/`end_node`: both set = an **open span** covering
        the edges from `start_node` walking forward (drawn node order,
        wrapping) to `end_node` — border just one or two sides of a patio;
        absent = the full ring. Circles (no nodes) always ring.
  - [x] core: `boundary_span_length` (arc/curve-aware, wrap-capable, 0 for a
        degenerate span) + span-aware ring take-off — an open band keeps its
        base length (no `2π` corner shrink), and a **dead span** (a stale node
        index after node deletion) measures 0 rather than silently billing the
        whole perimeter
  - [x] ui: each border row gains **From/To node selects** ("—" = whole
        perimeter; hidden for circles); a selected shape labels its node
        handles with their **indices** so From/To has something visible to
        refer to; a span band renders as an open sub-path (`M …` without `Z`)
        clipped to the area
  - [x] e2e: the full ring's 34 lf edging becomes **18 lf** when scoped From
        n0 / To n2 (the 10 ft + 8 ft sides), and the field only loses that
        band (80 → 71 ft²)
  - [x] *(manual-testing fixes)* **per-edge offset stacking**: a band nests
        inward only under earlier bands covering the same edges (disjoint
        spans no longer inherit each other's depth), splitting into runs
        where the offset changes; **junction patches** (tangent-aware): an
        exact miter quad fills the reflex-corner pie gap where two bands meet
        (convex corners overlap naturally); the border row becomes a two-line
        grid so its selects stay readable
  - [x] *(manual-testing fix)* bands render as true **offset ribbons** (the
        filled region between the boundary run and its inward offset —
        straight edges exact, arcs exact by radius shift, Béziers sampled),
        not centered strokes: a centered stroke's outer half survives the
        area clip wherever "just outside this edge" is another part of the
        interior (a concave pocket's lip), painting the band on both sides
        of the edge. Ribbons are one-sided by construction; the clip remains
        only as a guard where a deep band's inner offset crosses a tight
        pocket. Cost was never affected — nothing outside the boundary is
        counted, though at curvature tighter than the band depth the linear
        band model conservatively overstates the band (and understates the
        field) slightly.
- **B5.5 — movable band seams (fractional boundary positions)**
  - [ ] `start_node`/`end_node` generalize to **boundary positions** (whole
        part = node index, fraction = fraction of that edge's length), so a
        band can start/end mid-edge; old integer plans parse unchanged
  - [ ] core: `boundary_span_length` over fractional endpoints (a partial
        edge costs `t × edge_len` — exact by definition for straight, arc,
        and Bézier edges), mutation-tested
  - [ ] render: `span_path` from/to mid-edge points (straight lerp, partial
        arc sweep, de Casteljau split); a seam moved off a corner is a clean
        butt joint, no patch needed
  - [ ] ui: **seam/end handles** on the selected area's bands — drag to slide
        a shared seam along the boundary (both bands update together) or to
        grow/shrink a free end; the From/To selects remain the precise
        fallback
  - [ ] e2e: drag a seam off the corner → both bands' lengths and the
        estimate reprice

## Notes / refs

- **PLAN §6 "later" row's `B5` (soldier-course border), generalized** from one
  soldier course to N ordered rings of any border-suitable material.
- **Rounded-offset perimeter.** Insetting a convex-ish boundary by `d` shortens
  its perimeter by `2πd` (corners round off). Ring *i*'s centerline offset is
  `Σ outer widths + widthᵢ/2`, so its perimeter is `P − 2π × offset`, clamped
  at 0. Good enough for a buy estimate; exact polygon offsetting is not worth
  its complexity here.
- **Edging stones are just catalog items** — category `edging`, usually
  per-linear-ft, ingested from screenshots like any other product. No special
  schema.
- **Deferred:** click-two-nodes gesture to define a span (the From/To selects
  carry the data model already); border piece-count rendering (individual
  stones drawn along the ring).

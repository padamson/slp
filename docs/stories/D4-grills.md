# D4 — Grills (rectangular footprint + shape-following clearance)

*Epic D — placed catalog objects. A grill is a **rectangular appliance** with a
**keep-clear zone** (like a fire pit's, but its clearance follows the grill's
own rectangular shape rather than a circle): the zone is the footprint grown
outward by `clearance_ft` on every side, with rounded corners — the Minkowski
sum of the rectangle with a disk. It turns red when another object or a
structure edge intrudes, exactly like the fire-pit ring. This generalizes the
today's circle-only clearance to rectangles, which needs a little real
`slp-core` distance geometry.*

## Story

As a DIY homeowner, I want to place a grill — a rectangular footprint to scale
with a keep-clear zone that hugs its actual shape and flags when something's too
close — so that I can budget for it and leave the safe space a grill needs
around it before I buy.

## Vertical slices

- **D4.0 — rectangular clearance geometry (`slp-core`)** ✅
  - [x] distance primitives, pure and mutation-tested:
        `dist_point_to_segment`, `dist_segment_to_segment` (0 when they cross),
        `dist_point_to_polygon` (0 inside), `dist_segment_to_polygon` (0 when
        the segment touches/enters the polygon) — the tools to ask "is X within
        `clearance` of this rectangle?"
- **D4.1 — grills: rectangular footprint + a shape-following clearance zone** ✅
  - [x] starter catalog seeds a couple of grills (category `grill`; rectangular
        `width_ft`×`depth_ft`, a price, a `clearance_ft`) — their own "Grill"
        palette group, seeded on load
  - [x] a grill renders its rectangular footprint (a distinct appliance look)
        and, from its `clearance_ft`, a **dashed rounded-rectangle keep-clear
        zone** that follows the footprint: the rect grown by `clearance_ft` on
        every side, corners rounded at radius `clearance_ft` — it rotates with
        the grill. (A fire pit's *circular* zone is unchanged — round footprint
        keeps the circle; only non-round items get the rounded-rect zone.)
  - [x] the zone turns the darker intrusion red when another object's footprint
        or a structure edge (house/deck) comes within `clearance_ft` of the
        grill's rectangle — the same signal a fire pit uses, now measured
        against the rectangle via the D4.0 primitives
  - [x] placing/costing/move/delete/select/status are the existing machinery;
        a grill places anywhere (no on-ground rule — it's fine on a patio, a
        deck, or the yard)
  - [x] dokime: a grill renders a rounded-rect clearance zone (an `rx`-rounded
        rect, not a `<circle>`); it's quiet when isolated and red when an object
        sits inside the zone. e2e: place a grill → rectangular footprint + a
        clearance rect; drop another object inside the zone → it flags red

## Notes / refs

- **Only the clearance *zone* generalizes; the intrusion *model* is the same.**
  A fire pit's round zone stays a circle (its footprint is round). A grill's
  rectangular footprint gets a rounded-rect zone. Both ask the same question —
  "is anything within `clearance_ft` of my footprint?" — just against a circle
  vs. a rectangle. The D4.0 primitives answer it for the rectangle.
- **No new schema.** A grill reuses `CatalogItem`'s `category` (`grill`),
  `width_ft`/`depth_ft`, `height_ft`, `unit_price`, and the existing
  `clearance_ft`. The clearance render/intrusion just stops assuming the
  footprint is round.
- **The rounded rectangle is the correct offset.** Growing a rectangle outward
  by a uniform distance `d` yields straight sides plus quarter-circle corners of
  radius `d` — an SVG `<rect rx=d ry=d>` at the expanded bounds is exactly that.

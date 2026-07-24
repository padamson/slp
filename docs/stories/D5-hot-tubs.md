# D5 — Hot tubs (surface-bound unit on a costed concrete pad)

*Epic D — placed catalog objects. A hot tub is a **heavy unit that belongs on a
prepared surface** (a deck or a paver patio, not bare ground), rendered water
blue. Unlike a grill or fire pit it carries no keep-clear zone; instead it sits
on a poured **concrete pad** — a slab the footprint grown outward by a
configurable **overhang** on every side, at a configurable **thickness** — which
is costed as concrete by volume, the same `yd³ = ft²·in/324` math a paver's base
course uses. This reuses three existing mechanisms — the shared containment
ground rule, per-object numeric overrides (like a tree's trunk), and a catalog
item's sub-material course (like a paver's gravel base) — so the only genuinely
new piece is a rectangular slab volume folded into the take-off.*

## Story

As a DIY homeowner, I want to place a hot tub — a to-scale unit that flags when
it isn't fully on a deck or patio, sitting on a concrete pad whose thickness and
edge overhang I can set — so that I can budget for both the tub **and** the
concrete it needs, and confirm it lands on a surface that can carry it before I
buy.

## Vertical slices

- **D5.0 — hot tubs: a surface-bound unit, water blue** ✅ *(shipped in
  `2762c4b`, alongside grills)*
  - [x] starter catalog seeds a round and a square hot tub (category `hot-tub`;
        a price, a footprint), in their own palette group, seeded on load
  - [x] a hot tub renders a **water-blue** footprint (round or square) and, via
        the shared containment fit-check, flags red when it isn't fully on a
        deck or paver surface — a heavy unit belongs on prepared ground
  - [x] placing/costing/move/delete/select/status are the existing machinery
  - [x] dokime: a hot tub fills water blue, flags off a surface, quiet on one.
        e2e: place a round hot tub off the deck → water blue + flags

- **D5.1 — concrete pad: schema + take-off (`slp-core`)** ✅
  - [x] schema: a catalog item gains `slab_material_ref` (the concrete material
        its pad is poured from), `slab_thickness_in`, and `slab_overhang_in`
        (the pad's default thickness and per-side edge overhang); an object
        gains `slab_thickness_in`/`slab_overhang_in` **overrides** for this
        particular tub — mirroring how `base_material_ref`/`base_depth_in` and
        a tree's `trunk_diameter_ft` already work
  - [x] take-off: each placed (planned, non-virtual) hot tub adds a **concrete
        pad** volume to its `slab_material_ref`'s line — a rectangle
        `(width + 2·overhang) × (depth + 2·overhang)` at `slab_thickness_in`,
        by `yd³ = ft²·in/324`. Thickness/overhang resolve object-override →
        catalog default; each side floored at 0 (a wide overhang can't flip the
        area positive). So the concrete material lists one costed line summing
        every tub's pad. Pure, unit- and mutation-tested.

- **D5.2 — render the pad + set it in the inspector** ✅
  - [x] a hot tub draws its **concrete pad** — a gray rectangle behind the tub
        body, the footprint grown by the overhang on every side — so the pad's
        real extent reads on the plan
  - [x] the object inspector shows **Slab thickness** and **Slab overhang**
        number fields for a hot tub (like a tree's trunk/canopy), editing the
        per-object override; the starter concrete material + the two tub seeds
        carry sensible defaults (a 4-in pad, a 12-in overhang)
  - [x] dokime: a hot tub renders a gray pad larger than its footprint, a
        per-object overhang override widens it, a non-slab unit pours none; the
        inspector shows the two slab fields for a tub and not for other
        categories. e2e: place a hot tub → a gray pad + a **Concrete** line in
        the estimate; widen the overhang → the pad grows

## Notes / refs

- **The slab is a course, not a drawn area.** It reuses the paver base/bedding
  mechanism (B2.2): a catalog item names a sub-material and a depth, and the
  take-off adds that sub-material's volume. The only difference is the area
  comes from the object's grown footprint rather than a drawn `Shape`/`Circle`.
- **A rectangular pad under any tub.** Even a round tub sits on a rectangular
  pour, so the slab is always `(w+2·oh) × (d+2·oh)` — a square pad under a round
  tub. No polygon offset needed.
- **Overhang is the lip of slab visible around the tub** — the pad extends
  `slab_overhang_in` beyond the footprint on every side. Stored in inches for
  consistency with `slab_thickness_in` and the `…_in` depth convention.
- **No keep-clear zone.** A hot tub has no `clearance_ft`; its feedback is the
  containment fit-check (must sit on a surface), not an intrusion ring.

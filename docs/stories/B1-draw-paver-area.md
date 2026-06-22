# B1 — Draw a paver area by clicking corners

*Epic B — Hardscape areas · yard layer.*

## Story

As a DIY homeowner, I want to draw a paver patio by clicking corner points on the
yard and closing the shape, so that I can lay out exactly where pavers go.

## Vertical slices

- **B1.0 — draft & finish a polygon**
  - [ ] selecting the paver tool lets me click to drop corner points
  - [ ] a live preview follows the cursor while drafting
  - [ ] clicking the first point (or pressing Enter) closes the polygon
  - [ ] the finished area persists as a `Polygon` shape and renders filled
- **B1.1 — snapping**
  - [ ] dropped points snap to the half-foot grid
- **B1.2 — cancel / keyboard**
  - [ ] Esc cancels an in-progress draft; Enter finishes it

## Notes / refs

- Pointer→feet is the inverse of `slp-ui::Transform` (spike `ftpt`).
- Depends on A1.1 (reactive state) — extend it to a plan/shape list.
- Editing finished shapes (move/reshape/delete) is **story F1**, not here.

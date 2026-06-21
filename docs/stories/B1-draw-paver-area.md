# B1 — Draw a paver area by clicking corners

**Epic:** B — Hardscape areas · **Layer:** yard · **Status:** planned (Milestone 1)

## Story

As a DIY homeowner, I want to draw a paver patio by clicking corner points on the
yard and closing the shape, so that I can lay out exactly where pavers go.

## Acceptance criteria

- Selecting the "+ Paver Area" tool lets me click to drop corner points.
- A live preview polyline follows my clicks.
- Clicking the first point (or pressing Enter) closes the polygon; Esc cancels.
- The finished area is stored as a `Polygon` shape in the `Plan` (with default
  `material_ref`, `elevation`, `height`).
- The area renders filled on the canvas.

## Vertical slices

- **B1.0 — draft & finish a polygon** — tool mode + click-to-add points + live
  preview + close on first-point/Enter. Persists a `Polygon` to the `Plan`.
  *Tests:* `slp-core` test that a closed ring is well-formed; dokime test that a
  Plan with one polygon renders one filled `<polygon>`; e2e draws a triangle and
  asserts it appears.
- **B1.1 — snapping** — snap points to the half-foot grid (spike `snap`).
  *Tests:* `slp-core` unit test on `snap`.
- **B1.2 — cancel / keyboard** — Esc cancels a draft; Enter finishes. *Tests:*
  e2e starts a draft, presses Esc, asserts nothing persisted.

## Notes / refs

- Pointer→feet transform is the inverse of `slp-ui::Transform` (spike `ftpt`).
- Depends on A1.1 (Plan state) landing first.
- Editing the finished shape (move/reshape/delete) is **story F1**, not here —
  keep B1 to *drawing*.

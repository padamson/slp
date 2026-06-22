# A1 — Yard at the dimensions I set

*Epic A — Canvas · cross-cutting.*

## Story

As a DIY homeowner, I want to set my yard's width and depth and see it drawn to
scale, so that I'm planning against my actual space.

## Vertical slices

- **A1.0 — static yard**
  - [x] the yard renders to scale with a foot grid and a scale bar
  - [x] the WASM app boots and the SVG canvas mounts
- **A1.1 — editable dimensions**
  - [x] width and depth number inputs, clamped to a 1 ft minimum
  - [x] editing either dimension re-renders the canvas to the new scale
  - [x] the yard size persists across reload (`Plan` saved to `localStorage`)

## Notes / refs

- The yard is a rectangle (W×D) for now; an irregular yard boundary is a future
  story.
- The **house** and **deck** are *drawn* by the user, not baked in — see the
  Structures stories (H1, H2). Nothing is tied to a specific property.
- `Plan` (yard size) is panschema-generated; it grows with structures, shapes,
  and objects as later slices need them.

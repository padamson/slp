# A1 — Yard at dimensions I set, with house + deck reference

*Epic A — Canvas · cross-cutting.*

## Story

As a DIY homeowner, I want to see my backyard drawn to scale at dimensions I set,
with the house wall and the existing deck shown for reference, so that I'm
planning against the real space.

## Vertical slices

Each slice is a thin, shippable increment; its acceptance criteria are the
checkboxes below. Behavior is specified by the tests in code, not restated here.

- **A1.0 — static yard**
  - [x] the yard renders to scale with a foot grid and a scale bar
  - [x] the WASM app boots and the SVG canvas mounts
- **A1.1 — editable dimensions**
  - [x] width and depth number inputs, clamped to a 1 ft minimum
  - [x] editing either dimension re-renders the canvas to the new scale
- **A1.2 — house reference**
  - [ ] the back wall + bump-out are drawn along the bottom
- **A1.3 — deck reference**
  - [ ] the existing deck (polygon + stairs + railing) is drawn for reference
  - [ ] the deck can be slid left/right to match its real position

## Notes / refs

- Geometry constants (`HW`, `DECK`, stairs/railing) come from the gitignored
  spike — recreate as typed constants in `slp-ui`/`slp-core`, don't import it.
- State is plain Leptos signals; the LinkML `Plan` schema waits until
  shapes/persistence need it.

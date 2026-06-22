# T1 — Browse components in a sidebar; preview the selected one on a stage

*Pulled by: slp needing to eyeball `slp-ui` components in isolation.*

## Story

As a Leptos developer, I want to list my components in a sidebar and preview the
selected one on a stage, so that I can develop a component without the
surrounding app.

## Vertical slices

- **T1.0 — list + first preview**
  - [x] a `Story` is `(name, view closure)`
  - [x] `Gallery` lists one nav entry per story and shows the first by default
- **T1.1 — switch on click**
  - [x] clicking a name selects it; the active entry is marked
- **T1.2 — two-pane styling, mountable in a host**
  - [x] sidebar | stage layout; mounts in a host app
- **T1.3 — browser e2e**
  - [x] a playwright test drives the gallery and asserts switching (`theoria-e2e`)
- **T1.4 — selection persists across reload**
  - [x] the selected story is restored after a full page reload (localStorage),
    so Trunk's hot reload keeps you on the story you were viewing

## Notes / refs

- `StoryNav` is a leaf component — easy to unit-test and to reuse as an e2e
  fixture. Never a `Gallery` inside a `Gallery` (infinite mirror).

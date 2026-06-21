# T1 — Browse components in a sidebar; preview the selected one on a stage

**Status:** ✅ done · **Pulled by:** slp needing to eyeball `slp-ui` components
in isolation while developing them.

## Story

As a Leptos developer, I want to list my components in a sidebar and preview the
selected one on a stage, so that I can develop and eyeball a component without
the surrounding app.

## Acceptance criteria

- A `Story` is a `(name, view-producing closure)`.
- `Gallery` renders one nav entry per story and shows the selected story's view.
- The first story is shown by default; clicking another selects it; the active
  entry is visually marked.

## Vertical slices

- **1a — list + first preview** ✅ — `Story::new` + `Gallery` renders the names
  via `StoryNav` and shows the first story. *Tested:* dokime
  (`gallery.tests.rs`, `story_nav.tests.rs`).
- **1b — switch on click + active mark** ✅ — `on:click` selects; `class:active`
  highlights. *Tested:* dokime asserts exactly one `active` entry.
- **1c — two-pane styling, mountable in a host** ✅ — `.theoria` / `.theoria-nav`
  / `.theoria-stage` layout; mounts in a host app.
- **1d — browser e2e** — drive the gallery in a real browser, click a story,
  assert the stage swaps. *Deferred* to when `theoria-demo` + `theoria-e2e` are
  wired (story T4).

## Notes / refs

- `StoryNav` is a leaf component (no children) so it is easy to unit-test and to
  use as an e2e fixture later (T4) — never a `Gallery` inside a `Gallery`.
- Components live in `src/components/` (`gallery.rs`, `story_nav.rs` +
  `.stories.rs` / `.tests.rs`).

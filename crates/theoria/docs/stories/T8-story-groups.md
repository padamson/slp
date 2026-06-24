# T8 — Story groups / nesting in the sidebar

*Sidebar organization — pulled forward when slp-ui grew a `Door → Window → Wall
→ House` ladder (~11 stories) and a flat list stopped scaling.*

## Story

As a Leptos developer, I want to organize stories into a hierarchy, so that a
growing component set reads as a tree instead of a long flat list.

## Vertical slices

- **T8.0 — hierarchical sidebar from `/`-delimited names** ✅
  - [x] a story name is a path: `"Structures/House/Outline"` → groups
        `Structures` › `House`, leaf `Outline`
  - [x] equal group labels at the same level merge; insertion order preserved
  - [x] flat names (no `/`) still render as top-level leaves (backward-compatible)
  - [x] leaves are the clickable/selectable buttons; groups are labels; nesting
        is shown by indentation

## Notes / refs

- **Manual, not auto-derived from composition.** Nesting comes from the *name
  path* the author writes, not from the component render tree. A story is a
  runtime view (`Fn() -> AnyView`), and the same component appears in many
  stories — so sidebar hierarchy is an authoring concern, decoupled from render
  composition (the Storybook convention). The composition itself is still shown
  *in the stage* (e.g. the "Structures/House/Doors & windows" story renders
  walls → doors → windows).
- Implementation: `build_tree(names) -> Vec<NavNode>` (pure, unit-tested) +
  recursive `render_nodes` in `StoryNav`. Selection is still by the leaf's index
  in the original list, so localStorage persistence (by name) is unchanged.
- **Follow-ups (not now):** collapsible groups (per-group open/closed state);
  group-level styling beyond indentation. Pull when the tree gets deep enough to
  want collapsing.

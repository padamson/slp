# G1 — Save / load the plan file

*Enabler G — the plan already round-trips through `localStorage` (`slp-ui::
planner`'s `save_plan`/`load_plan`) as `.slp.json`-shaped `serde_json`, but
that's one browser's storage, not a file the user owns. G1 makes the plan an
actual file: exportable, shareable, and loadable in a fresh browser/profile/
machine — the same JSON, just handed to the user instead of hidden in
`localStorage`.*

## Story

As a DIY homeowner, I want to save my plan (yard, house, deck, catalog, and
everything I've placed) to a file, and load it back later — on this machine or
a different one — so that my work isn't trapped in one browser's storage and I
can back it up, share it, or start a fresh what-if copy.

## Vertical slices

- **G1.0 — export the plan to a file**
  - [ ] a "Save to file" control serializes the current `Plan` (the same
        `serde_json` shape `localStorage` already round-trips) and downloads it
        as `<plan-name-or-default>.slp.json`
  - [ ] `localStorage` autosave (today's behavior) is unchanged — G1 adds an
        explicit file export alongside it, not a replacement
- **G1.1 — import a plan from a file**
  - [ ] a "Load from file" control reads a user-picked `.slp.json`, validates
        it (panschema-generated `serde` types reject malformed shapes rather
        than silently half-loading), and replaces the current plan
  - [ ] a malformed/incompatible file shows an error and leaves the current
        plan untouched — never a half-applied state
  - [ ] e2e: export the current plan, reload the app (clearing in-memory
        state), import the exported file, and confirm the yard/house/deck/
        catalog/objects all match what was saved

## Notes / refs

- **No new schema.** The file *is* `Plan` — the same LinkML/panschema-generated
  type `localStorage` already serializes. G1 is a UI affordance (download/file-
  picker) over existing serialization, not a format change.
- **The custom catalog travels with the file for free.** Since `Plan.catalog`
  is already part of `Plan`, a user's ingested/edited catalog (see
  [M4–M5](M4-M5-material-ingestion.md)) saves and loads with everything else —
  no separate "catalog file" format needed unless a later story wants to share
  *just* a catalog between plans.
- **Versioning is out of scope for G1.0/G1.1.** The schema carries a `version`
  (`schema/slp.yaml`'s `version: 0.1.0`) but there's no migration story yet for
  loading an older-shaped file after the schema changes — a future slice if/when
  a breaking schema change actually needs one.
- `#[cfg(feature = "csr")]`-gated, like `load_plan`/`save_plan` today — file
  I/O (a download, a `<input type="file">` picker) only exists in the browser
  build, not under `ssr`/native tests.

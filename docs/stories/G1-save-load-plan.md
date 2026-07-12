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

- **G1.0 — export the plan to a file** ✅
  - [x] a **Save** control serializes the current `Plan` (the same `serde_json`
        shape `localStorage` already round-trips) and downloads it as
        `<plan-name-or-default>.slp.json` (`slp_core::plan_filename` slugifies
        the plan name; mutation-tested)
  - [x] `localStorage` autosave (today's behavior) is unchanged — G1 adds an
        explicit file export alongside it. The plan-assembly is now one
        `current_plan()` closure shared by the autosave and the export.
- **G1.1 — import a plan from a file** ✅
  - [x] an **Open** control reads a user-picked `.slp.json` via a hidden
        `<input type="file">`, validates it (`serde` rejects malformed/wrong
        shapes — `plan_file::parse_plan`), and replaces the whole plan
        (`apply_plan` fans it back out to every signal + resets selection)
  - [x] a malformed/incompatible file shows an inline error and leaves the
        current plan untouched — never a half-applied state
  - [x] e2e: Save the plan to a download, clear `localStorage` + reload
        (fresh default plan), Open the saved file, and confirm the yard width
        and placed object come back; a second e2e covers the malformed-file
        error path
- **G1.2 — a named current file: Save vs. Save As (multiple files)** ✅
  - [x] where the **File System Access API** is available (Chromium), **Save
        As** opens the native save dialog so the user names/places the
        `.slp.json`; SLP remembers that file handle as the *current file* (name
        shown in the toolbar) and writes to it. Calling Save As again makes a
        *different* file — the user can keep as many plans as they like.
  - [x] **Save** writes back to the current file with no dialog once one is
        chosen; with no current file it behaves as Save As.
  - [x] where the API is absent (Firefox/Safari), Save/Save As gracefully fall
        back to the G1.0 download and Open to the `<input type="file">` picker.
        (A `window.slpFs` bridge in `index.html` encapsulates the FSA +
        IndexedDB glue; `slp-ui::fs_access` is the thin async Rust bridge that
        degrades to the fallback when `slpFs` is absent.)
- **G1.3 — reopen the last file on startup** ✅
  - [x] the current file's handle is persisted (IndexedDB) so a return visit can
        reopen it. On startup, if the browser still holds read permission for
        that handle, SLP loads it **silently** (the closest a sandboxed web app
        can get to "open my file automatically").
  - [x] where the browser requires a fresh gesture (permission lapsed, or a
        different origin/profile), SLP shows a one-click **"Reopen &lt;name&gt;"**
        affordance instead of loading silently — a browser-security boundary, not
        a bug. `localStorage` autosave still restores the working plan with no
        gesture, as today.
  - [x] e2e (`plan_file_fsa.rs`): an `add_init_script` fake
        `showSaveFilePicker`/`showOpenFilePicker` + fake IndexedDB drives the
        real Save As / in-place Save / Open, silent startup reopen, and the
        one-click Reopen-after-permission-lapse — the pattern a future
        first-class playwright-rs FSA helper would replace (cross-repo note
        `playwright-rust--fsa-test-helper`).

## Notes / refs

- **Browser sandbox: no silent disk access without a grant.** A CSR/WASM app
  cannot scan the user's filesystem for a "default save location" and read it
  on load — the platform forbids it. The **File System Access API** is the only
  path to a real, named, re-openable file, and even it needs a user gesture to
  *first* grant a handle; startup reopen is silent only while that grant
  persists (Chromium remembers it per-origin for a while), else it's one click.
  This is why G1.3 is "reopen the last file (silent when permitted, one-click
  otherwise)" rather than "auto-load a file off disk."
- **Testability.** The download + `<input type="file">` fallback (G1.0/G1.1) is
  fully Playwright-drivable and is what the round-trip e2e exercises. The File
  System Access API path (G1.2/G1.3) uses native pickers and OS-file writes that
  Playwright can't drive, so it's covered by feature-detected code + dokime
  (control presence), not e2e — noted here so the coverage gap is explicit.

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

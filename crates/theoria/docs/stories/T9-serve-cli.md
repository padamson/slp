# T9 — `theoria serve`: generate and serve the gallery

**Status:** ✅ done · **Pulled by:** slp needing to view `slp-ui` components in a
gallery *without* hand-writing or committing a gallery crate (the question
"won't theoria generate and serve the gallery?").

## Story

As a Leptos developer, I want a CLI that discovers my crate's stories and
**generates and serves** a component gallery, so I never hand-write or commit a
gallery app — the Storybook `dev` experience for theoria.

## Acceptance criteria

- A `theoria.toml` declares the crate, its stories aggregator fn, and features.
- `theoria serve` generates a Trunk app under `target/theoria/gallery` and serves
  it with hot reload; `theoria build` builds it; `theoria generate` only writes
  the harness.
- The generated harness is gitignored (under `target/`) — nothing to commit.

## Vertical slices

- **9.0 — generate harness** ✅ — from `theoria.toml`, write `Cargo.toml` +
  `index.html` + `src/main.rs` (mounts `theoria::Gallery` with the aggregator),
  with its own `[workspace]` and absolute path deps.
- **9.1 — build/serve via Trunk** ✅ — `theoria build` / `theoria serve`.
- **9.2 — browser e2e of the served gallery + `#[story]` auto-discovery** —
  deferred (story T4 / T5), pull when needed.

## Notes / refs

- Lives in `crates/theoria-cli` (bin name `theoria`); native orchestration, no
  Leptos dep. Travels with the theoria family on spin-out.
- In-repo it uses path deps (`theoria_path` default `crates/theoria`). A
  published CLI would instead depend on a crates.io `theoria` version — future
  work when theoria is spun out.

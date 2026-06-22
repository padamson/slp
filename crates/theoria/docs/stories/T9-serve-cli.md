# T9 — `theoria serve`: generate and serve the gallery

*Pulled by: viewing components without hand-writing or committing a gallery crate.*

## Story

As a Leptos developer, I want a CLI that discovers a crate's stories and
**generates and serves** a gallery, so that I never hand-write or commit a
gallery app — the Storybook `dev` experience for theoria.

## Vertical slices

- **T9.0 — generate harness**
  - [x] from `theoria.toml`, write `Cargo.toml` + `index.html` + `src/main.rs`
  - [x] the generated crate is gitignored (under `target/`); nothing to commit
- **T9.1 — build / serve via Trunk**
  - [x] `theoria serve` serves with hot reload; `theoria build` builds
- **T9.2 — watch the source crate (true hot reload)**
  - [x] the generated `Trunk.toml` watches the source crates' `src/`, so editing
    a component or its `*.stories.rs` rebuilds and refreshes the browser

## Notes / refs

- Lives in `crates/theoria-cli` (bin `theoria`); native orchestration, no Leptos.
- In-repo it uses path deps; a published CLI would depend on a crates.io
  `theoria` version (future, on spin-out).

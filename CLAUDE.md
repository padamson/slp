# SLP — Simple Landscape Planner

A Leptos client-side (CSR/WASM) app to plan a backyard landscape — draw your
yard, house (walls + doors/windows), and deck, then add pavers, beds, walls,
steps, trees, equipment, and deck furniture; visualize it to scale and budget it.
A **general-purpose tool for any property** (nothing is hardcoded to a specific
house/yard/deck), built to help a DIYer **decide what to buy**. **Read
`docs/PLAN.md` for the full plan, decisions, backlog, and vertical-slice order.**

## Repo structure

```
crates/slp-core   # headless geometry + take-off math (member, native)
crates/slp-ui     # Leptos components, runtime-agnostic (member)
                  #   src/components/<name>.{rs, stories.rs, tests.rs}
crates/slp-app    # Leptos CSR/WASM app (excluded; built by Trunk; selects csr)
crates/slp-e2e    # playwright-rust dogfood tests (excluded; standalone)
schema/           # LinkML schema -> panschema-generated serde types (Milestone 1+)
materials/        # manifest.toml (committed) + cache/ (GITIGNORED — never commit assets)
docs/PLAN.md      # overview;  docs/stories/ = one detailed doc per story (sliced)
docs/notebooks/   # lab notebooks: WHY a design choice was made, with evidence
```

The wasm app (`slp-app`) and native-test crate (`slp-e2e`) are **excluded from
the workspace** so `cargo --workspace` stays native and fast, and no member
enables Leptos `csr` (which would clash with leptotes's `ssr`) — mirrors the
playwright-rust site / site-e2e split.

## Development commands

```bash
# All native crates: slp-core + slp-ui (fast inner loop)
cargo test --workspace
cargo nextest run --workspace

# A single crate's tests
cargo test -p slp-core      # geometry / take-off math
cargo test -p slp-ui        # slp-ui components (rendered via leptotes)

# Both hot-reload dev servers, the planner (8080) and the theoria gallery
# (8081), each auto-picking the next free port if its default is taken:
./scripts/dev.sh                        # or ./scripts/dev.sh 9000 to start higher

# The gallery alone, or a one-off build of it (from the repo root: reads ./theoria.toml)
theoria serve
theoria build

# Manual local testing of the PLANNER (hot reload at http://localhost:8080)
cd crates/slp-app && trunk serve
cd crates/slp-app && trunk build        # one-off build to dist/

# E2E (drives the real WASM in a browser, after building the app)
cd crates/slp-app && trunk build
cargo test --manifest-path crates/slp-e2e/Cargo.toml

# Lint / format (clippy pedantic is project policy)
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
cargo fmt --manifest-path crates/slp-app/Cargo.toml   # excluded crates: `--all` skips them
cargo fmt --manifest-path crates/slp-e2e/Cargo.toml
```

## Dogfood dependencies

leptotes (component testing for Leptos) and theoria (a component explorer for
Leptos) started here as workspace crates and now live in their own repos,
[padamson/leptotes](https://github.com/padamson/leptotes) and
[padamson/theoria](https://github.com/padamson/theoria). `slp-ui` takes both
from git `main`: leptotes as a dev-dependency, theoria behind the `stories`
feature. A change either one needs is made in its repo, and Dependabot's weekly
`git-dependencies` group brings it here.

The gallery app that `theoria serve` generates mounts through theoria's
csr-only `Stories::mount`, so `slp-ui`'s `csr` feature forwards `theoria?/csr`
and `all_stories` returns `theoria::Stories`. Miss either and the gallery build
fails with ``no method named `mount` ``.

When theoria's `main` changes what a stories crate provides, move three things
together: the CLI (reinstall it, below), the root lockfile (`cargo update -p
theoria -p theoria-macros`, which the workspace's own builds of `stories` use),
and the gallery harness's lockfile, which the harness keeps in its own
workspace under `target/theoria/` (`rm -rf target/theoria` makes the next
`theoria serve` resolve afresh).

### One-time setup

- `rustup target add wasm32-unknown-unknown`
- `cargo install trunk`
- Gallery CLI: `cargo install --locked --git https://github.com/padamson/theoria theoria-cli`
- E2E browser: `cargo run --manifest-path crates/slp-e2e/Cargo.toml --example install-browsers`
  (installs the chromium revision the locked playwright-rs expects)
- Pre-commit hooks: `cargo install prek && prek install`

## How to test as you develop

1. **Component / unit logic:** put math in `slp-core` and unit-test it natively
   (`cargo test -p slp-core`) — instant feedback, no browser.
2. **Manual / visual:** `trunk serve` and look at `http://localhost:8080`; hot
   reload picks up edits. **theoria** (component explorer) is for working on a
   single component in isolation; `./scripts/dev.sh` serves it next to the
   planner.
3. **Component tests:** **leptotes** for Leptos components.
4. **End-to-end gate:** the `slp-e2e` playwright-rust test drives the built app
   in chromium — it's the "does the WASM actually boot and react" gate.

## Conventions

- **Component-driven UI:** small single-purpose `#[component]`s; never one
  monolith. See `docs/PLAN.md` §3.1.
- **2.5D data model:** every shape carries footprint + height + material-ref, so
  the 2D plan extrudes to a future 3D view without a rewrite.
- **Ingested material assets are never committed** — only metadata + provenance
  in `materials/manifest.toml`. Binaries live in gitignored `materials/cache/`.
  Respect each source's robots.txt/ToS; never redistribute scraped images.
- **Clean-sheet `.slp.json`** — the plan-file format is defined by the LinkML
  schema (`schema/slp.yaml`), generated to Rust by panschema.
- Edition 2024, MSRV 1.95, `unsafe_code = "forbid"`, clippy `pedantic`.

## Delivery

Delivery milestones (see `docs/PLAN.md` §6). **Milestone 0 (walking skeleton) is done:**
workspace + Leptos app renders the yard to scale + first playwright-rust E2E
green. **Milestone 1 = "draw a paver area & see cost."**

# SLP — Simple Landscape Planner

A Leptos client-side (CSR/WASM) app to plan a backyard landscape (pavers, beds,
walls, steps, trees, equipment, deck furniture), visualize it to scale, and
budget it — built to help a DIYer **decide what to buy**. Ground-up Rust
rebuild of the prototypes in `spike/` (gitignored; content captured in
`docs/PLAN.md`). **Read `docs/PLAN.md` for the full plan, decisions, backlog,
and vertical-slice order.**

## Repo structure

```
crates/slp-core   # headless geometry + take-off math (member, native)
crates/slp-ui     # Leptos components, runtime-agnostic (member)
                  #   src/components/<name>.{rs, stories.rs, tests.rs}
crates/dokime     # DOGFOOD: component testing for Leptos (member, lib)
crates/theoria    # DOGFOOD: component explorer UI for Leptos (member)
crates/slp-app    # Leptos CSR/WASM app (excluded; built by Trunk; selects csr)
crates/slp-e2e    # playwright-rust dogfood tests (excluded; standalone)
schema/           # LinkML schema -> panschema-generated serde types (Milestone 1+)
materials/        # manifest.toml (committed) + cache/ (GITIGNORED — never commit assets)
docs/PLAN.md      # overview;  docs/stories/ = one detailed doc per story (sliced)
```

The wasm app (`slp-app`) and native-test crate (`slp-e2e`) are **excluded from
the workspace** so `cargo --workspace` stays native and fast, and no member
enables Leptos `csr` (which would clash with dokime's `ssr`) — mirrors the
playwright-rust site / site-e2e split.

## Development commands

```bash
# All native crates: slp-core + slp-ui + dokime + theoria (fast inner loop)
cargo test --workspace
cargo nextest run --workspace

# A single crate's tests
cargo test -p slp-core      # geometry / take-off math
cargo test -p dokime        # dokime's own self-tests
cargo test -p slp-ui        # slp-ui components (rendered via dokime)
cargo test -p theoria       # theoria's Gallery UI (rendered via dokime)

# Manual local testing of the PLANNER (hot reload at http://localhost:8080)
cd crates/slp-app && trunk serve
cd crates/slp-app && trunk build        # one-off build to dist/

# Manual local testing of COMPONENTS via the theoria gallery (Storybook-style;
# generated on the fly, served on http://localhost:8081):
cargo run -p theoria-cli -- serve                            # slp-ui components
cargo run -p theoria-cli -- serve --config theoria-e2e.toml  # theoria's own components
# (editing a component currently needs a re-run; trunk watches the generated
#  gallery crate, not the source crate — a watch-paths enhancement is future work)

# E2E (drives the real WASM in a browser, after building each target)
cd crates/slp-app && trunk build
cargo test --manifest-path crates/slp-e2e/Cargo.toml         # the planner
cargo run -p theoria-cli -- build --config theoria-e2e.toml
cargo test --manifest-path crates/theoria-e2e/Cargo.toml     # the theoria gallery

# Lint / format (clippy pedantic is project policy)
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt
```

## Dogfood sub-projects

`crates/dokime` (component testing) and `crates/theoria` (component explorer UI)
are incubated here and dogfooded against `slp-ui`, to be spun out to their own
repos later (`git subtree split`). Each is a TDD/component-driven sub-project
that follows the **same `PLAN.md` + `docs/stories/` convention as slp**:
`crates/dokime/docs/PLAN.md` and `crates/theoria/docs/PLAN.md` (each with its
own `docs/stories/`).

**Demand-driven rule:** a theoria/dokime story or slice is built **only when an
slp slice needs it** (or dokime when slp *or* theoria needs it) — never
speculatively. dokime tests itself; theoria is tested *by* dokime (we never
preview theoria inside theoria).

### One-time setup

- `rustup target add wasm32-unknown-unknown`
- `cargo install trunk`
- E2E browsers: `npx playwright@1.60.0 install chromium`
- Pre-commit hooks: `cargo install prek && prek install`

## How to test as you develop

1. **Component / unit logic:** put math in `slp-core` and unit-test it natively
   (`cargo test -p slp-core`) — instant feedback, no browser.
2. **Manual / visual:** `trunk serve` and look at `http://localhost:8080`; hot
   reload picks up edits. Use **theoria** (component explorer) to work on a
   single component in isolation as the tree grows.
3. **Component tests:** **dokime** for Leptos components (added Milestone 1+).
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
- **Clean-sheet `.slp.json`** — no coupling to the spike formats.
- Edition 2024, MSRV 1.95, `unsafe_code = "forbid"`, clippy `pedantic`.

## Delivery

Delivery milestones (see `docs/PLAN.md` §6). **Milestone 0 (walking skeleton) is done:**
workspace + Leptos app renders the yard to scale + first playwright-rust E2E
green. **Milestone 1 = "draw a paver area & see cost."**

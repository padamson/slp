# Mutation-testing policy

We use [cargo-mutants](https://mutants.rs/) to measure whether the test suite
actually *constrains* the logic — not just that it executes it. A mutant is a
small change to the source (flip a `<`, delete a `!`, swap `+=`/`-=`); a mutant
is **caught** if some test fails because of it and **missed** if every test
still passes. Missed mutants are gaps: behavior no test pins down.

## Scope — logic crates only

cargo-mutants runs the **native** test suite (`cargo nextest`) against each
mutant, so it can only catch mutants a native test exercises. That makes it a
fit for pure-logic crates and a poor fit for the Leptos *view* crates, whose
behavior is verified in a real browser (playwright) — which mutation testing
can't drive. Mutating view code yields surviving-mutant **noise**, not signal.

In scope (`examine_globs` in [`.cargo/mutants.toml`](../.cargo/mutants.toml)):

| Crate | What's mutated |
|---|---|
| `slp-core` | geometry / take-off math, the node-placement engine |
| `dokime` | render / count test helpers |
| `theoria-cli` | gallery generation |
| `theoria-macros` | the `#[story]` proc-macro: type mapping + doc/source extraction |

Out of scope, deliberately: `slp-ui`, `slp-app`, `theoria` (Leptos view) and
`slp-e2e`, `theoria-e2e` (browser e2e). Their correctness is the e2e suites' job.

## Target — 0 missed

Every surviving mutant must be resolved one of two ways:

1. **Add a test** that the mutant breaks (the default — a real gap).
2. **Exclude it with justification** via `exclude_re` in `.cargo/mutants.toml`,
   only when the mutant is genuinely not a gap:
   - **Equivalent** — the mutated program behaves identically (e.g. a shoelace
     sum negated then `.abs()`d; `<`↔`>` on a parity count).
   - **Boundary-unspecified** — changes behavior only at an input the function
     documents as unspecified (e.g. a point exactly on a polygon vertex/edge).
   - **Orchestration / I/O** — pure glue (arg parsing, shelling out) covered
     end-to-end by an e2e suite, not by the native tests mutants runs.

   No blanket excludes. Each pattern carries a one-line rationale comment.

## Three enforcement points

1. **Per-push diff gate** (`mutation-testing-diff` in
   [`security.yml`](../.github/workflows/security.yml)) — runs
   [`scripts/mutants.sh`](../scripts/mutants.sh) with `--in-diff` over the lines
   a push/PR touched. Fast; blocks *new* escapes. **Blind to unchanged code.**
2. **Weekly full sweep** (`mutation-testing`, `schedule` + `workflow_dispatch`)
   — runs the whole in-scope config to 0. This is the backstop the diff gate
   can't be: it catches latent survivors in code no recent PR touched, which is
   how backlog silently accumulates. Bounded by `examine_globs`, so it runs
   within the CI per-job timeout.
3. **Local pre-push** — before pushing changes to an in-scope crate, run
   `cargo mutants -f <changed-file>` (or `./scripts/mutants.sh <base>`). The
   diff gate is CI-only and not in the pre-commit hooks, so this is the
   developer's responsibility on logic changes.

## Spin-out alignment

`theoria` and `dokime` are incubated here and will be split out (`git subtree
split`) to their own repos. `.cargo/mutants.toml` lives at the slp root and
**won't travel**. When a crate spins out, its spin-out unit must carry its own
`.cargo/mutants.toml` (scope = that unit's logic crates) and a mutation CI job,
so this policy survives the move. Spin-out units:

- **theoria** → `theoria` + `theoria-cli` + `theoria-macros` + `theoria-e2e`
  (mutate `theoria-cli`, `theoria-macros`).
- **dokime** → `dokime` (mutate `dokime`).

## Running locally

```bash
cargo mutants                              # full in-scope sweep (slow; matches weekly CI)
cargo mutants -f crates/slp-core/src/geom.rs   # one file (fast; before pushing)
./scripts/mutants.sh main                  # only what changed vs. main (diff; matches per-push CI)
```

Reports land in `mutants.out/` (gitignored). A non-zero exit means missed
mutants — fix per the target above.

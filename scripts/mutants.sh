#!/usr/bin/env bash
#
# Run cargo-mutants locally over the lines changed in HEAD (or any base
# ref you choose), so the runtime is in the "useful while iterating"
# range rather than "leave on overnight."
#
# Usage:
#   ./scripts/mutants.sh                      # diff HEAD~1..HEAD
#   ./scripts/mutants.sh main                 # diff main..HEAD
#   ./scripts/mutants.sh 0bb7329              # diff <sha>..HEAD
#   ./scripts/mutants.sh HEAD~5               # diff last 5 commits
#   ./scripts/mutants.sh --working            # diff the working tree vs HEAD
#   ./scripts/mutants.sh -- --jobs 4          # default base + extra cargo-mutants args
#   ./scripts/mutants.sh main --jobs 4        # explicit base + extra args
#   ./scripts/mutants.sh --working --jobs 4   # uncommitted changes + extra args
#
# The first non-dash argument is the base ref; anything else (and
# everything after the first dash-prefixed arg) passes through to
# cargo-mutants. See https://mutants.rs/ for the full CLI surface.
#
# `--working` mutates the lines you've changed but NOT yet committed
# (`git diff HEAD`), so the diff gate matches a pause-before-commit
# workflow instead of only seeing committed history. It covers edits to
# tracked files; a brand-new file needs `git add -N <file>` first to
# appear in the diff.
#
# Why `--in-diff`: an unscoped `cargo mutants` run grows linearly with
# codebase size and routinely runs many hours. `--in-diff` narrows
# mutation to just the lines in the supplied diff — typically seconds
# to minutes for a normal commit.
#
# Prerequisites: `cargo install cargo-mutants` (once per machine).
#
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# --- Project-specific pre-setup ----------------------------------------
# cargo-mutants copies the source tree to a tempdir before building.
# Anything the build expects but that isn't checked in (wasm-pack
# artifacts referenced via include_str!/include_bytes!, generated FFI
# bindings, etc.) won't follow. Mutation testing doesn't exercise those
# bytes at runtime, so empty placeholders are enough.
#
# Example (delete or adapt for your project):
#   mkdir -p crates/foo-viz/pkg
#   touch crates/foo-viz/pkg/foo_viz.js crates/foo-viz/pkg/foo_viz_bg.wasm
# -----------------------------------------------------------------------

# Pull `--working` out of the args (it may sit anywhere); it selects the
# working-tree-vs-HEAD diff instead of a committed range. Everything else
# is left in place for the base-ref / passthrough logic below.
WORKING=0
REST=()
for arg in "$@"; do
  if [[ "$arg" == "--working" ]]; then
    WORKING=1
  else
    REST+=("$arg")
  fi
done
set -- ${REST[@]+"${REST[@]}"}

# Resolve the base ref: first non-dash positional arg, defaulting to
# HEAD~1. Anything starting with `-` is treated as a cargo-mutants arg.
# Skipped for --working, which diffs the working tree, not a ref range.
if [[ "$WORKING" -eq 0 ]]; then
  if [[ $# -gt 0 && "$1" != -* && "$1" != "--" ]]; then
    BASE="$1"
    shift
  else
    BASE="HEAD~1"
  fi

  # `--` separator is allowed for clarity; consume it so it doesn't pass
  # through to cargo-mutants as a positional.
  if [[ $# -gt 0 && "$1" == "--" ]]; then
    shift
  fi

  # The base ref may not resolve — most commonly on the repo's very first
  # (root) commit, where HEAD~1 has no parent. Skip rather than error (git would
  # exit 128), so CI's per-diff mutation job is a no-op on the initial commit.
  if ! git rev-parse --verify --quiet "${BASE}^{commit}" >/dev/null; then
    echo "base ref '${BASE}' does not resolve (e.g. root commit has no parent) — skipping mutation testing."
    exit 0
  fi
fi

DIFF="$(mktemp -t mutants.XXXXXX.diff)"
trap 'rm -f "$DIFF"' EXIT

# --working: uncommitted edits to tracked files (`git diff HEAD`), so the
# gate works before you commit. Otherwise: the committed range BASE..HEAD.
if [[ "$WORKING" -eq 1 ]]; then
  RANGE="the working tree vs HEAD"
  EMPTY_TIP="tip: --working mutates uncommitted edits to tracked files; 'git add -N <file>' a brand-new file to include it."
  git diff HEAD > "$DIFF"
else
  RANGE="${BASE}..HEAD"
  EMPTY_TIP="tip: commit your changes locally first, then re-run (or use --working for uncommitted edits)."
  git diff "${BASE}..HEAD" > "$DIFF"
fi

if [[ ! -s "$DIFF" ]]; then
  echo "no diff for ${RANGE} — nothing to mutate."
  echo "$EMPTY_TIP"
  exit 0
fi

echo "mutating changes in ${RANGE} ($(wc -l < "$DIFF") diff lines)"
exec cargo mutants --in-diff "$DIFF" "$@"

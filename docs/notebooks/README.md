# Lab notebooks

Experiment logs that record **why** a design choice was made, with the evidence
that produced it.

A story doc (`docs/stories/`) says *what* we're building and in what slices. A
notebook says *why we picked this approach over the alternatives* — the runs we
did, what came back, and what we decided as a result. The two link to each
other: the story's "Notes / refs" points at the notebook; the notebook names the
story it fed.

Write one when a decision rests on **empirical results rather than reasoning** —
a parameter sweep, a spike against an unfamiliar API, a "does this library
actually do what its README claims" check. Don't write one for a choice you can
justify in a sentence in the story doc.

## Convention

One file per investigation, dated:

```
docs/notebooks/YYYY-MM-DD-<topic>.md
docs/notebooks/img/YYYY-MM-DD-<topic>/*.jpg     # evidence, if visual
```

Each notebook carries:

- **The question** — what we didn't know, in one line.
- **Setup** — exact versions, models, hardware; enough that a reader can tell
  whether a finding still applies to them.
- **The runs** — what was tried, with **every parameter recorded**, and what came
  back. Include the experiments that failed or were botched; a wrong turn
  documented is a wrong turn nobody repeats.
- **Decisions** — what changed in the design because of this, stated plainly.
- **Still open** — what wasn't settled.

## Images

Record parameters precisely enough that any image here can be **regenerated**.
That makes the committed images *evidence*, not source of truth — and lets us
keep them small.

Downscale to ~600 px JPEG (roughly 40 KB each) before committing. Full-resolution
output stays out of the repo: git keeps every blob forever, and a notebook is
worth about a megabyte, not ten.

This is the same instinct as the materials rule in `CLAUDE.md` — provenance in
the repo, binaries out — relaxed only because these images are ours, tiny, and
the whole point of a visual experiment.

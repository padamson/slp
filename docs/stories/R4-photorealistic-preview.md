# R4 — Photorealistic AI preview (impression)

*Epic R — rendering & views. A **for-fun** photorealistic impression of the
current plan: turn the drawn layout into a natural-language scene description
plus a set of reference/control images, hand them to an image-generation model,
and show the result. It is **not** a purchase-decision surface — the 2D layout
and the estimate stay the source of truth for what to buy; this is "get an idea
of what it'll look like." Deliberately **backend-agnostic**: the app assembles
the prompt + images and calls a small `slpRender` bridge, exactly mirroring the
M4 `slpVision` browser-direct pattern, so the model can be a local runtime
(SwarmUI over ComfyUI, running Z-Image — the default) or a hosted API (fal.ai
Z-Image, Google Gemini) behind the same interface. Nothing is stored
in the plan file — it generates on demand, and its config lives in `localStorage`
like the API key.*

## Story

As a DIY homeowner, I want to press a button and see a photorealistic
impression of the backyard I've drawn — my deck, patio, hot tub, plantings, in
roughly the arrangement and materials I chose — so that I can enjoy imagining
the finished space, without it having to be dimensionally exact.

## Design notes (settled)

- **Two view modes** (both shipped; user picks per generation), both on the same
  **local** Z-Image model — settled empirically, see
  [the conditioning notebook](../notebooks/2026-07-26-preview-conditioning.md):
  - **Overhead (faithful).** Rasterize the 2D plan and feed it as an
    **`initimage`** (img2img) at creativity ≈0.85 → a photorealistic bird's-eye
    render that keeps things where they were drawn. **No ControlNet** — img2img
    alone held the layout, so the 1.8 GB ControlNet model and its union-type /
    preprocessor config drop out of the design.
  - **Eye-level (immersive).** The same call *without* an init image — plain
    text-to-image on `scene_prompt`. Plausible perspective, layout-approximate.
  - *A hosted instruction model (Gemini / gpt-image-1) remains an optional
    fidelity upgrade for layout-aware eye-level, but nothing requires it.*
- **Conditioning, not just a prompt.** Text alone gives an unrelated yard; the
  **control image** (plan raster, or a simplified flat-color massing) pins the
  layout, and **reference images** (catalog material/product photos, already
  data URIs) make textures/products read right. `scene_prompt` supplies scene,
  style, and — for eye-level — the "render at eye level" instruction.
- **Backend tiers** (same app-side interface; only the adapter differs):
  - **SwarmUI (local, default for *overhead*)** — ComfyUI engine, Z-Image +
    clean JSON REST (`POST /API/<route>`). Free, private, offline.
  - **Hosted instruction model (for *eye-level*)** — Gemini 2.5 Flash Image
    (up-to-20 described references) or gpt-image-1 edits. Per-image cost, key.
  - **fal.ai Z-Image** — hosted overhead alternative if local SwarmUI is a
    hassle. *(Plain Ollama image-gen is text-only — no conditioning — a degraded
    fallback, not a target.)*
- **Config in `localStorage`, never in the plan** (mirrors `api_key.rs`): the
  per-mode backend kind, endpoint URL, and optional API key. A plan file stays
  portable and key-free.

## Vertical slices

- **R4.0 — scene prompt + reference set (`slp-core`, headless)** ✅
  - [x] `scene_prompt(plan) -> String`: a natural-language description of the
        yard — its size, the house, the deck (levels + material), each drawn
        area (a paver patio with its pattern + material, mulch beds), and the
        placed objects grouped by category with rough position + count (a hot
        tub in a corner, a fire pit, N boxwoods along a side, furniture on the
        deck), plus an overall style line. Deterministic; unit + mutation tested.
  - [x] a position-phrasing helper mapping an `(x, y)` within the yard bounds to
        human terms ("near the back-left corner", "along the east fence") — pure
        and tested (it's the part most prone to silent off-by-one/side errors).
  - [x] `reference_images(plan) -> Vec<Reference>`: the distinct catalog
        material/product images present in the plan (deduped, capped to a model
        limit), each with a short label ("paver texture", "hot tub"). Pure.

- **R4.1 — the render bridge + SwarmUI adapter (browser)** ✅
  - [x] `window.slpRender` in `index.html`, shaped like `slpVision`:
        `generate(mode, prompt, controlImage, refsJson, configJson)` →
        `{image, source}` (R4.5 added `source`: where the backend keeps its own
        full-res copy). `mode` is `"overhead"`, `"eye-level"` or `"from-photo"`.
  - [x] SwarmUI flow: `GetNewSession` → `GenerateText2Image` → fetch the returned
        `/View/…` path and inline it as a `data:` URI. Overhead attaches the plan
        raster as `initimage` at `creativity`; eye-level sends no image. The bridge
        attaches an init image whenever one is *supplied*, so the app decides
        what to condition on rather than the bridge switching on the mode name.
  - [x] `render_config.rs` — `localStorage` config (endpoint, model, size, steps,
        creativity), reusing the `api_key.rs` storage approach; `render.rs` — the
        `wasm-bindgen` glue, csr-gated with an inert non-browser stub.
  - [x] e2e stubs `window.slpRender` (as the vision e2e stubs `slpVision`) so
        tests never hit a real model — the app's own `planRaster` still runs, so
        the stub covers rasterizing the live SVG too.

- **R4.2 — control image: rasterize the current plan** ✅
  - [x] `slpRender.planRaster` captures the plan to a PNG data URI in-browser
        (clone → strip → remap → `<canvas>` → `toDataURL`). It paints the
        background *before* drawing: a transparent PNG flattens to black on a
        canvas, and black ground photorealizes as asphalt.
  - [x] **a preview-specific render, not a screenshot** — `preview_render.rs`
        holds the rules (unit-tested): the near-white lawn and grid lines remap
        to grass green; selection blue and the two warning reds (app state, not
        the yard) neutralize; material fills are deliberately left alone, since
        they're the signal img2img works from. Editing chrome — nodes, handles,
        node numbers, the dashed clearance ring — is stripped; physical things
        (the hot tub's pad, a tree's trunk, borders) stay.
  - [ ] *moved to R4.7:* feed the **tiled material photos** SLP already supports
        into the raster instead of flat fills.

- **R4.3 — Preview UI: button, mode toggle, result modal** ✅
  - [x] a **Preview** button under the estimate with an **Overhead / Eye-level**
        toggle; it assembles the prompt + raster + references and calls
        `slpRender` for the chosen mode. Overhead *fails loudly* when there's no
        plan to rasterize rather than silently degrading to a prompt-only render.
  - [x] a portaled result modal (like the crop editor) with a spinner, the image,
        Regenerate, and a failure state naming both the cause and the fix.
  - [x] dokime component tests (every state, both modes) + e2e against a stubbed
        backend: overhead sends a real raster, eye-level sends none, and an
        unreachable backend is reported rather than swallowed.
  - [ ] *deferred:* a config surface. The defaults work against a stock local
        SwarmUI through the dev-server proxy, so there's nothing to configure
        yet; a different endpoint can be set via `localStorage`. This earns a UI
        when there's a second backend worth switching between.

- **R4.4 — tests + docs + verification** ✅
  - [x] full workspace (526) + full e2e (71) green; clippy pedantic + fmt clean.
        Mutation gate: **N/A this slice** — no new `slp-core` logic (R4.0's
        `scene.rs` gated 0-missed; `slp-ui` is out of mutation scope by design).
  - [x] tick this doc; stories index + PLAN.md later-row updated; the setup and
        the parameter evidence live in
        [the conditioning notebook](../notebooks/2026-07-26-preview-conditioning.md).

- **R4.5 — keep what you generate** ✅
  - [x] **Download** the render — an `<a download>` on the data URI, named
        `<plan-stem>-<mode>.png` so a folder of them stays legible. No JS, no
        bridge round trip.
  - [x] **Say where it already is on disk.** SwarmUI writes every generation to
        `Output/local/raw/<date>/…` before we ever fetch it, so the modal shows
        that path rather than implying it holds the only copy. A browser can't
        open Finder, so a selectable path is the honest affordance — not a button
        that pretends to. (The bridge returns it as `source`; `None` for a
        backend that hands back bytes and keeps nothing.)
  - [x] **A gallery of recent renders** — a thumbnail per render, click to
        reopen, so Regenerate no longer discards the one you liked. Compare-two
        would fall out of this naturally (still open).
  - [x] *Storage decision: **session-only**, in memory.* A 768×768 PNG is ~1 MB
        as a data URI and `localStorage` caps around 5 MB, with the plan itself
        living there (`slp:plan`) — persisting renders would evict the user's
        actual work. Session-only is the right call precisely *because* the
        backend keeps the full-res copies on disk: there's nothing to persist and
        nothing to evict. IndexedDB stays available if the gallery ever needs to
        survive a reload.

- **R4.6 — seed from real photos of the actual yard** 🚧
  - *Explored 2026-07-26; findings in
    [the notebook](../notebooks/2026-07-26-preview-conditioning.md) §2–3. The
    key result: img2img conditions on a **whole scene**, not a component, and
    IP-Adapter is unavailable for Z-Image (`useipadapter` offers only `None`).
    So "here's a photo of my house, put that house in a new layout" is not
    reachable locally — which splits this into two different features.*
  - [x] **"My actual yard, finished"** — a third **From photo** mode: upload a
        photo of the real backyard and it becomes the `initimage` instead of the
        plan raster, so the model re-renders *that* yard with the planned
        materials. The photo is **session-only** (never persisted — see the
        storage note in R4.5), and generating is blocked until one is chosen
        rather than silently falling back to a prompt-only render.
  - [x] **a separate creativity for photos** (`photoCreativity`, default 0.55 vs
        overhead's 0.85). The gap is the whole point: 0.85 is high because a
        *flat vector plan* needs that much noise before the model paints texture
        over it — a photograph is already photographic, and the same setting
        would throw the yard away. 0.55 is the notebook's run-3 value, which
        kept the house, deck and layout while re-surfacing materials.
  - [ ] **Per-item photos → text, via the vision bridge we already have.** A
        photo of the house through `slpVision` yields "cream stucco ranch, dark
        shingle roof, white trim"; fold that into `scene_prompt` so both modes
        render *your* house's character. Cheap (a few hundred bytes, not a
        megabyte), local-model-friendly, and it improves eye-level most.
  - [ ] where the photo/description attaches: `existing`-status objects and the
        house/deck are the natural anchors (SLP already models "already owned").
        Needs a schema decision — a description string is safe to persist; raw
        photos are **not** (same `localStorage` budget problem as R4.5).
  - [ ] *not viable locally:* true appearance transfer of a single component.
        Revisit only if a Z-Image-compatible IP-Adapter appears, or via a hosted
        instruction model that accepts described references.

- **R4.7 — a better raster** *(was R4.2's deferred item)*
  - [ ] feed SLP's **tiled material photos** (already supported on drawn areas)
        into the preview raster instead of flat fills — a semi-photographic init
        image should beat flat color, and it's free.
  - [ ] texture the lawn: the sweep's renders kept a flat green lawn because the
        input had no grass texture to build on.
  - [ ] does adding `ControlNet` *on top of* img2img fix the fire-pit drift?
        SwarmUI accepts both in one request.

## Notes / refs

- **Reuses what's already here.** The bridge is a near-copy of `slpVision`
  (browser-direct model call, key in `localStorage`, e2e-stubbable). The catalog
  already stores material/product images as data URIs (M4 ingestion) — ready-made
  reference images. The 2D SVG render is the control-image source.
- **No schema change.** The preview is generate-on-demand; nothing persists to
  the plan file. Backend config is app/browser config in `localStorage`, the same
  boundary the Anthropic key already respects.
- **Expectation-setting.** Overhead mode tracks the layout closely (it's the
  plan, rendered real); eye-level mode uses the plan for arrangement but takes
  more liberty (perspective is inferred). Either way it's an *impression*, not a
  measured view — exact dimensions/positions are the 2D plan's job.
- **Relationship to R2 (deterministic 3D).** Orthogonal. R2 is an accurate,
  measurable extruded 3D view over `slp-core`; R4 is a fun photorealistic
  impression via a generative model. R4 does not depend on R2, though R2 (or the
  massing raster from R4.2) would make an even better control image if it lands.
- **Backend research (2026-07):** SwarmUI (local, ComfyUI engine, Z-Image, JSON
  REST) is the default and needs no paid API; fal.ai Z-Image (~$0.005–0.0065/MP)
  and Gemini 2.5 Flash Image (~$0.039/image, up to 20 references) are hosted
  drop-ins; Ollama's image API is text-only (no conditioning).
- **Setup:** [`docs/preview-backend.md`](../preview-backend.md) — install, models, CORS, defaults.
- **Evidence:** parameters, failure modes, and the ControlNet-vs-img2img call are
  recorded in [`docs/notebooks/2026-07-26-preview-conditioning.md`](../notebooks/2026-07-26-preview-conditioning.md).
  Two environment gotchas live there too: the FP8 Z-Image build can't run on
  Apple Silicon, and SwarmUI needs `AccessControlAllowOrigin` set before a
  browser can call it.

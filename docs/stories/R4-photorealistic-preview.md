# R4 — Photorealistic AI preview (impression)

*Epic R — rendering & views. A **for-fun** photorealistic impression of the
current plan: turn the drawn layout into a natural-language scene description
plus a set of reference/control images, hand them to an image-generation model,
and show the result. It is **not** a purchase-decision surface — the 2D layout
and the estimate stay the source of truth for what to buy; this is "get an idea
of what it'll look like." Deliberately **backend-agnostic**: the app assembles
the prompt + images and calls a small `slpRender` bridge, exactly mirroring the
M4 `slpVision` browser-direct pattern, so the model can be a local runtime
(SwarmUI over ComfyUI, running Z-Image + ControlNet — the default) or a hosted
API (fal.ai Z-Image, Google Gemini) behind the same interface. Nothing is stored
in the plan file — it generates on demand, and its config lives in `localStorage`
like the API key.*

## Story

As a DIY homeowner, I want to press a button and see a photorealistic
impression of the backyard I've drawn — my deck, patio, hot tub, plantings, in
roughly the arrangement and materials I chose — so that I can enjoy imagining
the finished space, without it having to be dimensionally exact.

## Design notes (decisions to lock in R4.1)

- **Two view modes** (both shipped; user picks per generation), because the
  control image's viewpoint drives the output's:
  - **Overhead (faithful).** Rasterize the 2D plan and feed it as a **ControlNet
    structure** to a local Z-Image run → a photorealistic **bird's-eye** render
    that tracks exactly where things were drawn. Coherent, layout-accurate,
    free/offline, achievable now.
  - **Eye-level (immersive).** Hand the same plan raster + the material photos to
    an **instruction-following multimodal model** (Gemini 2.5 Flash Image /
    gpt-image-1) told to "render an eye-level photo of this backyard" → a
    "standing in it" perspective that uses the plan for arrangement. More
    flexible, hosted/paid, less rigidly faithful.
  - *Why not overhead→perspective locally: a strict ControlNet can't reproject a
    top-down structure into correct perspective. Faithful perspective needs the
    instruction model, or eventually R2's 3D view to render a perspective control
    image.*
- **Conditioning, not just a prompt.** Text alone gives an unrelated yard; the
  **control image** (plan raster, or a simplified flat-color massing) pins the
  layout, and **reference images** (catalog material/product photos, already
  data URIs) make textures/products read right. `scene_prompt` supplies scene,
  style, and — for eye-level — the "render at eye level" instruction.
- **Backend tiers** (same app-side interface; only the adapter differs):
  - **SwarmUI (local, default for *overhead*)** — ComfyUI engine, Z-Image +
    ControlNet, clean JSON REST (`POST /API/<route>`). Free, private, offline.
  - **Hosted instruction model (for *eye-level*)** — Gemini 2.5 Flash Image
    (up-to-20 described references) or gpt-image-1 edits. Per-image cost, key.
  - **fal.ai Z-Image** — hosted overhead alternative if local SwarmUI is a
    hassle. *(Plain Ollama image-gen is text-only — no conditioning — a degraded
    fallback, not a target.)*
- **Config in `localStorage`, never in the plan** (mirrors `api_key.rs`): the
  per-mode backend kind, endpoint URL, and optional API key. A plan file stays
  portable and key-free.

## Vertical slices

- **R4.0 — scene prompt + reference set (`slp-core`, headless)**
  - [ ] `scene_prompt(plan) -> String`: a natural-language description of the
        yard — its size, the house, the deck (levels + material), each drawn
        area (a paver patio with its pattern + material, mulch beds), and the
        placed objects grouped by category with rough position + count (a hot
        tub in a corner, a fire pit, N boxwoods along a side, furniture on the
        deck), plus an overall style line. Deterministic; unit + mutation tested.
  - [ ] a position-phrasing helper mapping an `(x, y)` within the yard bounds to
        human terms ("near the back-left corner", "along the east fence") — pure
        and tested (it's the part most prone to silent off-by-one/side errors).
  - [ ] `reference_images(plan) -> Vec<Reference>`: the distinct catalog
        material/product images present in the plan (deduped, capped to a model
        limit), each with a short label ("paver texture", "hot tub"). Pure.

- **R4.1 — the render bridge + two adapters (browser)**
  - [ ] `window.slpRender` in `index.html`, shaped like `slpVision`:
        `generate({ mode, prompt, controlImage, references, config }) -> dataUri`,
        where `mode` is `"overhead"` or `"eye-level"`, dispatching on the
        per-mode backend in `config`.
  - [ ] **overhead adapter — SwarmUI/Z-Image ControlNet** (control image →
        bird's-eye render, base64 back). Confirm SwarmUI's exact Z-Image +
        ControlNet route/payload and CORS (allow the app origin).
  - [ ] **eye-level adapter — a hosted instruction model** (Gemini 2.5 Flash
        Image or gpt-image-1): plan raster + references + an "eye-level"
        instruction → perspective render. Browser-direct with a user key, the
        `slpVision`/Anthropic pattern.
  - [ ] a `configured(mode)`/reachability check per mode; `slpRender` config in
        `localStorage` (per-mode backend + endpoint + optional key), reusing the
        `api_key.rs` approach.
  - [ ] e2e stubs `window.slpRender` (as the vision e2e stubs `slpVision`) so
        tests never hit a real model.

- **R4.2 — control image: rasterize the current plan**
  - [ ] capture the live plan `<svg>` to a PNG data URI in-browser
        (serialize → `<canvas>` → `toDataURL`) — used as the ControlNet
        structure for overhead and a reference image for eye-level. Evaluate a
        **simplified massing** render (flat category-colored blocks) as a
        cleaner ControlNet input than the detailed plan; pick one.

- **R4.3 — Preview UI: button, mode toggle, config, result modal**
  - [ ] a **Preview** button (by the estimate/catalog) with an **Overhead /
        Eye-level** toggle; it assembles the prompt + control image + references
        and calls `slpRender` for the chosen mode.
  - [ ] a small config surface (per-mode backend + endpoint + optional key) and
        a portaled result modal (like the crop editor) with a spinner, the
        image, a Regenerate button, and a graceful empty state ("start your
        local preview backend" / "add a key") when the mode's backend is
        unconfigured/unreachable.
  - [ ] dokime component tests + e2e (stubbed backend, both modes): click
        Preview → prompt + mode assembled from the plan → the stubbed image
        shown.

- **R4.4 — tests + docs + verification**
  - [ ] slp-core mutation gate (0-missed) on the prompt/reference/position logic;
        full workspace + e2e green.
  - [ ] tick this doc; update the stories index + PLAN.md later-row; add a short
        `docs/` note on running a local backend (SwarmUI + Z-Image) and the
        hosted alternatives.

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
- **Backend research (2026-07):** SwarmUI (local, ComfyUI engine, Z-Image +
  ControlNet, JSON REST) is the default; fal.ai Z-Image (~$0.005–0.0065/MP) and
  Gemini 2.5 Flash Image (~$0.039/image, up to 20 references) are hosted
  drop-ins; Ollama's image API is text-only (no conditioning).

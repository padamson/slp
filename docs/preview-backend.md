# Running the preview backend

The photorealistic preview (R4) renders through a local image model. Everything
runs on your machine — no API key, no per-image cost, nothing leaves the laptop.
Without a backend the app still works; the Preview button just reports that it
can't reach one.

## What you need

**[SwarmUI](https://github.com/mcmonkeyprojects/SwarmUI)** — it runs ComfyUI as
its engine but exposes a plain JSON REST API, which is what the app talks to.

Then one model, into SwarmUI's `Models/diffusion_models/`:

```
z_image_turbo_bf16.safetensors        # ~11 GB
https://huggingface.co/Comfy-Org/z_image_turbo/resolve/main/split_files/diffusion_models/z_image_turbo_bf16.safetensors
```

SwarmUI auto-downloads the matching text encoder and VAE on first use.

**Take the BF16 build, not FP8.** The FP8 variant fails on Apple Silicon with
`Undefined type Float8_e4m3fn` — MPS has no FP8. BF16 wants ~16 GB of (unified)
memory.

No ControlNet model is needed. Overhead mode conditions with img2img instead;
see [the conditioning notebook](notebooks/2026-07-26-preview-conditioning.md)
for why.

## Letting the browser talk to it

Nothing to configure under `trunk serve`. `Trunk.toml` proxies `/swarm/` to
`localhost:7801`, so the app calls a **same-origin** path and no CORS check ever
happens — which also means it doesn't matter which port the dev server lands on.

That last part is the reason for the proxy. `Access-Control-Allow-Origin` takes
a single exact origin (or `*`) — no lists, no ranges — so pinning it to
`http://localhost:8080` breaks the moment the dev server picks 8081 because
another app already had 8080.

**Serving the built app instead of `trunk serve`?** There's no proxy then, so
point `endpoint` straight at the backend (below) and set, in
`SwarmUI/Data/Settings.fds`:

```
AccessControlAllowOrigin: http://localhost:8080     # or * for any local port
```

Restart SwarmUI, then check it took:

```bash
curl -s -i -XPOST http://localhost:7801/API/GetNewSession \
  -H 'content-type: application/json' -H 'Origin: http://localhost:8080' -d '{}' \
  | grep -i access-control-allow
```

`*` works with any port, at the cost of letting *any* page you have open drive
your SwarmUI — its API is unauthenticated and can, among other things, shut the
server down. Fine on a personal machine; the proxy avoids the question entirely.

## Defaults, and changing them

The app assumes a stock install — the `/swarm` proxy,
`z_image_turbo_bf16.safetensors`, 768×768, 20 steps, creativity 0.85. Those
values come from the sweep in the notebook.

There's no settings UI yet. To point somewhere else, set `slp.renderConfig` in
`localStorage`:

```js
localStorage.setItem("slp.renderConfig", JSON.stringify({
  endpoint: "/swarm",            // dev-server proxy; use "http://localhost:7801" without trunk serve
  model: "z_image_turbo_bf16.safetensors",
  width: 768, height: 768, steps: 20, cfgscale: 1.0,
  creativity: 0.85,   // lower = follows the plan more literally; below ~0.8 it stays a flat diagram
}));
```

## What to expect

First render loads ~11 GB, so it takes a minute or two; afterwards the model
stays resident and a render is roughly 20–70 seconds.

**Overhead** conditions on a render of your plan, so the layout comes back close
to what you drew. **Eye-level** goes on the scene description alone — plausible,
and it honors the materials and rough arrangement, but it isn't your yard's
geometry.

Either way it's an impression, not a measurement. The 2D plan and the estimate
remain the source of truth for what to buy.

## Troubleshooting

| Symptom | Cause |
|---|---|
| "Can't reach the image backend" | SwarmUI isn't running; or you're on a built app (no proxy) without CORS set |
| `Undefined type Float8_e4m3fn` | FP8 model on Apple Silicon — use the BF16 build |
| A flat, cartoonish diagram | `creativity` too low; 0.85 is the photoreal knee |
| "There's no plan on screen to preview yet" | Overhead needs something drawn |

Hosted alternatives (fal.ai Z-Image, Google Gemini) would slot in behind the
same `slpRender` bridge, at a per-image cost. Nothing in the app requires them.

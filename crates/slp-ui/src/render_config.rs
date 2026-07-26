//! Config for the photorealistic preview backend (R4): the `SwarmUI` endpoint,
//! model, and per-mode knobs. Stored in `localStorage` as app/browser config —
//! **not** in the `Plan` — so an exported `.slp.json` never carries a machine's
//! endpoint (the same boundary [`crate::api_key`] keeps for the API key).
//! csr-gated; off the browser it just yields the default.

/// `localStorage` key for the preview config JSON — namespaced, distinct from
/// the plan's own key. Only the browser (`csr`) read/write path references it.
#[cfg(feature = "csr")]
pub const RENDER_CONFIG_STORAGE: &str = "slp.renderConfig";

/// Starter config the `slpRender` bridge understands: the local `SwarmUI`,
/// reached **through the dev server's proxy**, generating with Z-Image Turbo.
///
/// `endpoint` is the relative path `/swarm`, which `Trunk.toml` forwards to
/// `localhost:7801`. That makes every call **same-origin**, so the browser runs
/// no CORS check and the backend needs no `AccessControlAllowOrigin` pinned to
/// whichever port the dev server happened to get. Pointing straight at
/// `http://localhost:7801` still works — it just needs CORS configured, which
/// is what a built (non-`trunk serve`) app has to do.
///
/// **Overhead** mode sends the plan raster as an `initimage` (img2img) at
/// `creativity` — no `ControlNet`. The defaults come from the sweep in
/// `docs/notebooks/2026-07-26-preview-conditioning.md`: below ~0.8 the render
/// stays a flat diagram, 0.85 with 20 steps produces a photograph, and steps
/// alone can't substitute for creativity. **Eye-level** mode sends no init
/// image — plain text-to-image on the scene prompt.
pub const DEFAULT_RENDER_CONFIG: &str = r#"{
  "endpoint": "/swarm",
  "model": "z_image_turbo_bf16.safetensors",
  "width": 768,
  "height": 768,
  "steps": 20,
  "cfgscale": 1.0,
  "creativity": 0.85
}"#;

/// The effective preview config JSON: the user's stored config, or
/// [`DEFAULT_RENDER_CONFIG`] when none is set (or off the browser).
#[must_use]
pub fn render_config() -> String {
    #[cfg(feature = "csr")]
    {
        if let Some(cfg) = storage()
            .and_then(|s| s.get_item(RENDER_CONFIG_STORAGE).ok().flatten())
            .filter(|c| !c.trim().is_empty())
        {
            return cfg;
        }
    }
    DEFAULT_RENDER_CONFIG.to_string()
}

// There's deliberately no setter yet. The defaults work against a stock local
// `SwarmUI`, so the MVP needs no settings UI; someone pointing at a different
// endpoint can set `slp.renderConfig` in `localStorage` directly. A proper
// config surface lands when there's a second backend worth switching between.

#[cfg(feature = "csr")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_config_is_a_local_swarmui() {
        // Off the browser, the effective config is the default: a localhost
        // `SwarmUI`, and it carries the img2img creativity default.
        let cfg = render_config();
        assert_eq!(cfg, DEFAULT_RENDER_CONFIG);
        assert!(
            cfg.contains("\"endpoint\": \"/swarm\""),
            "same-origin proxy path, so no CORS check applies"
        );
        assert!(cfg.contains("z_image_turbo"), "a Z-Image model by default");
        assert!(
            cfg.contains("\"creativity\""),
            "the img2img dial is configurable"
        );
    }
}

//! Bridge to `window.slpRender.generate` for the photorealistic preview (R4).
//! The heavy lifting (`SwarmUI` session flow, payload, image fetch) lives in the
//! app shell's `window.slpRender` (see `slp-app/index.html`), mirroring the
//! `window.slpVision` ingestion bridge; this is the thin `wasm-bindgen` glue.
//! csr-only — off the browser it's inert, and the app gates the Preview button
//! on the browser build.

/// Rasterize the on-screen plan into the init image for overhead mode: a
/// derived render with the editing chrome stripped and UI colors remapped to
/// material-true ones (see `slp_core::preview_render`).
///
/// # Errors
/// Returns a message when the bridge is absent or there's no plan on screen.
pub async fn plan_raster(size: u32) -> Result<String, String> {
    imp::plan_raster(&slp_core::preview_raster_config(), size).await
}

/// One finished render: the image to show, and where the backend keeps its own
/// full-resolution copy (when it has one).
#[derive(Clone, PartialEq, Debug, serde::Deserialize)]
pub struct Render {
    /// The image as a `data:` URI.
    pub image: String,
    /// The backend's own path for this render — `SwarmUI` writes every generation
    /// to `Output/…` before we fetch it, so the file outlives the browser
    /// session. `None` for a backend that hands back bytes and keeps nothing.
    #[serde(default)]
    pub source: Option<String>,
}

/// Generate a preview for the plan. `mode` is `"overhead"`, `"eye-level"` or
/// `"from-photo"`; `control_image` is the init image for the modes that
/// condition on one; `references_json` and `config_json` pass through.
///
/// # Errors
/// Returns a message when the bridge is absent (non-browser), the backend is
/// unreachable, generation fails, or the reply doesn't parse.
pub async fn generate(
    mode: &str,
    prompt: &str,
    control_image: Option<&str>,
    references_json: &str,
    config_json: &str,
) -> Result<Render, String> {
    let json = imp::generate(
        mode,
        prompt,
        control_image.unwrap_or(""),
        references_json,
        config_json,
    )
    .await?;
    serde_json::from_str(&json).map_err(|e| format!("The preview reply didn't parse: {e}"))
}

#[cfg(feature = "csr")]
mod imp {
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    fn slprender() -> Option<JsValue> {
        let win = web_sys::window()?;
        let v = js_sys::Reflect::get(&win, &JsValue::from_str("slpRender")).ok()?;
        (!v.is_undefined() && !v.is_null()).then_some(v)
    }

    /// Call `window.slpRender.planRaster(rulesJson, size)` and return the plan
    /// rendered to a `data:` URI.
    pub async fn plan_raster(rules: &str, size: u32) -> Result<String, String> {
        let v = slprender().ok_or("The preview isn't available here.")?;
        let f = js_sys::Reflect::get(&v, &JsValue::from_str("planRaster"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .ok_or("The preview isn't available here.")?;
        let promise = js_sys::Reflect::apply(
            &f,
            &v,
            &js_sys::Array::of2(
                &JsValue::from_str(rules),
                &JsValue::from_f64(f64::from(size)),
            ),
        )
        .map_err(|_| "Couldn't rasterize the plan.".to_string())?;
        let promise: js_sys::Promise = promise
            .dyn_into()
            .map_err(|_| "Couldn't rasterize the plan.".to_string())?;
        match JsFuture::from(promise).await {
            Ok(val) => val
                .as_string()
                .ok_or_else(|| "The plan rasterized to nothing.".to_string()),
            Err(e) => Err(js_sys::Reflect::get(&e, &JsValue::from_str("message"))
                .ok()
                .and_then(|m| m.as_string())
                .unwrap_or_else(|| "Couldn't rasterize the plan.".to_string())),
        }
    }

    /// Call `window.slpRender.generate(mode, prompt, controlImage, refsJson,
    /// configJson)`, await it, and return the resulting `data:` URI string.
    pub async fn generate(
        mode: &str,
        prompt: &str,
        control_image: &str,
        references_json: &str,
        config_json: &str,
    ) -> Result<String, String> {
        let v = slprender().ok_or("The preview isn't available here.")?;
        let f = js_sys::Reflect::get(&v, &JsValue::from_str("generate"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .ok_or("The preview isn't available here.")?;
        let args = js_sys::Array::of5(
            &JsValue::from_str(mode),
            &JsValue::from_str(prompt),
            &JsValue::from_str(control_image),
            &JsValue::from_str(references_json),
            &JsValue::from_str(config_json),
        );
        let promise = js_sys::Reflect::apply(&f, &v, &args)
            .map_err(|_| "The preview failed to start.".to_string())?;
        let promise: js_sys::Promise = promise
            .dyn_into()
            .map_err(|_| "The preview returned no result.".to_string())?;
        match JsFuture::from(promise).await {
            Ok(val) => val
                .as_string()
                .ok_or_else(|| "The preview returned an empty image.".to_string()),
            Err(e) => Err(js_sys::Reflect::get(&e, &JsValue::from_str("message"))
                .ok()
                .and_then(|m| m.as_string())
                .unwrap_or_else(|| "The preview failed.".to_string())),
        }
    }
}

#[cfg(not(feature = "csr"))]
mod imp {
    pub async fn plan_raster(_rules: &str, _size: u32) -> Result<String, String> {
        Err("The preview is only available in the browser.".to_string())
    }

    pub async fn generate(
        _mode: &str,
        _prompt: &str,
        _control_image: &str,
        _references_json: &str,
        _config_json: &str,
    ) -> Result<String, String> {
        Err("The preview is only available in the browser.".to_string())
    }
}

//! Browser file I/O for the plan (G1): download the current `Plan` as a
//! `.slp.json` file, and read a user-picked file back into a `Plan`. The file
//! *is* the plan — plain `serde_json` over the panschema-generated `Plan`, the
//! same shape `localStorage` already round-trips; this module only adds the
//! browser affordances (a download anchor, reading a `<input type="file">`).
//!
//! Everything here is `csr`-gated: file I/O exists only in the browser build.
//! Under `ssr`/native tests the entry points are inert no-ops so the crate
//! still compiles and renders.

use slp_core::{Plan, plan_filename};

/// Serialize `plan` to pretty JSON and trigger a browser download named after
/// the plan (`<slug>.slp.json`). No-op off the browser. Returns the filename it
/// used (for display/tests), or `None` if serialization failed.
#[cfg(feature = "csr")]
pub fn download_plan(plan: &Plan) -> Option<String> {
    use wasm_bindgen::JsCast;

    let json = serde_json::to_string_pretty(plan).ok()?;
    let filename = plan_filename(plan);

    // Blob([json], {type: "application/json"}) — a Blob (not a giant data: URI)
    // keeps large plans (embedded material photos) off the anchor's href.
    let parts = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(&json));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("application/json");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts).ok()?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).ok()?;

    let document = web_sys::window()?.document()?;
    let anchor = document
        .create_element("a")
        .ok()?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .ok()?;
    anchor.set_href(&url);
    anchor.set_download(&filename);
    anchor.click();
    // The object URL is held only for the synchronous click; release it.
    let _ = web_sys::Url::revoke_object_url(&url);
    Some(filename)
}

#[cfg(not(feature = "csr"))]
#[must_use]
pub fn download_plan(plan: &Plan) -> Option<String> {
    Some(plan_filename(plan))
}

/// Parse `.slp.json` text into a `Plan`, rejecting malformed input (so a bad
/// file never half-loads). Pure — shared by the browser reader and tests.
///
/// # Errors
/// Returns a human-readable message when the text isn't a valid plan.
pub fn parse_plan(text: &str) -> Result<Plan, String> {
    serde_json::from_str::<Plan>(text).map_err(|e| format!("Not a valid .slp.json plan: {e}"))
}

/// Programmatically open the file picker of the hidden `<input type="file">`
/// with id `input_id` (so a styled button can drive it). No-op off the browser.
#[cfg(feature = "csr")]
pub fn open_file_dialog(input_id: &str) {
    use wasm_bindgen::JsCast;

    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(input_id))
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.click();
    }
}

#[cfg(not(feature = "csr"))]
pub fn open_file_dialog(_input_id: &str) {}

/// Read the file just picked in `ev`'s `<input>` as text and hand the parsed
/// result to `on_result` (`Ok(Plan)` or a human-readable `Err`). Mirrors
/// `FileInput`'s event-based read so the signature is identical across cfgs;
/// a no-op off the browser.
#[cfg(feature = "csr")]
pub fn load_from_event(ev: &leptos::ev::Event, on_result: impl Fn(Result<Plan, String>) + 'static) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    use web_sys::{FileReader, HtmlInputElement};

    let Some(file) = ev
        .target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .and_then(|input| input.files())
        .and_then(|list| list.get(0))
    else {
        return;
    };
    let Ok(reader) = FileReader::new() else {
        return;
    };
    let reader_for_load = reader.clone();
    let onload = Closure::<dyn FnMut()>::new(move || {
        let text = reader_for_load
            .result()
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        on_result(parse_plan(&text));
    });
    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    let _ = reader.read_as_text(&file);
}

#[cfg(not(feature = "csr"))]
pub fn load_from_event(
    _ev: &leptos::ev::Event,
    _on_result: impl Fn(Result<Plan, String>) + 'static,
) {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_round_trips_a_serialized_plan() {
        let plan = Plan {
            name: Some("Back Yard".to_string()),
            yard_width: 40.0,
            yard_depth: 25.0,
            ..Default::default()
        };
        let json = serde_json::to_string(&plan).unwrap();
        let back = parse_plan(&json).expect("valid plan parses");
        assert_eq!(back.name.as_deref(), Some("Back Yard"));
        assert!((back.yard_width - 40.0).abs() < 1e-9);
        assert!((back.yard_depth - 25.0).abs() < 1e-9);
    }

    #[test]
    fn parse_rejects_malformed_json() {
        let err = parse_plan("{not json").unwrap_err();
        assert!(err.contains("valid"), "human-readable error: {err}");
    }

    #[test]
    fn parse_rejects_json_that_is_not_a_plan() {
        // Valid JSON, wrong shape (yard_width must be a number).
        let err = parse_plan(r#"{"yard_width": "wide"}"#).unwrap_err();
        assert!(!err.is_empty(), "reports the shape mismatch");
    }
}

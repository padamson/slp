//! Vision extraction (Phase B3): turn a pasted product screenshot into a draft
//! product via the Anthropic Messages API. The network call is a browser
//! concern — it goes through the app shell's `window.slpVision` bridge (defined
//! in `slp-app/index.html`), which POSTs the image with the user's key and
//! forces a **tool call** whose `input` is validated against a JSON Schema, so
//! the model returns structured data (not free-form text to guess at). This
//! module owns that schema — derived from [`ExtractedProduct`] via `schemars`,
//! one source of truth — the prompt, and the pure parse + guard of the returned
//! input; the parsing is unit-tested natively.
//!
//! Nothing here is live until the user curates it (M4.2): a draft is a
//! suggestion, reviewed and corrected before it becomes catalog items.

use schemars::JsonSchema;

/// The default vision model — cheap and ample for a few fields off a product
/// screenshot. Configurable in the UI.
pub const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";

/// How a material's price is charged. A closed set (a JSON Schema `enum`), so
/// the model can't return a free-form unit like "sqft" or "each". The
/// model-facing tool is named `extract_product` (see `index.html`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)] // "Per…" prefix is the whole point.
pub enum PriceUnitHint {
    /// Per discrete item (e.g. a bench, a fire pit).
    PerItem,
    /// Per square foot of surface (pavers, slabs).
    PerSquareFoot,
    /// Per cubic yard of loose material (mulch, gravel, sand).
    PerCubicYard,
    /// Per linear foot (edging, coping runs).
    PerLinearFoot,
}

/// One selectable option on a product configurator (a color, size, or texture).
#[derive(Debug, Clone, PartialEq, serde::Deserialize, JsonSchema)]
pub struct Variant {
    /// The option's label exactly as shown (e.g. "Shale Grey", "60 MM").
    pub name: String,
    /// Whether the option is selectable for the current configuration. Set
    /// `false` for options shown greyed-out / disabled on the page.
    #[serde(default = "yes")]
    #[schemars(default = "yes")]
    pub available: bool,
}

fn yes() -> bool {
    true
}

/// One size/format option, with its real dimensions — the geometry a catalog
/// item needs to tile and cost. A "size" is often a *bundle* of pieces; then
/// these are the most representative single unit.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, JsonSchema)]
pub struct SizeVariant {
    /// The size/format label exactly as shown (e.g. "60 MM", "6 × 13", "Grande").
    pub name: String,
    /// Whether the size is selectable for the current configuration (`false` for
    /// greyed-out / disabled options).
    #[serde(default = "yes")]
    #[schemars(default = "yes")]
    pub available: bool,
    /// A representative unit's east–west span in FEET (convert from inches:
    /// inches / 12). For a multi-piece size, the most representative single
    /// piece. Null if not shown.
    #[serde(default)]
    pub width_ft: Option<f64>,
    /// A representative unit's north–south span in FEET (inches / 12). Null if
    /// not shown.
    #[serde(default)]
    pub depth_ft: Option<f64>,
    /// The unit's thickness in inches — the installed depth (convert from
    /// millimetres if that's all that's shown: mm / 25.4). Null if not shown.
    #[serde(default)]
    pub thickness_in: Option<f64>,
}

/// A landscaping product read from a screenshot: the shared material fields plus
/// the full **variant matrix** (every color × texture × size the page offers).
/// The user multi-selects combos during curation (M4.2), each becoming one
/// catalog item — a color's look at a size's dimensions — to place and compare.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, JsonSchema)]
pub struct ExtractedProduct {
    /// The product's name exactly as shown on the page.
    pub name: String,
    /// The kind of hardscape, lowercase — e.g. "paver", "slab", "wall", "cap",
    /// "step", "edge". Null if unclear.
    #[serde(default)]
    pub category: Option<String>,
    /// How this material is priced. Pavers/slabs are `per_square_foot`; loose
    /// material (mulch, gravel) is `per_cubic_yard`; edging is `per_linear_foot`;
    /// discrete objects are `per_item`. Null if unclear.
    #[serde(default)]
    pub price_unit: Option<PriceUnitHint>,
    /// The unit price in dollars, ONLY if a price is clearly shown on the page.
    /// Manufacturer pages usually show NO price (you buy through a dealer) — in
    /// that case return null. NEVER guess or estimate a price.
    #[serde(default)]
    pub unit_price: Option<f64>,
    /// Every color option shown, greyed-out ones marked `available: false`.
    #[serde(default)]
    pub colors: Vec<Variant>,
    /// Every texture/finish option shown.
    #[serde(default)]
    pub textures: Vec<Variant>,
    /// Every size/format option shown, each with its dimensions. If the product
    /// has a single size, return one entry carrying its dimensions.
    #[serde(default)]
    pub sizes: Vec<SizeVariant>,
    /// Any caveat worth surfacing to the user (e.g. "no price listed on page").
    #[serde(default)]
    pub notes: Option<String>,
}

/// The JSON Schema for the extractor's tool input, as a string — derived from
/// [`ExtractedProduct`] so the schema and the type can never drift.
#[must_use]
pub fn tool_schema() -> String {
    serde_json::to_string(&schemars::schema_for!(ExtractedProduct)).unwrap_or_default()
}

/// Drop values a screenshot read shouldn't be trusted to have gotten right,
/// belt-and-suspenders behind the schema: non-positive/absurd dimensions and any
/// non-positive price (and we never surface a guessed price regardless).
fn sanitize(mut p: ExtractedProduct) -> ExtractedProduct {
    let dim = |o: Option<f64>| o.filter(|v| v.is_finite() && *v > 0.0 && *v < 1000.0);
    for s in &mut p.sizes {
        s.width_ft = dim(s.width_ft);
        s.depth_ft = dim(s.depth_ft);
        s.thickness_in = dim(s.thickness_in);
    }
    p.unit_price = p.unit_price.filter(|v| v.is_finite() && *v > 0.0);
    p
}

/// The instruction sent with the screenshot. The detailed rules live on the
/// schema (each field's description); this just points the model at the tool.
pub const EXTRACTION_PROMPT: &str = "\
This is a screenshot of a landscaping product page. Extract the product into \
the material catalog by calling the `extract_product` tool. Follow each field's \
description exactly — especially: capture EVERY color/texture/size option (mark \
greyed-out ones unavailable), convert dimensions to the requested units, and \
NEVER guess a price (return null when none is shown).";

/// Parse the model's text output into an [`ExtractedProduct`], tolerating a
/// ```json fenced block or surrounding whitespace.
///
/// # Errors
/// Returns a human-readable message when the text isn't parseable JSON of the
/// expected shape.
pub fn parse_extraction(text: &str) -> Result<ExtractedProduct, String> {
    let json = extract_json(text);
    serde_json::from_str::<ExtractedProduct>(json)
        .map_err(|e| format!("Couldn't read the extracted product: {e}"))
}

/// Pull the JSON body out of a model response: strip a ```json … ``` fence if
/// present, else take the first `{` … last `}` span, else the trimmed text.
fn extract_json(text: &str) -> &str {
    let t = text.trim();
    if let Some(rest) = t.strip_prefix("```") {
        // ```json\n{…}\n```  → drop the opening fence's language tag + closing.
        let body = rest.split_once('\n').map_or(rest, |(_, b)| b);
        return body.trim().strip_suffix("```").unwrap_or(body).trim();
    }
    match (t.find('{'), t.rfind('}')) {
        (Some(a), Some(b)) if b >= a => &t[a..=b],
        _ => t,
    }
}

/// Extract a draft product from `screenshot` (a `data:` URI) using the user's
/// `api_key` and `model`, via the `window.slpVision` bridge.
///
/// # Errors
/// Returns a human-readable message when the bridge is absent (non-browser /
/// no shell helper), the API call fails, or the response doesn't parse.
pub async fn extract(
    api_key: &str,
    model: &str,
    screenshot: &str,
) -> Result<ExtractedProduct, String> {
    let json = imp::extract(
        api_key,
        model,
        screenshot,
        EXTRACTION_PROMPT,
        &tool_schema(),
    )
    .await?;
    parse_extraction(&json).map(sanitize)
}

#[cfg(feature = "csr")]
mod imp {
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    fn slpvision() -> Option<JsValue> {
        let win = web_sys::window()?;
        let v = js_sys::Reflect::get(&win, &JsValue::from_str("slpVision")).ok()?;
        (!v.is_undefined() && !v.is_null()).then_some(v)
    }

    /// Call `window.slpVision.extract(apiKey, model, dataUri, prompt, schema)`,
    /// await it, and return the tool's structured `input` as a JSON string (or a
    /// human-readable error).
    pub async fn extract(
        api_key: &str,
        model: &str,
        screenshot: &str,
        prompt: &str,
        schema: &str,
    ) -> Result<String, String> {
        let v = slpvision().ok_or("Screenshot extraction isn't available here.")?;
        let f = js_sys::Reflect::get(&v, &JsValue::from_str("extract"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .ok_or("Screenshot extraction isn't available here.")?;
        let args = js_sys::Array::of5(
            &JsValue::from_str(api_key),
            &JsValue::from_str(model),
            &JsValue::from_str(screenshot),
            &JsValue::from_str(prompt),
            &JsValue::from_str(schema),
        );
        let promise = js_sys::Reflect::apply(&f, &v, &args)
            .map_err(|_| "Screenshot extraction failed to start.".to_string())?;
        let promise: js_sys::Promise = promise
            .dyn_into()
            .map_err(|_| "Screenshot extraction returned no result.".to_string())?;
        match JsFuture::from(promise).await {
            Ok(val) => val
                .as_string()
                .ok_or_else(|| "The extractor returned an empty response.".to_string()),
            Err(e) => Err(e
                .as_string()
                .or_else(|| {
                    js_sys::Reflect::get(&e, &JsValue::from_str("message"))
                        .ok()
                        .and_then(|m| m.as_string())
                })
                .unwrap_or_else(|| "Screenshot extraction failed.".to_string())),
        }
    }
}

#[cfg(not(feature = "csr"))]
mod imp {
    pub async fn extract(
        _api_key: &str,
        _model: &str,
        _screenshot: &str,
        _prompt: &str,
        _schema: &str,
    ) -> Result<String, String> {
        Err("Screenshot extraction is only available in the browser.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "name": "Blu 60 Slate Slabs",
        "category": "slab",
        "price_unit": "per_square_foot",
        "unit_price": null,
        "colors": [
            {"name": "Shale Grey", "available": true},
            {"name": "Onyx Black", "available": false}
        ],
        "textures": [{"name": "Slate", "available": true}],
        "sizes": [
            {"name": "60 MM", "width_ft": 1.083, "depth_ft": 1.083, "thickness_in": 2.375},
            {"name": "Grande"}
        ],
        "notes": "No price listed."
    }"#;

    #[test]
    fn parses_a_clean_json_product() {
        let p = parse_extraction(SAMPLE).expect("parses");
        assert_eq!(p.name, "Blu 60 Slate Slabs");
        assert_eq!(p.category.as_deref(), Some("slab"));
        assert_eq!(p.price_unit, Some(PriceUnitHint::PerSquareFoot));
        assert_eq!(p.unit_price, None, "no invented price");
        assert_eq!(p.colors.len(), 2);
        assert!(p.colors[0].available, "Shale Grey available");
        assert!(!p.colors[1].available, "Onyx Black unavailable");
        // A size carries its dimensions; a size with no `available` field
        // defaults to available.
        assert!(p.sizes[0].available, "sizes default to available");
        assert_eq!(p.sizes[0].width_ft, Some(1.083), "the size's width");
        assert_eq!(p.sizes[0].thickness_in, Some(2.375), "the size's thickness");
        assert_eq!(
            p.sizes[1].width_ft, None,
            "an unspecified size dim is absent"
        );
    }

    #[test]
    fn rejects_an_out_of_set_price_unit() {
        // The schema constrains price_unit to a closed enum; a free-form value
        // (what an unschematized model might return) fails to parse.
        let bad = SAMPLE.replace("per_square_foot", "sqft");
        assert!(
            parse_extraction(&bad).is_err(),
            "\"sqft\" is not in the enum"
        );
    }

    #[test]
    fn the_tool_schema_carries_descriptions_and_the_price_enum() {
        let schema = tool_schema();
        // Field-level guidance the model reads.
        assert!(
            schema.contains("NEVER guess"),
            "the price rule is in the schema"
        );
        assert!(
            schema.contains("in FEET"),
            "the unit-conversion rule is in the schema"
        );
        // The closed price-unit set.
        for id in [
            "per_item",
            "per_square_foot",
            "per_cubic_yard",
            "per_linear_foot",
        ] {
            assert!(schema.contains(id), "price_unit enum includes {id}");
        }
        assert!(schema.contains("\"name\""), "the required name field");
    }

    #[test]
    fn extract_guards_drop_untrusted_values() {
        // A non-positive price and an absurd dimension are dropped, never
        // surfaced into the estimate.
        let mut p = parse_extraction(SAMPLE).expect("parses");
        p.unit_price = Some(-5.0);
        p.sizes[0].thickness_in = Some(99999.0);
        let p = sanitize(p);
        assert_eq!(p.unit_price, None, "a non-positive price is dropped");
        assert_eq!(
            p.sizes[0].thickness_in, None,
            "an absurd dimension is dropped"
        );
        assert_eq!(p.sizes[0].width_ft, Some(1.083), "plausible dims survive");
    }

    #[test]
    fn tolerates_a_markdown_fenced_block() {
        let fenced = format!("Here is the product:\n```json\n{SAMPLE}\n```\n");
        let p = parse_extraction(&fenced).expect("parses through the fence");
        assert_eq!(p.name, "Blu 60 Slate Slabs");
    }

    #[test]
    fn tolerates_prose_around_a_bare_object() {
        let noisy = format!("Sure! {SAMPLE} Let me know if you need more.");
        let p = parse_extraction(&noisy).expect("parses the embedded object");
        assert_eq!(p.category.as_deref(), Some("slab"));
    }

    #[test]
    fn rejects_non_json() {
        assert!(parse_extraction("I couldn't read the image.").is_err());
    }
}

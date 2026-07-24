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
use slp_core::{CatalogItem, PriceUnit};

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

impl PriceUnitHint {
    /// The catalog's [`PriceUnit`] this hint maps to.
    #[must_use]
    pub fn to_price_unit(self) -> PriceUnit {
        match self {
            Self::PerItem => PriceUnit::per_item,
            Self::PerSquareFoot => PriceUnit::per_square_foot,
            Self::PerCubicYard => PriceUnit::per_cubic_yard,
            Self::PerLinearFoot => PriceUnit::per_linear_foot,
        }
    }
}

/// A bounding box within a pasted screenshot, as fractions of its size (each
/// 0–1), plus which screenshot it's on.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, JsonSchema)]
pub struct BBox {
    /// Which pasted screenshot this box is on: a 0-based index into the images,
    /// in the order they were pasted. `0` when there is a single screenshot.
    #[serde(default)]
    #[schemars(default)]
    pub image: usize,
    /// Left edge, as a fraction of the image width (0 = left, 1 = right).
    pub x: f64,
    /// Top edge, as a fraction of the image height (0 = top, 1 = bottom).
    pub y: f64,
    /// Width, as a fraction of the image width.
    pub width: f64,
    /// Height, as a fraction of the image height.
    pub height: f64,
}

/// One selectable option on a product configurator (a color or texture).
#[derive(Debug, Clone, PartialEq, serde::Deserialize, JsonSchema)]
pub struct Variant {
    /// The option's label exactly as shown (e.g. "Shale Grey", "Slate").
    pub name: String,
    /// Whether the option is selectable for the current configuration. Set
    /// `false` for options shown greyed-out / disabled on the page.
    #[serde(default = "yes")]
    #[schemars(default = "yes")]
    pub available: bool,
    /// The bounding box of THIS option's swatch thumbnail within the screenshot,
    /// so its image can be cropped out. Give the tightest box around just the
    /// swatch tile (exclude the text label). Null if not locatable.
    #[serde(default)]
    pub bbox: Option<BBox>,
    /// The option's swatch image as a `data:` URI, cropped from the screenshot
    /// at `bbox`. Populated client-side, never by the model — so it's skipped
    /// from the tool schema.
    #[serde(default, skip)]
    #[schemars(skip)]
    pub swatch: Option<String>,
}

fn yes() -> bool {
    true
}

/// One purchasable **format** of a paver/slab (e.g. "60 MM", "6 × 13",
/// "Grande") — the granularity the user buys and lays as a unit. A format is
/// often a *system* of several piece sizes laid in a pattern; keep it as ONE
/// format (do not split it into pieces) and record the included pieces in
/// `includes`. Its tile dimensions are the real-world repeat of the installed
/// pattern, used to tile the color swatch photo to scale.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, JsonSchema)]
pub struct SizeVariant {
    /// The format label exactly as shown (e.g. "60 MM", "6 × 13", "Grande").
    pub name: String,
    /// Whether the format is selectable for the current configuration (`false`
    /// for greyed-out / disabled options).
    #[serde(default = "yes")]
    #[schemars(default = "yes")]
    pub available: bool,
    /// The installed pattern's repeat WIDTH in FEET (inches / 12) — for a
    /// single-piece format, the piece width; for a multi-piece format, a
    /// representative module width. Null if unclear.
    #[serde(default)]
    pub width_ft: Option<f64>,
    /// The installed pattern's repeat DEPTH/length in FEET (inches / 12). Null
    /// if unclear.
    #[serde(default)]
    pub depth_ft: Option<f64>,
    /// The unit thickness in inches (a "60 MM" label means 60 mm ≈ 2.36 in).
    #[serde(default)]
    pub thickness_in: Option<f64>,
    /// For a multi-piece format, the included piece sizes exactly as shown —
    /// e.g. "A: 6½×13, B: 13×13, C: 19½×13 in" — metadata for a future coverage
    /// calc. Null for a single-piece format.
    #[serde(default)]
    pub includes: Option<String>,
}

/// One **laying pattern** the product docs publish for a paver/slab format —
/// herringbone, parquet, linear, random … — its label plus, when locatable,
/// the bounding box of its diagram thumbnail so the app can crop it out.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, JsonSchema)]
pub struct Pattern {
    /// The pattern's label exactly as shown (e.g. "Herringbone", "Random").
    pub name: String,
    /// The bounding box of THIS pattern's diagram within its screenshot (set
    /// the box's `image` to that screenshot's index). Give the tightest box
    /// around just the diagram (exclude the text label). Null if not locatable.
    #[serde(default)]
    pub bbox: Option<BBox>,
    /// The diagram cropped from the screenshot as a `data:` URI. Populated
    /// client-side, never by the model — skipped from the tool schema.
    #[serde(default, skip)]
    #[schemars(skip)]
    pub diagram: Option<String>,
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
    /// Every purchasable size **format** shown (e.g. "60 MM", "6 × 13",
    /// "Grande"). Keep each as ONE format — do NOT split a multi-piece format
    /// (a "SIZES INCLUDED: A …, B …, C …" block) into separate pieces; record
    /// those pieces in the format's `includes` instead. If only one format is
    /// shown, return that one.
    #[serde(default)]
    pub sizes: Vec<SizeVariant>,
    /// Every **laying pattern** the docs show for this product (herringbone,
    /// parquet, linear …), typically on a dedicated laying-patterns screenshot.
    /// Empty if none are shown.
    #[serde(default)]
    pub patterns: Vec<Pattern>,
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

/// Turn a curated draft into catalog items — the cross product of the selected
/// colors and sizes (each `(color, size)` becomes one [`CatalogItem`]), sharing
/// the edited `category`/`unit_price`/`price_unit`. A size carries the tile
/// geometry; a color carries the swatch image (attached in B4). The selected
/// laying `patterns` ride **every** item (format-level — each color combo of a
/// product supports the same layouts). An empty color/size selection
/// contributes a single unnamed axis, so a product with no colors or no sizes
/// still yields items.
#[must_use]
pub fn to_catalog_items(
    product: &ExtractedProduct,
    colors: &[usize],
    sizes: &[usize],
    patterns: &[usize],
    category: &str,
    unit_price: Option<f64>,
    price_unit: &PriceUnit,
) -> Vec<CatalogItem> {
    let laying: Vec<slp_core::LayingPattern> = patterns
        .iter()
        .filter_map(|&i| product.patterns.get(i))
        .map(|p| {
            let mut lp = slp_core::LayingPattern::new(p.name.clone());
            lp.diagram.clone_from(&p.diagram);
            lp
        })
        .collect();
    let color_opts: Vec<Option<&Variant>> = if colors.is_empty() {
        vec![None]
    } else {
        colors
            .iter()
            .filter_map(|&i| product.colors.get(i))
            .map(Some)
            .collect()
    };
    let size_opts: Vec<Option<&SizeVariant>> = if sizes.is_empty() {
        vec![None]
    } else {
        sizes
            .iter()
            .filter_map(|&i| product.sizes.get(i))
            .map(Some)
            .collect()
    };
    let category = category.trim();
    let mut items = Vec::new();
    for color in &color_opts {
        for size in &size_opts {
            let mut parts = vec![product.name.clone()];
            if let Some(c) = color {
                parts.push(c.name.clone());
            }
            if let Some(s) = size {
                parts.push(s.name.clone());
            }
            let name = parts.join(" — ");
            let mut item = CatalogItem::new(slug(&name));
            item.name = Some(name);
            if !category.is_empty() {
                item.category = Some(category.to_string());
            }
            item.unit_price = unit_price;
            item.price_unit.clone_from(price_unit);
            if let Some(s) = size {
                item.tile_width_ft = s.width_ft;
                item.tile_depth_ft = s.depth_ft;
            }
            if let Some(c) = color {
                item.image.clone_from(&c.swatch);
            }
            item.patterns.clone_from(&laying);
            items.push(item);
        }
    }
    items
}

/// A filesystem/id-safe slug: lowercase, runs of non-alphanumerics collapsed to
/// single dashes, trimmed.
fn slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending_dash = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

/// The instruction sent with the screenshot. The detailed rules live on the
/// schema (each field's description); this just points the model at the tool.
pub const EXTRACTION_PROMPT: &str = "\
These are screenshots of ONE landscaping product page (colors, sizes, and/or \
laying patterns may be split across several images, in the order given). Extract \
the single product into the material catalog by calling the `extract_product` \
tool, merging what all the images show. Follow each field's description exactly \
— especially: capture EVERY color/texture option (mark greyed-out ones \
unavailable); for sizes, list each purchasable FORMAT as shown (e.g. '60 MM', \
'6 × 13', 'Grande') as ONE entry each — do NOT split a multi-piece format into \
separate pieces; record its included pieces in `includes` and give the format's \
tile dimensions as the installed pattern's repeat; capture every LAYING PATTERN \
shown (herringbone, parquet, linear, …) with its diagram's bounding box; for \
every bounding box set `image` to the 0-based index of the screenshot it's on; \
convert dimensions to the requested units; and NEVER guess a price (return null \
when none is shown).";

/// Crop `bbox` out of `screenshot` (a `data:` URI), returning the region as a
/// `data:` URI — used to re-crop a swatch when the user adjusts the box (B5).
/// `None` off the browser or on any failure.
pub async fn crop(screenshot: &str, bbox: BBox) -> Option<String> {
    imp::crop(screenshot, bbox).await
}

/// The price basis to assume when the page shows none (the manufacturer norm),
/// from the extracted category: surfaces (pavers, slabs, tile) sell per ft²,
/// loose material per yd³, edging/coping runs per linear ft, everything else
/// per item. Keeps a slab from landing in the catalog as a per-item object —
/// which would hide it from the Area tool's material picker.
#[must_use]
pub fn default_price_unit(category: Option<&str>) -> PriceUnit {
    let cat = category.unwrap_or_default().to_ascii_lowercase();
    if ["paver", "slab", "tile", "flagstone"]
        .iter()
        .any(|k| cat.contains(k))
    {
        PriceUnit::per_square_foot
    } else if ["mulch", "gravel", "sand", "soil", "aggregate"]
        .iter()
        .any(|k| cat.contains(k))
    {
        PriceUnit::per_cubic_yard
    } else if ["edg", "coping", "border"].iter().any(|k| cat.contains(k)) {
        PriceUnit::per_linear_foot
    } else {
        PriceUnit::per_item
    }
}

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

/// Extract a draft product from one or more `screenshots` (each a `data:` URI —
/// a product's page is often several screenshots: colors, sizes, laying
/// patterns) using the user's `api_key` and `model`, via the `window.slpVision`
/// bridge. All images go in one request, tagged so a bounding box knows which
/// screenshot it's on.
///
/// # Errors
/// Returns a human-readable message when the bridge is absent (non-browser /
/// no shell helper), the API call fails, or the response doesn't parse.
pub async fn extract(
    api_key: &str,
    model: &str,
    screenshots: &[String],
) -> Result<ExtractedProduct, String> {
    let json = imp::extract(
        api_key,
        model,
        screenshots,
        EXTRACTION_PROMPT,
        &tool_schema(),
    )
    .await?;
    let mut product = parse_extraction(&json).map(sanitize)?;
    // Best-effort: crop each color's swatch — and each laying pattern's
    // diagram — out of the screenshot its box names.
    for color in &mut product.colors {
        if let Some(bbox) = color.bbox
            && let Some(shot) = screenshots.get(bbox.image.min(screenshots.len().saturating_sub(1)))
        {
            color.swatch = imp::crop(shot, bbox).await;
        }
    }
    for pat in &mut product.patterns {
        if let Some(bbox) = pat.bbox
            && let Some(shot) = screenshots.get(bbox.image.min(screenshots.len().saturating_sub(1)))
        {
            pat.diagram = imp::crop(shot, bbox).await;
        }
    }
    Ok(product)
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

    /// Call `window.slpVision.extract(apiKey, model, dataUris, prompt, schema)`
    /// with the pasted screenshots as an array, await it, and return the tool's
    /// structured `input` as a JSON string (or a human-readable error).
    pub async fn extract(
        api_key: &str,
        model: &str,
        screenshots: &[String],
        prompt: &str,
        schema: &str,
    ) -> Result<String, String> {
        let v = slpvision().ok_or("Screenshot extraction isn't available here.")?;
        let f = js_sys::Reflect::get(&v, &JsValue::from_str("extract"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .ok_or("Screenshot extraction isn't available here.")?;
        let uris = js_sys::Array::new();
        for s in screenshots {
            uris.push(&JsValue::from_str(s));
        }
        let args = js_sys::Array::of5(
            &JsValue::from_str(api_key),
            &JsValue::from_str(model),
            uris.as_ref(),
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

    /// Crop `bbox` out of `screenshot` via `window.slpVision.crop`, returning the
    /// cropped region as a `data:` URI (or `None` on any failure).
    pub async fn crop(screenshot: &str, bbox: super::BBox) -> Option<String> {
        let v = slpvision()?;
        let f = js_sys::Reflect::get(&v, &JsValue::from_str("crop"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())?;
        let args = js_sys::Array::of5(
            &JsValue::from_str(screenshot),
            &JsValue::from_f64(bbox.x),
            &JsValue::from_f64(bbox.y),
            &JsValue::from_f64(bbox.width),
            &JsValue::from_f64(bbox.height),
        );
        let promise: js_sys::Promise = js_sys::Reflect::apply(&f, &v, &args)
            .ok()?
            .dyn_into()
            .ok()?;
        JsFuture::from(promise).await.ok()?.as_string()
    }
}

#[cfg(not(feature = "csr"))]
mod imp {
    pub async fn extract(
        _api_key: &str,
        _model: &str,
        _screenshots: &[String],
        _prompt: &str,
        _schema: &str,
    ) -> Result<String, String> {
        Err("Screenshot extraction is only available in the browser.".to_string())
    }

    pub async fn crop(_screenshot: &str, _bbox: super::BBox) -> Option<String> {
        None
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
            {"name": "Shale Grey", "available": true,
             "bbox": {"x": 0.1, "y": 0.2, "width": 0.12, "height": 0.12}},
            {"name": "Onyx Black", "available": false}
        ],
        "textures": [{"name": "Slate", "available": true}],
        "sizes": [
            {"name": "60 MM", "width_ft": 1.083, "depth_ft": 1.083, "thickness_in": 2.375,
             "includes": "A: 6½×13, B: 13×13, C: 19½×13 in"},
            {"name": "Grande"}
        ],
        "patterns": [
            {"name": "Herringbone",
             "bbox": {"image": 1, "x": 0.05, "y": 0.1, "width": 0.2, "height": 0.2}},
            {"name": "Linear"}
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
        let bbox = p.colors[0].bbox.expect("a color carries its swatch bbox");
        assert!((bbox.x - 0.1).abs() < 1e-9 && (bbox.width - 0.12).abs() < 1e-9);
        // A size carries its dimensions; a size with no `available` field
        // defaults to available.
        assert!(p.sizes[0].available, "formats default to available");
        assert_eq!(p.sizes[0].width_ft, Some(1.083), "the format's tile width");
        assert_eq!(
            p.sizes[0].thickness_in,
            Some(2.375),
            "the format's thickness"
        );
        assert_eq!(
            p.sizes[0].includes.as_deref(),
            Some("A: 6½×13, B: 13×13, C: 19½×13 in"),
            "a multi-piece format keeps its pieces as metadata, not split out"
        );
        assert_eq!(
            p.sizes[1].width_ft, None,
            "an unspecified format dim is absent"
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

    #[test]
    fn curation_makes_one_item_per_color_size_combo() {
        let p = parse_extraction(SAMPLE).expect("parses"); // 2 colors, 2 sizes
        // Pick both colors × the first size → 2 items.
        let items = to_catalog_items(
            &p,
            &[0, 1],
            &[0],
            &[],
            "slab",
            Some(12.5),
            &PriceUnit::per_square_foot,
        );
        assert_eq!(items.len(), 2, "one item per color × size");

        let first = &items[0];
        assert_eq!(
            first.name.as_deref(),
            Some("Blu 60 Slate Slabs — Shale Grey — 60 MM")
        );
        assert_eq!(
            first.id, "blu-60-slate-slabs-shale-grey-60-mm",
            "slugged id"
        );
        assert_eq!(first.category.as_deref(), Some("slab"));
        assert_eq!(first.unit_price, Some(12.5), "the edited price is applied");
        assert_eq!(first.price_unit, PriceUnit::per_square_foot);
        // The size's dimensions become the tile geometry.
        assert_eq!(first.tile_width_ft, Some(1.083));
        assert_eq!(first.tile_depth_ft, Some(1.083));
        // Distinct ids per combo.
        assert_ne!(items[0].id, items[1].id);
    }

    #[test]
    fn curation_with_no_sizes_yields_one_item_per_color() {
        let p = parse_extraction(SAMPLE).expect("parses");
        let items = to_catalog_items(&p, &[0], &[], &[], "", None, &PriceUnit::per_item);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].name.as_deref(),
            Some("Blu 60 Slate Slabs — Shale Grey")
        );
        assert_eq!(items[0].category, None, "a blank category stays unset");
        assert_eq!(items[0].tile_width_ft, None, "no size → no tile geometry");
        assert!(
            items[0].patterns.is_empty(),
            "no patterns selected → none ride"
        );
    }

    #[test]
    fn parses_and_attaches_laying_patterns() {
        // The fixture's patterns parse — Herringbone's diagram box (on image 1)
        // included — and the selected ones ride EVERY approved item.
        let mut p = parse_extraction(SAMPLE).expect("parses");
        assert_eq!(p.patterns.len(), 2);
        assert_eq!(p.patterns[0].name, "Herringbone");
        let bb = p.patterns[0]
            .bbox
            .expect("herringbone carries a diagram box");
        assert_eq!(bb.image, 1, "the box names its screenshot");
        assert_eq!(p.patterns[1].bbox, None, "linear has no box");
        // A client-side cropped diagram rides through to the catalog items.
        p.patterns[0].diagram = Some("data:image/png;base64,DIAG".to_string());
        let items = to_catalog_items(
            &p,
            &[0, 1],
            &[],
            &[0, 1],
            "slab",
            None,
            &PriceUnit::per_square_foot,
        );
        assert_eq!(items.len(), 2, "patterns don't multiply items");
        for item in &items {
            assert_eq!(item.patterns.len(), 2, "every combo carries the list");
            assert_eq!(item.patterns[0].name, "Herringbone");
            assert_eq!(
                item.patterns[0].diagram.as_deref(),
                Some("data:image/png;base64,DIAG")
            );
            assert_eq!(item.patterns[1].name, "Linear");
            assert_eq!(item.patterns[1].diagram, None);
        }
        // Selecting only one pattern drops the other.
        let items = to_catalog_items(&p, &[0], &[], &[1], "", None, &PriceUnit::per_item);
        assert_eq!(items[0].patterns.len(), 1);
        assert_eq!(items[0].patterns[0].name, "Linear");
    }

    #[test]
    fn a_missing_price_basis_defaults_from_the_category() {
        // Surfaces sell per ft², loose material per yd³, edging per linear ft;
        // anything unrecognized (or uncategorized) stays per item.
        assert_eq!(default_price_unit(Some("slab")), PriceUnit::per_square_foot);
        assert_eq!(
            default_price_unit(Some("Paver")),
            PriceUnit::per_square_foot,
            "case-insensitive"
        );
        assert_eq!(default_price_unit(Some("mulch")), PriceUnit::per_cubic_yard);
        assert_eq!(
            default_price_unit(Some("edging")),
            PriceUnit::per_linear_foot
        );
        assert_eq!(default_price_unit(Some("furniture")), PriceUnit::per_item);
        assert_eq!(default_price_unit(None), PriceUnit::per_item);
    }

    #[test]
    fn price_unit_hint_maps_to_the_catalog_unit() {
        assert_eq!(
            PriceUnitHint::PerCubicYard.to_price_unit(),
            PriceUnit::per_cubic_yard
        );
        assert_eq!(PriceUnitHint::PerItem.to_price_unit(), PriceUnit::per_item);
    }
}

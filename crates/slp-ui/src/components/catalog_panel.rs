//! The catalog inspector: a panel to browse and edit the plan's catalog — the
//! catalog-side counterpart of [`ObjectInspector`](super::ObjectInspector),
//! which edits a *placed* object. It lists every catalog item (starter,
//! hand-added, or ingested), and for the selected one lets you edit its
//! name/category/price and footprint dimensions. Edits are live: an object
//! references its catalog item by `catalog_ref` (not a copy), so changing an
//! item's price or size reprices and re-renders every object placed from it.

use leptos::prelude::*;
use slp_core::{CatalogItem, PriceUnit};

use super::{FileInput, MaterialSwatch, NumberField, SelectField, TextField, Toggle};
use crate::vision::{ExtractedProduct, SizeVariant, Variant};

/// The `price_unit` id an area material / object is costed by — the string the
/// `SelectField` round-trips.
fn price_unit_id(unit: &PriceUnit) -> &'static str {
    match unit {
        PriceUnit::per_square_foot => "per_square_foot",
        PriceUnit::per_cubic_yard => "per_cubic_yard",
        PriceUnit::per_linear_foot => "per_linear_foot",
        // per_item, and any future variant, id as per-item.
        _ => "per_item",
    }
}

/// Parse a `price_unit` id back to its enum (unknown → per-item).
fn price_unit_from_id(id: &str) -> PriceUnit {
    match id {
        "per_square_foot" => PriceUnit::per_square_foot,
        "per_cubic_yard" => PriceUnit::per_cubic_yard,
        "per_linear_foot" => PriceUnit::per_linear_foot,
        _ => PriceUnit::per_item,
    }
}

/// The `price_unit` choices, `(id, label)`, for the editor's dropdown.
fn price_unit_options() -> Vec<(String, String)> {
    vec![
        ("per_item".to_string(), "Per item".to_string()),
        ("per_square_foot".to_string(), "Per ft²".to_string()),
        ("per_cubic_yard".to_string(), "Per yd³".to_string()),
        ("per_linear_foot".to_string(), "Per linear ft".to_string()),
    ]
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
#[component]
pub fn CatalogPanel(
    /// Every item in the plan's catalog.
    #[prop(into)]
    catalog: Signal<Vec<CatalogItem>>,
    /// The id of the catalog item being edited, if any.
    #[prop(into)]
    selected: Signal<Option<String>>,
    /// Select the item with this id for editing.
    on_select: Callback<String>,
    /// Set the selected item's display name.
    on_name: Callback<String>,
    /// Set the selected item's category.
    on_category: Callback<String>,
    /// Set the selected item's unit price (dollars).
    on_price: Callback<f64>,
    /// Set the selected item's price unit (per item / ft² / yd³ / linear ft).
    on_price_unit: Callback<PriceUnit>,
    /// Add a new (blank) catalog item and select it for editing.
    on_add: Callback<()>,
    /// Set whether the selected item is a sub-base aggregate (a course material).
    #[prop(default = Callback::new(|_: bool| {}))]
    on_aggregate: Callback<bool>,
    /// Set the selected item's image (a `data:` URI or URL).
    #[prop(default = Callback::new(|_: String| {}))]
    on_image: Callback<String>,
    /// Set the image tile's real-world east–west span (ft).
    #[prop(default = Callback::new(|_: f64| {}))]
    on_tile_width: Callback<f64>,
    /// Set the image tile's real-world north–south span (ft).
    #[prop(default = Callback::new(|_: f64| {}))]
    on_tile_depth: Callback<f64>,
    /// Set the selected item's footprint width / diameter (ft).
    on_width: Callback<f64>,
    /// Set the selected item's footprint depth (ft).
    on_depth: Callback<f64>,
    /// Set the selected item's height (ft).
    on_height: Callback<f64>,
    /// The current Anthropic API key for screenshot ingestion; empty when
    /// unset. App/browser config only — never persisted in the plan.
    #[prop(into, default = Signal::derive(String::new))]
    api_key: Signal<String>,
    /// Persist the API key (an empty value clears it). Defaults to a no-op so
    /// callers that don't wire ingestion still compile.
    #[prop(default = Callback::new(|_: String| {}))]
    on_api_key: Callback<String>,
    /// The pasted product screenshot as a `data:` URI (empty when none) — the
    /// input the vision extractor will read.
    #[prop(into, default = Signal::derive(String::new))]
    screenshot: Signal<String>,
    /// Set the pasted screenshot (a `data:` URI); an empty value clears it.
    #[prop(default = Callback::new(|_: String| {}))]
    on_screenshot: Callback<String>,
    /// The vision model id used for extraction (editable).
    #[prop(into, default = Signal::derive(|| crate::vision::DEFAULT_MODEL.to_string()))]
    model: Signal<String>,
    /// Set the vision model id.
    #[prop(default = Callback::new(|_: String| {}))]
    on_model: Callback<String>,
    /// Run vision extraction on the pasted screenshot.
    #[prop(default = Callback::new(|(): ()| {}))]
    on_extract: Callback<()>,
    /// Whether an extraction is currently in flight.
    #[prop(into, default = Signal::derive(|| false))]
    extracting: Signal<bool>,
    /// The last extraction error, if any.
    #[prop(into, default = Signal::derive(|| None::<String>))]
    extract_error: Signal<Option<String>>,
    /// The extracted draft product awaiting curation, if any.
    #[prop(into, default = Signal::derive(|| None::<ExtractedProduct>))]
    draft: Signal<Option<ExtractedProduct>>,
    /// Close the catalog panel.
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <aside class="catalog-panel" data-testid="catalog-panel">
            <header class="catalog-panel-head">
                <h2>"Catalog"</h2>
                <div class="catalog-head-actions">
                    <button
                        class="catalog-add"
                        data-testid="catalog-add"
                        on:click=move |_| on_add.run(())
                    >
                        "+ Add"
                    </button>
                    <button
                        class="catalog-close"
                        data-testid="catalog-close"
                        on:click=move |_| on_close.run(())
                    >
                        "Close"
                    </button>
                </div>
            </header>
            // Screenshot ingestion (Phase B): the API key that gates the
            // vision "extract from screenshot" flow. Stored as app config, not
            // in the plan (see `api_key` module).
            <section class="ingest-section" data-testid="ingest-section">
                <h3 class="ingest-title">"Screenshot ingestion"</h3>
                <TextField
                    label="Anthropic API key"
                    testid="ingest-api-key"
                    input_type="password"
                    value=api_key
                    placeholder="sk-ant-…"
                    on_input=on_api_key
                />
                {move || {
                    if api_key.get().trim().is_empty() {
                        view! {
                            <p
                                class="ingest-status ingest-status--off"
                                data-testid="ingest-status"
                            >
                                "Add your Anthropic API key to enable screenshot ingestion."
                            </p>
                        }
                            .into_any()
                    } else {
                        view! {
                            <p
                                class="ingest-status ingest-status--on"
                                data-testid="ingest-status"
                            >
                                "Screenshot ingestion enabled."
                            </p>
                            // Paste a product screenshot (⌘⇧4 → ⌘V) → a data URI
                            // the vision extractor (B3) will read.
                            <div
                                class="ingest-paste"
                                data-testid="ingest-paste"
                                tabindex="0"
                                on:paste=move |ev| read_pasted_image(&ev, on_screenshot)
                            >
                                "Click here, then paste a product screenshot (⌘V)."
                            </div>
                            {move || {
                                let shot = screenshot.get();
                                (!shot.is_empty())
                                    .then(|| {
                                        view! {
                                            <div class="ingest-shot">
                                                <img
                                                    class="ingest-screenshot"
                                                    data-testid="ingest-screenshot"
                                                    src=shot.clone()
                                                    alt="pasted screenshot"
                                                />
                                                <button
                                                    class="ingest-clear"
                                                    data-testid="ingest-clear"
                                                    on:click=move |_| on_screenshot.run(String::new())
                                                >
                                                    "Clear"
                                                </button>
                                            </div>
                                        }
                                    })
                            }}
                            // Once a screenshot is pasted: pick a model and run
                            // the vision extraction.
                            {move || {
                                (!screenshot.get().is_empty())
                                    .then(|| {
                                        view! {
                                            <TextField
                                                label="Model"
                                                testid="ingest-model"
                                                value=model
                                                on_input=on_model
                                            />
                                            <button
                                                class="ingest-extract"
                                                data-testid="ingest-extract"
                                                disabled=move || extracting.get()
                                                on:click=move |_| on_extract.run(())
                                            >
                                                {move || {
                                                    if extracting.get() {
                                                        "Extracting…"
                                                    } else {
                                                        "Extract details"
                                                    }
                                                }}
                                            </button>
                                        }
                                    })
                            }}
                            {move || {
                                extract_error
                                    .get()
                                    .map(|e| {
                                        view! {
                                            <p class="ingest-error" data-testid="ingest-error">
                                                {e}
                                            </p>
                                        }
                                    })
                            }}
                            {move || draft.get().map(|d| draft_view(&d))}
                        }
                            .into_any()
                    }
                }}
            </section>
            <div class="catalog-list">
                {move || {
                    let sel = selected.get();
                    catalog
                        .get()
                        .into_iter()
                        .map(|item| row(item, sel.clone(), on_select))
                        .collect::<Vec<_>>()
                }}
            </div>
            {move || {
                let sel = selected.get()?;
                let item = catalog.get().into_iter().find(|c| c.id == sel)?;
                Some(
                    view! {
                        <div class="catalog-editor" data-testid="catalog-editor">
                            <TextField
                                label="Name"
                                testid="catalog-name"
                                value=item.name.clone().unwrap_or_default()
                                on_input=on_name
                            />
                            <TextField
                                label="Category"
                                testid="catalog-category"
                                value=item.category.clone().unwrap_or_default()
                                placeholder="e.g. furniture"
                                on_input=on_category
                            />
                            <NumberField
                                label="Price ($)"
                                testid="catalog-price"
                                value=item.unit_price.unwrap_or(0.0)
                                step=1.0
                                min=0.0
                                on_input=on_price
                            />
                            <SelectField
                                label="Priced"
                                testid="catalog-price-unit"
                                value=price_unit_id(&item.price_unit).to_string()
                                options=price_unit_options()
                                on_change=Callback::new(move |id: String| {
                                    on_price_unit.run(price_unit_from_id(&id));
                                })
                            />
                            // A bulk (per-yd³) material can be marked a sub-base
                            // aggregate, making it selectable as a paver course.
                            {(item.price_unit == PriceUnit::per_cubic_yard)
                                .then(|| {
                                    view! {
                                        <Toggle
                                            label="Sub-base aggregate"
                                            testid="catalog-aggregate"
                                            checked=item.is_aggregate == Some(true)
                                            on_toggle=on_aggregate
                                        />
                                    }
                                })}
                            <NumberField
                                label="Width (ft)"
                                testid="catalog-width"
                                value=item.width_ft.unwrap_or(0.0)
                                step=0.5
                                min=0.0
                                on_input=on_width
                            />
                            <NumberField
                                label="Depth (ft)"
                                testid="catalog-depth"
                                value=item.depth_ft.unwrap_or(0.0)
                                step=0.5
                                min=0.0
                                on_input=on_depth
                            />
                            <NumberField
                                label="Height (ft)"
                                testid="catalog-height"
                                value=item.height_ft.unwrap_or(0.0)
                                step=0.5
                                min=0.0
                                on_input=on_height
                            />
                            <TextField
                                label="Image"
                                testid="catalog-image"
                                value=item.image.clone().unwrap_or_default()
                                placeholder="image URL or data URI"
                                on_input=on_image
                            />
                            // …or upload a file (read to a data URI).
                            <FileInput
                                label="Upload"
                                testid="catalog-image-file"
                                accept="image/*"
                                on_file=on_image
                            />
                            <NumberField
                                label="Tile W (ft)"
                                testid="catalog-tile-width"
                                value=item.tile_width_ft.unwrap_or(0.0)
                                step=0.5
                                min=0.0
                                on_input=on_tile_width
                            />
                            <NumberField
                                label="Tile D (ft)"
                                testid="catalog-tile-depth"
                                value=item.tile_depth_ft.unwrap_or(0.0)
                                step=0.5
                                min=0.0
                                on_input=on_tile_depth
                            />
                            // A live preview of the material photo, when set.
                            {item
                                .image
                                .clone()
                                .filter(|s| !s.is_empty())
                                .map(|src| {
                                    view! {
                                        <img
                                            class="catalog-image-preview"
                                            data-testid="catalog-image-preview"
                                            src=src
                                            alt="material preview"
                                        />
                                    }
                                })}
                        </div>
                    },
                )
            }}
        </aside>
    }
}

/// Read the first image on the clipboard from a `paste` event to a `data:` URI
/// and hand it to `on_image`. Browser-only — a no-op when not compiled for
/// `csr` (mirrors [`FileInput`](super::FileInput)'s file read).
#[cfg(feature = "csr")]
fn read_pasted_image(ev: &leptos::ev::Event, on_image: Callback<String>) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    use web_sys::{ClipboardEvent, FileReader};

    let Some(items) = ev
        .dyn_ref::<ClipboardEvent>()
        .and_then(ClipboardEvent::clipboard_data)
        .map(|dt| dt.items())
    else {
        return;
    };
    // The clipboard can hold several representations; take the first image.
    let mut file = None;
    for i in 0..items.length() {
        if let Some(item) = items.get(i)
            && item.type_().starts_with("image/")
            && let Ok(Some(f)) = item.get_as_file()
        {
            file = Some(f);
            break;
        }
    }
    let Some(file) = file else {
        return;
    };
    let Ok(reader) = FileReader::new() else {
        return;
    };
    let reader_for_load = reader.clone();
    let onload = Closure::<dyn FnMut()>::new(move || {
        if let Some(url) = reader_for_load.result().ok().and_then(|v| v.as_string()) {
            on_image.run(url);
        }
    });
    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    let _ = reader.read_as_data_url(&file);
}

#[cfg(not(feature = "csr"))]
fn read_pasted_image(_ev: &leptos::ev::Event, _on_image: Callback<String>) {}

/// A read-only summary of an extracted draft product: name, a metadata line
/// (category · price-or-"no price"), and the variant lists with unavailable
/// options dimmed. Multi-select curation into catalog items is M4.2.
fn draft_view(d: &ExtractedProduct) -> impl IntoView + use<> {
    let cat = d
        .category
        .clone()
        .unwrap_or_else(|| "uncategorized".to_string());
    let price = d
        .unit_price
        .map_or_else(|| "no price listed".to_string(), |p| format!("${p:.2}"));
    let meta = format!("{cat} · {price}");
    let notes = d.notes.clone();
    view! {
        <div class="ingest-draft" data-testid="ingest-draft">
            <h4 class="ingest-draft-name">{d.name.clone()}</h4>
            <p class="ingest-draft-meta">{meta}</p>
            {variant_list("Colors", &d.colors)}
            {variant_list("Textures", &d.textures)}
            {size_list(&d.sizes)}
            {notes.map(|n| view! { <p class="ingest-draft-notes">{n}</p> })}
        </div>
    }
}

/// One labeled variant group (Colors / Sizes / Textures) as a list, with
/// unavailable options dimmed; nothing when the group is empty.
fn variant_list(label: &'static str, variants: &[Variant]) -> Option<impl IntoView + use<>> {
    (!variants.is_empty()).then(|| {
        let items = variants
            .iter()
            .map(|v| {
                let name = v.name.clone();
                view! {
                    <li class="ingest-variant" class:unavailable=!v.available>
                        {name}
                    </li>
                }
            })
            .collect::<Vec<_>>();
        view! {
            <div class="ingest-draft-group">
                <span class="ingest-draft-label">{label}</span>
                <ul class="ingest-variant-list">{items}</ul>
            </div>
        }
    })
}

/// The Sizes group: each size with its dimensions (`w×d ft · t in` when known),
/// unavailable ones dimmed; nothing when there are no sizes.
fn size_list(sizes: &[SizeVariant]) -> Option<impl IntoView + use<>> {
    (!sizes.is_empty()).then(|| {
        let items = sizes
            .iter()
            .map(|s| {
                let dims = match (s.width_ft, s.depth_ft) {
                    (Some(w), Some(d)) => format!(" — {w:.2}×{d:.2} ft"),
                    _ => String::new(),
                };
                let thick = s
                    .thickness_in
                    .map_or_else(String::new, |t| format!(" · {t:.2} in"));
                let label = format!("{}{dims}{thick}", s.name);
                view! {
                    <li class="ingest-variant" class:unavailable=!s.available>
                        {label}
                    </li>
                }
            })
            .collect::<Vec<_>>();
        view! {
            <div class="ingest-draft-group">
                <span class="ingest-draft-label">"Sizes"</span>
                <ul class="ingest-variant-list">{items}</ul>
            </div>
        }
    })
}

/// One catalog item as a selectable list row: its name and price, highlighted
/// when it's the item being edited.
#[allow(clippy::needless_pass_by_value)]
fn row(item: CatalogItem, selected: Option<String>, on_select: Callback<String>) -> impl IntoView {
    let id = item.id.clone();
    let is_selected = selected.as_deref() == Some(id.as_str());
    let name = item.name.clone().unwrap_or_else(|| id.clone());
    let price = item
        .unit_price
        .map_or_else(String::new, |p| format!("${p:.0}"));
    let testid = format!("catalog-row-{id}");
    let pick = id.clone();
    view! {
        <button
            class="catalog-row"
            class:selected=is_selected
            data-testid=testid
            on:click=move |_| on_select.run(pick.clone())
        >
            <MaterialSwatch image=item.image.clone() category=item.category.clone() />
            <span class="catalog-row-name">{name}</span>
            <span class="catalog-row-price">{price}</span>
        </button>
    }
}

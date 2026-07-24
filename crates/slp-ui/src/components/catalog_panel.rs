//! The catalog inspector: a panel to browse and edit the plan's catalog — the
//! catalog-side counterpart of [`ObjectInspector`](super::ObjectInspector),
//! which edits a *placed* object. It lists every catalog item (starter,
//! hand-added, or ingested), and for the selected one lets you edit its
//! name/category/price and footprint dimensions. Edits are live: an object
//! references its catalog item by `catalog_ref` (not a copy), so changing an
//! item's price or size reprices and re-renders every object placed from it.

use leptos::prelude::*;
use slp_core::{CatalogItem, PriceUnit};

use super::{FileInput, IngestDraft, MaterialSwatch, NumberField, SelectField, TextField, Toggle};
use crate::vision::ExtractedProduct;

/// The `price_unit` id an area material / object is costed by — the string the
/// `SelectField` round-trips.
pub(crate) fn price_unit_id(unit: &PriceUnit) -> &'static str {
    match unit {
        PriceUnit::per_square_foot => "per_square_foot",
        PriceUnit::per_cubic_yard => "per_cubic_yard",
        PriceUnit::per_linear_foot => "per_linear_foot",
        // per_item, and any future variant, id as per-item.
        _ => "per_item",
    }
}

/// Parse a `price_unit` id back to its enum (unknown → per-item).
pub(crate) fn price_unit_from_id(id: &str) -> PriceUnit {
    match id {
        "per_square_foot" => PriceUnit::per_square_foot,
        "per_cubic_yard" => PriceUnit::per_cubic_yard,
        "per_linear_foot" => PriceUnit::per_linear_foot,
        _ => PriceUnit::per_item,
    }
}

/// The `price_unit` choices, `(id, label)`, for the editor's dropdown.
pub(crate) fn price_unit_options() -> Vec<(String, String)> {
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
    /// input the vision extractor will read. A product's page is often several
    /// screenshots (colors, sizes, laying patterns), so this is a list.
    #[prop(into, default = Signal::derive(Vec::new))]
    screenshots: Signal<Vec<String>>,
    /// Append a pasted screenshot (a `data:` URI) to the list.
    #[prop(default = Callback::new(|_: String| {}))]
    on_paste_image: Callback<String>,
    /// Remove the screenshot at the given index.
    #[prop(default = Callback::new(|_: usize| {}))]
    on_remove_image: Callback<usize>,
    /// Clear all pasted screenshots.
    #[prop(default = Callback::new(|(): ()| {}))]
    on_clear_images: Callback<()>,
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
    /// Approve curation: add these catalog items (one per selected combo).
    #[prop(default = Callback::new(|_: Vec<CatalogItem>| {}))]
    on_add_draft: Callback<Vec<CatalogItem>>,
    /// Discard the current draft.
    #[prop(default = Callback::new(|(): ()| {}))]
    on_discard_draft: Callback<()>,
    /// How many places in the plan reference the selected item (placements,
    /// area materials, courses, other items' base/bedding layers). Deletion is
    /// blocked while non-zero — removing the item would leave dangling refs.
    #[prop(into, default = Signal::derive(|| 0))]
    selected_in_use: Signal<usize>,
    /// Delete the selected catalog item (only reachable when unreferenced).
    #[prop(default = Callback::new(|_: String| {}))]
    on_delete: Callback<String>,
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
                            // Paste one or more product screenshots (⌘⇧4 → ⌘V,
                            // repeat) → data URIs the vision extractor (B3) reads.
                            <div
                                class="ingest-paste"
                                data-testid="ingest-paste"
                                tabindex="0"
                                on:paste=move |ev| read_pasted_image(&ev, on_paste_image)
                            >
                                "Click here, then paste product screenshots (⌘V, repeat for each — colors, sizes, laying patterns)."
                            </div>
                            {move || {
                                let shots = screenshots.get();
                                (!shots.is_empty())
                                    .then(|| {
                                        let thumbs = shots
                                            .into_iter()
                                            .enumerate()
                                            .map(|(i, shot)| {
                                                view! {
                                                    <div class="ingest-thumb">
                                                        <img
                                                            class="ingest-screenshot"
                                                            data-testid="ingest-screenshot"
                                                            src=shot
                                                            alt="pasted screenshot"
                                                        />
                                                        <button
                                                            class="ingest-thumb-remove"
                                                            data-testid=format!("ingest-remove-{i}")
                                                            title="Remove this screenshot"
                                                            on:click=move |_| on_remove_image.run(i)
                                                        >
                                                            "×"
                                                        </button>
                                                    </div>
                                                }
                                            })
                                            .collect::<Vec<_>>();
                                        view! {
                                            <div class="ingest-shots">{thumbs}</div>
                                            <button
                                                class="ingest-clear"
                                                data-testid="ingest-clear"
                                                on:click=move |_| on_clear_images.run(())
                                            >
                                                "Clear all"
                                            </button>
                                        }
                                    })
                            }}
                            // Once a screenshot is pasted: pick a model and run
                            // the vision extraction.
                            {move || {
                                (!screenshots.get().is_empty())
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
                            {move || {
                                draft
                                    .get()
                                    .map(|d| {
                                        view! {
                                            <IngestDraft
                                                product=d
                                                on_add=on_add_draft
                                                on_discard=on_discard_draft
                                                screenshots=screenshots
                                            />
                                        }
                                    })
                            }}
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
                let del_id = item.id.clone();
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
                            // Footprint (Width/Depth/Height) applies only to a
                            // placeable object; an area material (priced per
                            // ft²/yd³/linear-ft) tiles instead, so it shows Tile
                            // W/D below and hides these.
                            {(item.price_unit == PriceUnit::per_item)
                                .then(|| {
                                    view! {
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
                                    }
                                })}
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
                            // Tile W/D — the real-world repeat of the photo — is
                            // a material concern; an object doesn't tile.
                            {(item.price_unit != PriceUnit::per_item)
                                .then(|| {
                                    view! {
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
                                    }
                                })}
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
                            // Delete — blocked while anything in the plan still
                            // references the item (no dangling refs, ever).
                            <div class="catalog-delete-row">
                                <button
                                    class="catalog-delete"
                                    data-testid="catalog-delete"
                                    // Braced: a bare `> 0` here would end the tag.
                                    disabled={move || selected_in_use.get() > 0}
                                    on:click=move |_| on_delete.run(del_id.clone())
                                >
                                    "Delete"
                                </button>
                                {move || {
                                    let n = selected_in_use.get();
                                    (n > 0)
                                        .then(|| {
                                            view! {
                                                <span
                                                    class="catalog-delete-note"
                                                    data-testid="catalog-delete-note"
                                                >
                                                    {format!(
                                                        "In use — {n} reference{} in the plan",
                                                        if n == 1 { "" } else { "s" },
                                                    )}
                                                </span>
                                            }
                                        })
                                }}
                            </div>
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

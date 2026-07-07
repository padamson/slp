//! The catalog inspector: a panel to browse and edit the plan's catalog — the
//! catalog-side counterpart of [`ObjectInspector`](super::ObjectInspector),
//! which edits a *placed* object. It lists every catalog item (starter,
//! hand-added, or ingested), and for the selected one lets you edit its
//! name/category/price and footprint dimensions. Edits are live: an object
//! references its catalog item by `catalog_ref` (not a copy), so changing an
//! item's price or size reprices and re-renders every object placed from it.

use leptos::prelude::*;
use slp_core::{CatalogItem, PriceUnit};

use super::{NumberField, SelectField, TextField};

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

#[allow(clippy::too_many_arguments)]
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
    /// Set the selected item's footprint width / diameter (ft).
    on_width: Callback<f64>,
    /// Set the selected item's footprint depth (ft).
    on_depth: Callback<f64>,
    /// Set the selected item's height (ft).
    on_height: Callback<f64>,
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
                        </div>
                    },
                )
            }}
        </aside>
    }
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
            <span class="catalog-row-name">{name}</span>
            <span class="catalog-row-price">{price}</span>
        </button>
    }
}

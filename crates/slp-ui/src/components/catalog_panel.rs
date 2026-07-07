//! The catalog inspector: a panel to browse and edit the plan's catalog — the
//! catalog-side counterpart of [`ObjectInspector`](super::ObjectInspector),
//! which edits a *placed* object. It lists every catalog item (starter,
//! hand-added, or ingested), and for the selected one lets you edit its
//! name/category/price and footprint dimensions. Edits are live: an object
//! references its catalog item by `catalog_ref` (not a copy), so changing an
//! item's price or size reprices and re-renders every object placed from it.

use leptos::prelude::*;
use slp_core::CatalogItem;

use super::{NumberField, TextField};

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
                <button
                    class="catalog-close"
                    data-testid="catalog-close"
                    on:click=move |_| on_close.run(())
                >
                    "Close"
                </button>
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

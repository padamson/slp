//! A dropdown to choose which catalog item the furniture tool will place.
//! Controlled — the parent owns the selected id; choosing an option reports it
//! back via `on_pick`.

use leptos::prelude::*;
use slp_core::CatalogItem;

#[component]
pub fn CatalogPicker(
    testid: &'static str,
    /// The catalog to choose from.
    #[prop(into)]
    catalog: Signal<Vec<CatalogItem>>,
    /// The selected catalog id (empty = nothing chosen), owned by the parent.
    #[prop(into)]
    selected: Signal<String>,
    /// The user chose an item — its catalog id.
    on_pick: Callback<String>,
) -> impl IntoView {
    view! {
        <label class="catalog-picker">
            "Item "
            <select
                data-testid=testid
                prop:value=move || selected.get()
                on:change=move |ev| on_pick.run(event_target_value(&ev))
            >
                {move || {
                    catalog
                        .get()
                        .into_iter()
                        .map(|item| {
                            let label = item.name.unwrap_or_else(|| item.id.clone());
                            view! { <option value=item.id>{label}</option> }
                        })
                        .collect::<Vec<_>>()
                }}
            </select>
        </label>
    }
}

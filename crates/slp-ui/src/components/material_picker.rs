//! The Area tool's material picker: the catalog's area materials, **grouped by
//! category** so the toolbar stays compact as the catalog grows (ingesting a
//! dozen paver colors adds dropdown options, not a dozen buttons). Each category
//! is one armable button (showing the selected type's swatch) plus a **type
//! dropdown** when it has more than one material. Arming a category arms its
//! selected type; the drawn area then references that material for pricing and
//! tiling.

use std::collections::HashMap;

use leptos::prelude::*;
use slp_core::CatalogItem;

use super::{MaterialSwatch, ToolButton};

#[component]
pub fn MaterialPicker(
    /// The area materials to offer, in catalog order.
    #[prop(into)]
    materials: Signal<Vec<CatalogItem>>,
    /// The currently armed material id, if any.
    #[prop(into)]
    armed: Signal<Option<String>>,
    /// Arm the material with this id.
    on_arm: Callback<String>,
) -> impl IntoView {
    // Per-category chosen type (material id); lazily defaults to the category's
    // first material until the user picks from its dropdown.
    let selection = RwSignal::new(HashMap::<String, String>::new());
    view! {
        <div class="material-picker" data-testid="material-picker">
            {move || {
                // Group by category, preserving first-appearance (catalog) order.
                let mut order: Vec<String> = Vec::new();
                let mut groups: HashMap<String, Vec<CatalogItem>> = HashMap::new();
                for m in materials.get() {
                    let cat = m.category.clone().unwrap_or_else(|| "Other".to_string());
                    if !groups.contains_key(&cat) {
                        order.push(cat.clone());
                    }
                    groups.entry(cat).or_default().push(m);
                }
                order
                    .into_iter()
                    .map(|cat| {
                        let items = groups.remove(&cat).unwrap_or_default();
                        category_group(cat, items, selection, armed, on_arm)
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}

/// One category's control: an armable button (swatch + prettified name) plus a
/// type dropdown when the category has more than one material.
#[allow(clippy::needless_pass_by_value)] // owned data is cloned into 'static closures
fn category_group(
    cat: String,
    items: Vec<CatalogItem>,
    selection: RwSignal<HashMap<String, String>>,
    armed: Signal<Option<String>>,
    on_arm: Callback<String>,
) -> impl IntoView {
    let first_id = items.first().map(|c| c.id.clone()).unwrap_or_default();
    let options: Vec<(String, String)> = items
        .iter()
        .map(|c| (c.id.clone(), c.name.clone().unwrap_or_else(|| c.id.clone())))
        .collect();

    // The selected type for this category (armed one if it belongs here, else
    // the sticky choice, else the first).
    let selected = {
        let cat = cat.clone();
        Signal::derive(move || {
            selection
                .get()
                .get(&cat)
                .cloned()
                .unwrap_or_else(|| first_id.clone())
        })
    };
    let active = Signal::derive(move || armed.get() == Some(selected.get()));

    let arm_selected = move |()| on_arm.run(selected.get());
    let pick_type = {
        let cat = cat.clone();
        move |id: String| {
            selection.update(|m| {
                m.insert(cat.clone(), id.clone());
            });
            on_arm.run(id);
        }
    };

    let swatch = {
        let items = items.clone();
        move || {
            let sel = selected.get();
            let found = items.iter().find(|c| c.id == sel);
            view! {
                <MaterialSwatch
                    image=found.and_then(|c| c.image.clone())
                    category=found.and_then(|c| c.category.clone())
                />
            }
        }
    };

    let show_dropdown = options.len() > 1;
    let sel_testid = format!("area-mat-select-{cat}");
    view! {
        <div class="material-group">
            <ToolButton
                label=prettify(&cat)
                testid=format!("area-mat-cat-{cat}")
                active=active
                on_pick=Callback::new(arm_selected)
            >
                {swatch}
            </ToolButton>
            {show_dropdown
                .then(|| {
                    view! {
                        <select
                            class="material-type"
                            data-testid=sel_testid
                            on:change=move |ev| pick_type(event_target_value(&ev))
                        >
                            {options
                                .into_iter()
                                .map(|(id, name)| {
                                    let is_sel = {
                                        let id = id.clone();
                                        move || selected.get() == id
                                    };
                                    view! {
                                        <option value=id selected=is_sel>
                                            {name}
                                        </option>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    }
                })}
        </div>
    }
}

/// A category id as a button label: dashes → spaces, first letter upper-cased
/// (e.g. `mulch-bed` → "Mulch bed", `paver` → "Paver").
fn prettify(cat: &str) -> String {
    let spaced = cat.replace('-', " ");
    let mut chars = spaced.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_uppercase().collect::<String>() + chars.as_str()
    })
}

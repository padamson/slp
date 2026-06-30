//! theoria story for `CatalogPicker`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::CatalogItem;

use super::CatalogPicker;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Controls/CatalogPicker", || {
        let catalog = vec![
            furniture("sofa", "3-Seat Sofa"),
            furniture("chair", "Armchair"),
            furniture("table", "Coffee Table"),
        ];
        let selected = RwSignal::new("chair".to_string());
        view! {
            <CatalogPicker
                testid="catalog-picker"
                catalog=Signal::derive(move || catalog.clone())
                selected=selected
                on_pick=Callback::new(move |id| selected.set(id))
            />
        }
    })]
}

fn furniture(id: &str, name: &str) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c
}

//! theoria stories for `MaterialPicker`. Compiled only under the `stories`
//! feature.

use leptos::prelude::*;
use slp_core::{CatalogItem, PriceUnit};

use super::MaterialPicker;
use theoria::Story;

fn mat(id: &str, name: &str, category: &str, unit: PriceUnit) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c.category = Some(category.to_string());
    c.price_unit = unit;
    c
}

pub fn stories() -> Vec<Story> {
    vec![Story::new(
        "Panels/MaterialPicker/Mulch, paver, and ingested slabs",
        || {
            // Mulch + starter Pavers each stand alone; a batch of ingested Blu 60
            // colors collapse into one "Slab" button with a type dropdown.
            let materials = vec![
                mat("mulch", "Mulch", "mulch-bed", PriceUnit::per_cubic_yard),
                mat("paver", "Pavers", "paver", PriceUnit::per_square_foot),
                mat(
                    "blu-shale",
                    "Blu 60 — Shale Grey",
                    "slab",
                    PriceUnit::per_square_foot,
                ),
                mat(
                    "blu-onyx",
                    "Blu 60 — Onyx Black",
                    "slab",
                    PriceUnit::per_square_foot,
                ),
                mat(
                    "blu-brown",
                    "Blu 60 — Chestnut Brown",
                    "slab",
                    PriceUnit::per_square_foot,
                ),
            ];
            view! {
                <MaterialPicker
                    materials=Signal::derive(move || materials.clone())
                    armed=Signal::derive(|| Some("paver".to_string()))
                    on_arm=Callback::new(|_: String| {})
                />
            }
        },
    )]
}

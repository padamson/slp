//! theoria stories for `CatalogPanel`. Compiled only under the `stories`
//! feature.

use leptos::prelude::*;
use slp_core::{CatalogItem, PriceUnit};

use super::CatalogPanel;
use crate::vision::{ExtractedProduct, PriceUnitHint, SizeVariant, Variant};
use theoria::Story;

fn noop_str() -> Callback<String> {
    Callback::new(|_| {})
}
fn noop_f64() -> Callback<f64> {
    Callback::new(|_| {})
}
fn noop_pu() -> Callback<PriceUnit> {
    Callback::new(|_| {})
}
fn noop() -> Callback<()> {
    Callback::new(|()| {})
}

fn item(id: &str, name: &str, category: &str, price: f64) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c.category = Some(category.to_string());
    c.unit_price = Some(price);
    c.width_ft = Some(2.5);
    c.depth_ft = Some(3.0);
    c.height_ft = Some(2.5);
    c
}

fn sample() -> Vec<CatalogItem> {
    vec![
        item("lounge-chair", "Lounge chair", "furniture", 199.0),
        item("dining-table", "Dining table", "furniture", 649.0),
        item("fire-pit", "Fire pit", "fire-pit", 349.0),
    ]
}

#[allow(clippy::too_many_lines)]
pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Panels/CatalogPanel/Editing an item", || {
            let catalog = sample();
            view! {
                <CatalogPanel
                    catalog=Signal::derive(move || catalog.clone())
                    selected=Signal::derive(|| Some("lounge-chair".to_string()))
                    on_select=noop_str()
                    on_name=noop_str()
                    on_category=noop_str()
                    on_price=noop_f64()
                    on_price_unit=noop_pu()
                    on_add=noop()
                    on_width=noop_f64()
                    on_depth=noop_f64()
                    on_height=noop_f64()
                    on_close=noop()
                />
            }
        }),
        Story::new("Panels/CatalogPanel/Nothing selected", || {
            let catalog = sample();
            view! {
                <CatalogPanel
                    catalog=Signal::derive(move || catalog.clone())
                    selected=Signal::derive(|| None::<String>)
                    on_select=noop_str()
                    on_name=noop_str()
                    on_category=noop_str()
                    on_price=noop_f64()
                    on_price_unit=noop_pu()
                    on_add=noop()
                    on_width=noop_f64()
                    on_depth=noop_f64()
                    on_height=noop_f64()
                    on_close=noop()
                />
            }
        }),
        // Screenshot ingestion enabled: a key is present, so the gate reads
        // "enabled" (vs the default gated-off state the stories above show).
        Story::new("Panels/CatalogPanel/Ingestion enabled", || {
            let catalog = sample();
            view! {
                <CatalogPanel
                    catalog=Signal::derive(move || catalog.clone())
                    selected=Signal::derive(|| None::<String>)
                    on_select=noop_str()
                    on_name=noop_str()
                    on_category=noop_str()
                    on_price=noop_f64()
                    on_price_unit=noop_pu()
                    on_add=noop()
                    on_width=noop_f64()
                    on_depth=noop_f64()
                    on_height=noop_f64()
                    api_key=Signal::derive(|| "sk-ant-demo".to_string())
                    on_close=noop()
                />
            }
        }),
        // An extracted draft product (a configurator's variant matrix), pending
        // multi-select curation.
        Story::new("Panels/CatalogPanel/Extracted draft", || {
            let catalog = sample();
            let draft = ExtractedProduct {
                name: "Blu 60 Slate Slabs".to_string(),
                category: Some("slab".to_string()),
                price_unit: Some(PriceUnitHint::PerSquareFoot),
                unit_price: None,
                colors: vec![
                    Variant {
                        name: "Shale Grey".to_string(),
                        available: true,
                        bbox: None,
                        swatch: None,
                    },
                    Variant {
                        name: "Chestnut Brown".to_string(),
                        available: true,
                        bbox: None,
                        swatch: None,
                    },
                    Variant {
                        name: "Onyx Black".to_string(),
                        available: false,
                        bbox: None,
                        swatch: None,
                    },
                ],
                textures: vec![Variant {
                    name: "Slate".to_string(),
                    available: true,
                    bbox: None,
                    swatch: None,
                }],
                sizes: vec![
                    SizeVariant {
                        name: "60 MM".to_string(),
                        available: true,
                        width_ft: Some(1.083),
                        depth_ft: Some(1.083),
                        thickness_in: Some(2.375),
                        includes: Some("A: 6½×13, B: 13×13, C: 19½×13 in".to_string()),
                    },
                    SizeVariant {
                        name: "Grande".to_string(),
                        available: true,
                        width_ft: Some(2.71),
                        depth_ft: Some(1.63),
                        thickness_in: Some(2.375),
                        includes: None,
                    },
                ],
                notes: Some("No price listed — add your dealer quote.".to_string()),
            };
            view! {
                <CatalogPanel
                    catalog=Signal::derive(move || catalog.clone())
                    selected=Signal::derive(|| None::<String>)
                    on_select=noop_str()
                    on_name=noop_str()
                    on_category=noop_str()
                    on_price=noop_f64()
                    on_price_unit=noop_pu()
                    on_add=noop()
                    on_width=noop_f64()
                    on_depth=noop_f64()
                    on_height=noop_f64()
                    api_key=Signal::derive(|| "sk-ant-demo".to_string())
                    screenshots=Signal::derive(|| {
                        vec!["data:image/png;base64,iVBORw0KGgo=".to_string()]
                    })
                    draft=Signal::derive(move || Some(draft.clone()))
                    on_close=noop()
                />
            }
        }),
    ]
}

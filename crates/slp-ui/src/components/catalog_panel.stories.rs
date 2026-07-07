//! theoria stories for `CatalogPanel`. Compiled only under the `stories`
//! feature.

use leptos::prelude::*;
use slp_core::{CatalogItem, PriceUnit};

use super::CatalogPanel;
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
    ]
}

//! dokime component tests for `CatalogPanel`.

use leptos::prelude::*;
use slp_core::CatalogItem;

use super::CatalogPanel;

fn noop_str() -> Callback<String> {
    Callback::new(|_| {})
}
fn noop_f64() -> Callback<f64> {
    Callback::new(|_| {})
}
fn noop() -> Callback<()> {
    Callback::new(|()| {})
}

fn item(id: &str, name: &str, price: f64) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c.category = Some("furniture".to_string());
    c.unit_price = Some(price);
    c.width_ft = Some(2.5);
    c.depth_ft = Some(3.0);
    c.height_ft = Some(2.5);
    c
}

fn panel(catalog: Vec<CatalogItem>, selected: Option<String>) -> String {
    dokime::render(move || {
        let catalog = catalog.clone();
        let selected = selected.clone();
        view! {
            <CatalogPanel
                catalog=Signal::derive(move || catalog.clone())
                selected=Signal::derive(move || selected.clone())
                on_select=noop_str()
                on_name=noop_str()
                on_category=noop_str()
                on_price=noop_f64()
                on_width=noop_f64()
                on_depth=noop_f64()
                on_height=noop_f64()
                on_close=noop()
            />
        }
    })
}

#[test]
fn lists_every_catalog_item_with_its_price() {
    let html = panel(
        vec![
            item("chair", "Lounge chair", 199.0),
            item("table", "Dining table", 649.0),
        ],
        None,
    );
    assert!(html.contains("Lounge chair"));
    assert!(html.contains("$199"));
    assert!(html.contains("Dining table"));
    assert!(html.contains(r#"data-testid="catalog-row-chair""#));
    assert!(html.contains(r#"data-testid="catalog-row-table""#));
    // Nothing selected → no editor.
    assert_eq!(dokime::count(&html, r#"data-testid="catalog-editor""#), 0);
}

#[test]
fn the_selected_item_opens_an_editor_with_its_fields() {
    let html = panel(
        vec![item("chair", "Lounge chair", 199.0)],
        Some("chair".to_string()),
    );
    assert!(
        html.contains(r#"data-testid="catalog-editor""#),
        "the editor opens"
    );
    // Its current values are rendered into the fields.
    assert!(html.contains("Lounge chair"), "the name field");
    assert!(html.contains(r#"data-testid="catalog-name""#));
    assert!(html.contains(r#"data-testid="catalog-category""#));
    assert!(html.contains(r#"data-testid="catalog-price""#));
    assert!(html.contains(r#"data-testid="catalog-width""#));
    assert!(html.contains(r#"data-testid="catalog-height""#));
    // The selected row is marked.
    assert!(html.contains("selected"));
}

#[test]
fn has_a_close_button() {
    let html = panel(vec![item("chair", "Lounge chair", 199.0)], None);
    assert!(html.contains(r#"data-testid="catalog-close""#));
}

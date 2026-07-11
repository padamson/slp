//! dokime component tests for `CatalogPanel`.

use leptos::prelude::*;
use slp_core::{CatalogItem, PriceUnit};

use super::CatalogPanel;

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
                on_price_unit=noop_pu()
                on_add=noop()
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
fn the_editor_has_a_price_unit_control_reflecting_the_item() {
    let mut mulch = item("mulch", "Mulch", 40.0);
    mulch.price_unit = PriceUnit::per_cubic_yard;
    let html = panel(vec![mulch], Some("mulch".to_string()));
    assert!(
        html.contains(r#"data-testid="catalog-price-unit""#),
        "the price-unit dropdown"
    );
    // The item's per-yd³ unit is the selected option.
    assert!(
        html.contains(r#"value="per_cubic_yard" selected"#),
        "the current price_unit is selected"
    );
}

#[test]
fn a_bulk_material_offers_an_aggregate_toggle() {
    // A per-yd³ material can be marked a sub-base aggregate; a per-item object
    // cannot (the toggle is hidden).
    let mut gravel = item("gravel", "Gravel", 55.0);
    gravel.price_unit = PriceUnit::per_cubic_yard;
    let html = panel(vec![gravel], Some("gravel".to_string()));
    assert!(
        html.contains(r#"data-testid="catalog-aggregate""#),
        "a bulk material offers the aggregate toggle"
    );

    let chair_html = panel(
        vec![item("chair", "Chair", 199.0)],
        Some("chair".to_string()),
    );
    assert_eq!(
        dokime::count(&chair_html, r#"data-testid="catalog-aggregate""#),
        0,
        "a per-item object has no aggregate toggle"
    );
}

#[test]
fn the_editor_shows_an_image_field_and_previews_a_set_image() {
    // No image → the field is there but no preview.
    let plain = panel(
        vec![item("chair", "Chair", 199.0)],
        Some("chair".to_string()),
    );
    assert!(
        plain.contains(r#"data-testid="catalog-image""#),
        "an image field"
    );
    assert!(
        plain.contains(r#"data-testid="catalog-tile-width""#),
        "a tile-width field"
    );
    assert!(
        plain.contains(r#"data-testid="catalog-tile-depth""#),
        "a tile-depth field"
    );
    assert_eq!(
        dokime::count(&plain, r#"data-testid="catalog-image-preview""#),
        0,
        "no preview without an image"
    );

    // With an image → a preview <img> shows it.
    let mut paver = item("paver", "Pavers", 8.0);
    let src = "data:image/png;base64,iVBORw0KGgo=";
    paver.image = Some(src.to_string());
    let with_img = panel(vec![paver], Some("paver".to_string()));
    assert!(
        with_img.contains(r#"data-testid="catalog-image-preview""#),
        "a preview appears"
    );
    assert!(with_img.contains(src), "the preview points at the image");
}

#[test]
fn has_add_and_close_buttons() {
    let html = panel(vec![item("chair", "Lounge chair", 199.0)], None);
    assert!(
        html.contains(r#"data-testid="catalog-add""#),
        "an add-material button"
    );
    assert!(
        html.contains(r#"data-testid="catalog-close""#),
        "a close button"
    );
}

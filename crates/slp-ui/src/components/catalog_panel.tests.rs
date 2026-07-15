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
        plain.contains(r#"data-testid="catalog-image-file""#),
        "a file-upload input"
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

/// Render the panel with the screenshot-ingestion `api_key`/`screenshot` props
/// set (the section is independent of the catalog/selection).
fn panel_ingest(key: &str, shot: &str) -> String {
    let key = key.to_string();
    let shot = shot.to_string();
    dokime::render(move || {
        let key = key.clone();
        let shot = shot.clone();
        view! {
            <CatalogPanel
                catalog=Signal::derive(Vec::<CatalogItem>::new)
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
                api_key=Signal::derive(move || key.clone())
                screenshot=Signal::derive(move || shot.clone())
                on_close=noop()
            />
        }
    })
}

#[test]
fn the_ingestion_section_gates_on_the_api_key() {
    // No key → the section + a masked key field are shown, gated off.
    let off = panel(vec![item("chair", "Lounge chair", 199.0)], None);
    assert!(
        off.contains(r#"data-testid="ingest-section""#),
        "the screenshot-ingestion section renders"
    );
    assert!(
        off.contains(r#"data-testid="ingest-api-key""#),
        "the API-key field renders"
    );
    assert!(
        off.contains(r#"type="password""#),
        "the key field is masked"
    );
    assert!(
        off.contains("Add your Anthropic API key"),
        "the gated-off note prompts for a key"
    );
    assert!(
        !off.contains("Screenshot ingestion enabled"),
        "not enabled without a key"
    );
    assert_eq!(
        dokime::count(&off, r#"data-testid="ingest-paste""#),
        0,
        "no paste zone without a key"
    );

    // With a key → the gate flips to enabled and the paste zone appears.
    let on = panel_ingest("sk-ant-abc123", "");
    assert!(
        on.contains("Screenshot ingestion enabled"),
        "enabled once a key is present"
    );
    assert!(
        !on.contains("Add your Anthropic API key"),
        "the prompt is gone once a key is present"
    );
    assert!(
        on.contains(r#"data-testid="ingest-paste""#),
        "a paste zone appears once keyed"
    );
    // Nothing pasted yet → no preview.
    assert_eq!(
        dokime::count(&on, r#"data-testid="ingest-screenshot""#),
        0,
        "no screenshot preview before a paste"
    );
}

#[test]
fn a_pasted_screenshot_previews_with_a_clear_action() {
    let src = "data:image/png;base64,iVBORw0KGgo=";
    let html = panel_ingest("sk-ant-abc123", src);
    assert!(
        html.contains(r#"data-testid="ingest-screenshot""#),
        "the pasted screenshot previews"
    );
    assert!(html.contains(src), "the preview points at the pasted image");
    assert!(
        html.contains(r#"data-testid="ingest-clear""#),
        "a clear action is offered"
    );
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

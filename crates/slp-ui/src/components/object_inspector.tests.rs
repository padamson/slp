//! dokime component tests for `ObjectInspector`. (Clicking the status/virtual/
//! reset controls is exercised end-to-end in slp-e2e; dokime renders markup.)

use leptos::prelude::*;
use slp_core::{CatalogItem, Corner, FootprintShape, ItemStatus, Object};

use super::ObjectInspector;

fn chair() -> CatalogItem {
    let mut c = CatalogItem::new("chair".to_string());
    c.name = Some("Lounge chair".to_string());
    c.category = Some("furniture".to_string());
    c.width_ft = Some(2.5);
    c.depth_ft = Some(3.0);
    c.height_ft = Some(2.5);
    c.unit_price = Some(199.0);
    c
}

#[test]
fn shows_metadata_status_and_reset() {
    let mut obj = Object::new("chair".to_string(), 12.0, 8.0);
    obj.rot = Some(90.0);
    obj.status = ItemStatus::existing;
    let html = dokime::render(move || {
        view! {
            <ObjectInspector
                object=obj
                item=Some(chair())
                corner=Corner::Nw
                on_status=Callback::new(|_| {})
                on_virtual=Callback::new(|_| {})
                on_reset_rotation=Callback::new(|()| {})
                on_delete=Callback::new(|()| {})
            />
        }
    });
    assert!(html.contains("Lounge chair"), "name");
    assert!(html.contains("furniture"), "category");
    assert!(html.contains("2.5 × 3 ft"), "footprint");
    assert!(html.contains("$199.00"), "unit price");
    assert!(html.contains("(12.0, 8.0) ft"), "position");
    assert!(html.contains("90°"), "rotation");
    assert!(
        html.contains(r#"data-testid="reset-rotation""#),
        "reset button"
    );
    assert!(
        html.contains(r#"data-testid="status-existing""#),
        "status buttons"
    );
    assert!(
        html.contains(r#"data-testid="delete-object""#),
        "remove button"
    );
    assert!(
        html.contains(r#"data-testid="inspector-virtual""#),
        "the virtual toggle"
    );
    assert!(
        html.contains(r#"data-corner="nw""#),
        "floats in the chosen corner"
    );
    // The object's status (existing) is the one marked active.
    assert_eq!(
        dokime::count(&html, "active"),
        1,
        "one active status button"
    );
}

#[test]
fn falls_back_when_the_catalog_item_is_missing() {
    let obj = Object::new("mystery-id".to_string(), 1.0, 2.0);
    let html = dokime::render(move || {
        view! {
            <ObjectInspector
                object=obj
                corner=Corner::Se
                on_status=Callback::new(|_| {})
                on_virtual=Callback::new(|_| {})
                on_reset_rotation=Callback::new(|()| {})
                on_delete=Callback::new(|()| {})
            />
        }
    });
    assert!(
        html.contains("mystery-id"),
        "name falls back to the catalog_ref"
    );
    assert!(html.contains("—"), "missing metadata fields show a dash");
    assert!(html.contains(r#"data-corner="se""#), "the chosen corner");
}

#[test]
fn a_round_item_shows_its_diameter() {
    let mut fire_pit = CatalogItem::new("fire-pit".to_string());
    fire_pit.name = Some("Fire pit".to_string());
    fire_pit.shape = FootprintShape::circle;
    fire_pit.width_ft = Some(3.0);
    fire_pit.depth_ft = Some(3.0);
    let obj = Object::new("fire-pit".to_string(), 5.0, 5.0);
    let html = dokime::render(move || {
        view! {
            <ObjectInspector
                object=obj
                item=Some(fire_pit)
                corner=Corner::Nw
                on_status=Callback::new(|_| {})
                on_virtual=Callback::new(|_| {})
                on_reset_rotation=Callback::new(|()| {})
                on_delete=Callback::new(|()| {})
            />
        }
    });
    assert!(html.contains("⌀ 3 ft"), "a circle shows its diameter");
    assert!(!html.contains("×"), "not a width × depth for a round item");
}

//! dokime component tests for `Furnishings` (placed-object footprints).

use leptos::prelude::*;
use slp_core::{CatalogItem, Coord, ItemStatus, Object};

use super::{Furnishings, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 20.0,
    }
}

fn item(id: &str, w_ft: Option<f64>, d_ft: Option<f64>) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.width_ft = w_ft;
    c.depth_ft = d_ft;
    c
}

#[test]
fn renders_a_footprint_to_scale() {
    // A 3 ft × 1.5 ft item at (5,5): 10 px/ft → a 30 × 15 px rectangle centered
    // at sx(5)=50, sy(5)=150.
    let catalog = vec![item("chair", Some(3.0), Some(1.5))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        html.contains(r#"class="furnishings""#),
        "tagged for queries"
    );
    assert!(html.contains("<rect"), "the footprint is a rect");
    assert!(html.contains(r#"width="30""#), "3 ft → 30 px wide");
    assert!(html.contains(r#"height="15""#), "1.5 ft → 15 px deep");
    assert!(
        html.contains("translate(50,150)"),
        "centered at the object's position"
    );
}

#[test]
fn renders_one_group_per_object() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![
        Object::new("chair".to_string(), 2.0, 2.0),
        Object::new("chair".to_string(), 8.0, 8.0),
    ];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert_eq!(
        dokime::count(&html, r#"class="furniture-item""#),
        2,
        "one group per placed object"
    );
}

#[test]
fn rotation_is_applied_clockwise() {
    let catalog = vec![item("chair", Some(2.0), Some(1.0))];
    let mut obj = Object::new("chair".to_string(), 5.0, 5.0);
    obj.rot = Some(90.0);
    let objects = vec![obj];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        html.contains("rotate(90)"),
        "clockwise-from-north maps to SVG rotate(+rot)"
    );
}

#[test]
fn unresolved_catalog_ref_is_not_drawn() {
    // An object referencing an id absent from the catalog has no footprint to
    // draw, so nothing renders (mirrors the cost take-off's exclusion).
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![Object::new("ghost-id".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(!html.contains(r#"class="furnishings""#));
}

#[test]
fn existing_and_virtual_objects_still_render() {
    // status affects only the cost take-off, not visibility — every placed object
    // is shown on the plan.
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let mut existing = Object::new("chair".to_string(), 2.0, 2.0);
    existing.status = ItemStatus::existing;
    let mut ghost = Object::new("chair".to_string(), 8.0, 8.0);
    ghost.status = ItemStatus::r#virtual;
    let objects = vec![existing, ghost];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert_eq!(
        dokime::count(&html, r#"class="furniture-item""#),
        2,
        "existing + virtual both render"
    );
}

#[test]
fn missing_dimensions_fall_back_to_a_default_footprint() {
    // A catalog item with no width/depth still places a visible 1 ft (= 10 px)
    // square so the object can be seen and selected.
    let catalog = vec![item("mystery", None, None)];
    let objects = vec![Object::new("mystery".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(html.contains(r#"width="10""#), "1 ft default → 10 px");
}

fn deck(w: f64, d: f64) -> Vec<Coord> {
    vec![
        Coord::new(0.0, 0.0),
        Coord::new(w, 0.0),
        Coord::new(w, d),
        Coord::new(0.0, d),
    ]
}

#[test]
fn an_object_overhanging_its_surface_is_highlighted() {
    // A 4×4 ft chair centered at (5,5) pokes past a 6×6 ft deck (corners reach
    // x=7, y=7) — it does not fit on a single surface.
    let catalog = vec![item("chair", Some(4.0), Some(4.0))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog surfaces=vec![deck(6.0, 6.0)] /> }
    });
    assert!(
        html.contains("furniture-item--overflows"),
        "an overhanging object is highlighted"
    );
}

#[test]
fn an_object_fully_on_a_surface_is_not_highlighted() {
    // A 2×2 ft chair well inside a 10×10 ft deck fits.
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog surfaces=vec![deck(10.0, 10.0)] /> }
    });
    assert!(
        !html.contains("furniture-item--overflows"),
        "a contained object keeps the normal outline"
    );
}

#[test]
fn no_surfaces_means_no_fit_check() {
    // Without surfaces there is nothing to fit within, so nothing is highlighted.
    let catalog = vec![item("chair", Some(4.0), Some(4.0))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog /> }
    });
    assert!(!html.contains("furniture-item--overflows"));
}

#[test]
fn no_objects_renders_nothing() {
    let html = dokime::render(move || view! { <Furnishings t=t() objects=Vec::new() /> });
    assert!(!html.contains(r#"class="furnishings""#));
}

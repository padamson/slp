//! dokime component tests for `MaterialPicker`.

use leptos::prelude::*;
use slp_core::{CatalogItem, PriceUnit};

use super::MaterialPicker;

fn mat(id: &str, name: &str, category: &str) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c.category = Some(category.to_string());
    c.price_unit = PriceUnit::per_square_foot;
    c
}

fn render(materials: Vec<CatalogItem>, armed: Option<String>) -> String {
    dokime::render(move || {
        let materials = materials.clone();
        let armed = armed.clone();
        view! {
            <MaterialPicker
                materials=Signal::derive(move || materials.clone())
                armed=Signal::derive(move || armed.clone())
                on_arm=Callback::new(|_: String| {})
            />
        }
    })
}

#[test]
fn one_button_per_category_with_a_prettified_label() {
    let html = render(
        vec![
            mat("mulch", "Mulch", "mulch-bed"),
            mat("paver", "Pavers", "paver"),
        ],
        None,
    );
    assert!(html.contains(r#"data-testid="area-mat-cat-mulch-bed""#));
    assert!(html.contains(r#"data-testid="area-mat-cat-paver""#));
    // Category ids become tidy button labels.
    assert!(html.contains("Mulch bed"), "dashes → spaces, capitalized");
    assert!(html.contains("Paver"));
}

#[test]
fn a_multi_material_category_gets_a_type_dropdown() {
    // Several ingested slabs collapse into one button + a type dropdown, so the
    // toolbar stays compact.
    let html = render(
        vec![
            mat("blu-shale", "Blu 60 — Shale Grey", "slab"),
            mat("blu-onyx", "Blu 60 — Onyx Black", "slab"),
            mat("blu-brown", "Blu 60 — Chestnut Brown", "slab"),
        ],
        None,
    );
    assert_eq!(
        dokime::count(&html, r#"data-testid="area-mat-cat-slab""#),
        1,
        "one button for the whole category"
    );
    assert!(
        html.contains(r#"data-testid="area-mat-select-slab""#),
        "a type dropdown for the category"
    );
    assert!(html.contains("Blu 60 — Shale Grey"));
    assert!(html.contains("Blu 60 — Onyx Black"));
}

#[test]
fn a_single_material_category_has_no_dropdown() {
    let html = render(vec![mat("paver", "Pavers", "paver")], None);
    assert_eq!(
        dokime::count(&html, r#"data-testid="area-mat-select-paver""#),
        0,
        "a lone material needs no type dropdown"
    );
}

#[test]
fn the_armed_materials_category_button_is_active() {
    let armed = render(
        vec![
            mat("mulch", "Mulch", "mulch-bed"),
            mat("paver", "Pavers", "paver"),
        ],
        Some("paver".to_string()),
    );
    assert!(
        armed.contains(r#"class="active""#),
        "the armed category button is marked active"
    );
    let none = render(
        vec![
            mat("mulch", "Mulch", "mulch-bed"),
            mat("paver", "Pavers", "paver"),
        ],
        None,
    );
    assert!(
        !none.contains(r#"class="active""#),
        "nothing is active when nothing is armed"
    );
}

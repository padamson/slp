//! dokime component tests for `EstimatePanel`.

use leptos::prelude::*;
use slp_core::{BillOfMaterials, LineItem, PriceUnit};

use super::EstimatePanel;

fn line(catalog_ref: &str, name: Option<&str>, qty: u32, unit: f64) -> LineItem {
    LineItem {
        catalog_ref: catalog_ref.to_string(),
        name: name.map(ToString::to_string),
        quantity: f64::from(qty),
        unit: PriceUnit::per_item,
        unit_price: unit,
        line_total: f64::from(qty) * unit,
    }
}

fn material_line(name: &str, quantity: f64, unit: PriceUnit, unit_price: f64) -> LineItem {
    LineItem {
        catalog_ref: name.to_lowercase(),
        name: Some(name.to_string()),
        quantity,
        unit,
        unit_price,
        line_total: quantity * unit_price,
    }
}

#[test]
fn renders_line_items_and_grand_total() {
    let bom = BillOfMaterials {
        lines: vec![
            line("chair", Some("Lounge chair"), 2, 199.0),
            line("sofa", Some("Outdoor sofa"), 1, 899.0),
        ],
        grand_total: 1297.0,
    };
    let html =
        dokime::render(move || view! { <EstimatePanel bom=Signal::derive(move || bom.clone()) /> });
    assert!(
        html.contains("Lounge chair") && html.contains("Outdoor sofa"),
        "item names"
    );
    assert_eq!(
        dokime::count(&html, r#"class="estimate-row""#),
        2,
        "a row per line item"
    );
    assert!(html.contains("$398.00"), "2 × $199 line total");
    assert!(html.contains("$1297.00"), "the grand total");
}

#[test]
fn a_line_falls_back_to_its_id_when_unnamed() {
    let bom = BillOfMaterials {
        lines: vec![line("side-table", None, 1, 89.0)],
        grand_total: 89.0,
    };
    let html =
        dokime::render(move || view! { <EstimatePanel bom=Signal::derive(move || bom.clone()) /> });
    assert!(
        html.contains("side-table"),
        "the catalog id labels the row when there's no name"
    );
}

#[test]
fn a_material_line_reads_its_quantity_in_its_own_measure() {
    // A mulch line reads yd³; a paver line reads ft² — not a bare count.
    let bom = BillOfMaterials {
        lines: vec![
            material_line("Mulch", 0.74, PriceUnit::per_cubic_yard, 40.0),
            material_line("Pavers", 100.0, PriceUnit::per_square_foot, 6.0),
        ],
        grand_total: 629.6,
    };
    let html =
        dokime::render(move || view! { <EstimatePanel bom=Signal::derive(move || bom.clone()) /> });
    assert!(html.contains("0.7 yd³"), "mulch quantity in yd³");
    assert!(html.contains("100 ft²"), "paver quantity in ft²");
}

#[test]
fn an_empty_bom_shows_a_placeholder_not_a_table() {
    let html =
        dokime::render(|| view! { <EstimatePanel bom=Signal::derive(BillOfMaterials::default) /> });
    assert!(
        html.contains("Place furniture"),
        "a prompt when nothing is placed"
    );
    assert!(!html.contains("<table"), "no table without line items");
}

//! theoria story for `EstimatePanel`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{BillOfMaterials, LineItem, PriceUnit};

use super::EstimatePanel;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Panels/EstimatePanel/Populated", || {
            let line =
                |catalog_ref: &str, name: &str, quantity: f64, unit: PriceUnit, price: f64| {
                    LineItem {
                        catalog_ref: catalog_ref.to_string(),
                        name: Some(name.to_string()),
                        quantity,
                        unit,
                        unit_price: price,
                        line_total: quantity * price,
                        patterns: Vec::new(),
                    }
                };
            let lines = vec![
                line(
                    "lounge-chair",
                    "Lounge chair",
                    4.0,
                    PriceUnit::per_item,
                    199.0,
                ),
                line(
                    "outdoor-sofa",
                    "Outdoor sofa",
                    1.0,
                    PriceUnit::per_item,
                    899.0,
                ),
                line("mulch", "Mulch", 2.3, PriceUnit::per_cubic_yard, 40.0),
                line("paver", "Pavers", 120.0, PriceUnit::per_square_foot, 6.0),
            ];
            let grand_total = lines.iter().map(|l| l.line_total).sum();
            let bom = BillOfMaterials { lines, grand_total };
            view! { <EstimatePanel bom=Signal::derive(move || bom.clone()) /> }
        }),
        Story::new("Panels/EstimatePanel/Empty (placeholder)", || {
            let bom = BillOfMaterials {
                lines: Vec::new(),
                grand_total: 0.0,
            };
            view! { <EstimatePanel bom=Signal::derive(move || bom.clone()) /> }
        }),
    ]
}

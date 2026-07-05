//! theoria story for `EstimatePanel`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{BillOfMaterials, LineItem};

use super::EstimatePanel;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Panels/EstimatePanel/Populated", || {
            let line = |catalog_ref: &str, name: &str, qty: u32, unit: f64| LineItem {
                catalog_ref: catalog_ref.to_string(),
                name: Some(name.to_string()),
                qty,
                unit_price: unit,
                line_total: f64::from(qty) * unit,
            };
            let lines = vec![
                line("lounge-chair", "Lounge chair", 4, 199.0),
                line("outdoor-sofa", "Outdoor sofa", 1, 899.0),
                line("dining-table", "Dining table", 1, 649.0),
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

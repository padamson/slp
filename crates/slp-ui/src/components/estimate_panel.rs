//! The estimate side panel: a live bill of materials — one row per catalog
//! item or material (quantity × unit price = line total) plus the grand total.
//! A row's quantity reads in its own measure (a count of objects, ft² of a
//! paver surface, or yd³ of mulch/gravel). It's a pure view of a
//! `BillOfMaterials`; the take-off math lives in `slp_core::take_off`, and the
//! parent recomputes it as furniture/areas change, so the panel reacts live.

use leptos::prelude::*;
use slp_core::{BillOfMaterials, PriceUnit};

#[component]
pub fn EstimatePanel(#[prop(into)] bom: Signal<BillOfMaterials>) -> impl IntoView {
    view! {
        <aside class="estimate" data-testid="estimate">
            <h2>"Estimate"</h2>
            {move || {
                let bom = bom.get();
                if bom.lines.is_empty() {
                    view! {
                        <p class="estimate-empty">"Place furniture to see its cost."</p>
                    }
                        .into_any()
                } else {
                    let rows = bom
                        .lines
                        .into_iter()
                        .map(|line| {
                            let name = line.name.unwrap_or(line.catalog_ref);
                            // Note the chosen laying pattern(s) on a material
                            // line, so the shopping trip knows the layout the
                            // piece mix is ordered for.
                            let pattern_note = (!line.patterns.is_empty())
                                .then(|| {
                                    view! {
                                        <span
                                            class="estimate-pattern"
                                            data-testid="estimate-pattern"
                                        >
                                            {format!(" ({})", line.patterns.join(", "))}
                                        </span>
                                    }
                                });
                            view! {
                                <tr class="estimate-row">
                                    <td class="estimate-name">{name}{pattern_note}</td>
                                    <td class="estimate-qty">{measure(line.quantity, &line.unit)}</td>
                                    <td class="estimate-unit">{dollars(line.unit_price)}</td>
                                    <td class="estimate-line">{dollars(line.line_total)}</td>
                                </tr>
                            }
                        })
                        .collect::<Vec<_>>();
                    view! {
                        <table>
                            <thead>
                                <tr>
                                    <th>"Item"</th>
                                    <th>"Qty"</th>
                                    <th>"Unit"</th>
                                    <th>"Total"</th>
                                </tr>
                            </thead>
                            <tbody>{rows}</tbody>
                            <tfoot>
                                <tr class="estimate-grand">
                                    <td colspan="3">"Total"</td>
                                    <td data-testid="estimate-total">{dollars(bom.grand_total)}</td>
                                </tr>
                            </tfoot>
                        </table>
                    }
                        .into_any()
                }
            }}
        </aside>
    }
}

/// Format dollars for display, e.g. `199.5` → `"$199.50"`.
fn dollars(v: f64) -> String {
    format!("${v:.2}")
}

/// A line's quantity in its own measure: a whole count for objects, or a
/// decimal + unit for a material (ft²/yd³/linear-ft).
fn measure(quantity: f64, unit: &PriceUnit) -> String {
    match unit {
        PriceUnit::per_item => format!("{quantity:.0}"),
        PriceUnit::per_square_foot => format!("{quantity:.0} ft²"),
        PriceUnit::per_cubic_yard => format!("{quantity:.1} yd³"),
        PriceUnit::per_linear_foot => format!("{quantity:.0} lf"),
        _ => format!("{quantity:.1}"),
    }
}

//! The estimate side panel: a live bill of materials for the placed objects —
//! one row per catalog item (qty × unit price = line total) plus the grand
//! total. It's a pure view of a `BillOfMaterials`; the take-off math lives in
//! `slp_core::take_off`, and the parent recomputes it as furniture is placed or
//! removed, so the panel reacts live.

use leptos::prelude::*;
use slp_core::BillOfMaterials;

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
                            view! {
                                <tr class="estimate-row">
                                    <td class="estimate-name">{name}</td>
                                    <td class="estimate-qty">{line.qty}</td>
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

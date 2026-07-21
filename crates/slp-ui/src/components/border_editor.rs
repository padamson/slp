//! The border editor: a drawn area's ordered **border rings** (a contrasting
//! paver course, a cobble band, an edging stone), outermost first — each a
//! material + a laid width in feet, editable per area. Embedded in the
//! [`AreaInspector`](super::AreaInspector) so a patio can be edged right where
//! its composition is tuned. Controlled — the parent owns the `borders` and
//! applies each edit, recomputing cost and re-tiling the rings live.

use leptos::prelude::*;
use slp_core::Border;

use super::{NumberField, SelectField};

/// The From/To options for a span select: "—" (whole perimeter) plus one entry
/// per boundary node index, and — when the current value is a fractional
/// boundary position set by dragging its handle — that position too, so the
/// select shows the truth instead of falling back to "—".
fn span_options(node_count: usize, current: Option<f64>) -> Vec<(String, String)> {
    let mut opts: Vec<(String, String)> = std::iter::once((String::new(), "—".to_string()))
        .chain((0..node_count).map(|i| (i.to_string(), format!("n{i}"))))
        .collect();
    if let Some(v) = current
        && v.fract().abs() > 1e-9
    {
        opts.push((format!("{v}"), format!("{v:.2}")));
    }
    opts
}

#[allow(clippy::needless_pass_by_value)]
#[component]
pub fn BorderEditor(
    /// The area's ordered border rings, outermost first.
    borders: Vec<Border>,
    /// Materials a ring may be made of, as `(id, label)` — the catalog's
    /// per-ft² and per-linear-ft materials (border pavers and edging stones).
    material_options: Vec<(String, String)>,
    /// How many boundary nodes the area has — drives the From/To span selects
    /// (0, e.g. a circle, hides them: circles only ring the full perimeter).
    #[prop(default = 0)]
    node_count: usize,
    /// Set ring `i`'s material to the given catalog id.
    on_material: Callback<(usize, String)>,
    /// Set ring `i`'s laid width (feet).
    on_width: Callback<(usize, f64)>,
    /// Set ring `i`'s span start position (`None` = whole perimeter).
    #[prop(default = Callback::new(|_: (usize, Option<f64>)| {}))]
    on_start: Callback<(usize, Option<f64>)>,
    /// Set ring `i`'s span end position (`None` = whole perimeter).
    #[prop(default = Callback::new(|_: (usize, Option<f64>)| {}))]
    on_end: Callback<(usize, Option<f64>)>,
    /// Append a new ring (inside the current innermost).
    on_add: Callback<()>,
    /// Remove ring `i`.
    on_remove: Callback<usize>,
) -> impl IntoView {
    let rows = borders
        .into_iter()
        .enumerate()
        .map(|(i, border)| {
            let options = material_options.clone();
            // From/To span selects on their own full-width line, so each shows
            // its node number: border only the edges between two nodes (walking
            // forward in drawn order), or "—" for the whole ring.
            let span = (node_count > 0).then(|| {
                let val = |v: Option<f64>| v.map_or_else(String::new, |v| format!("{v}"));
                view! {
                    <div class="border-row-span">
                        <SelectField
                            label="from"
                            testid="border-from"
                            value=val(border.start_node)
                            options=span_options(node_count, border.start_node)
                            on_change=Callback::new(move |id: String| {
                                on_start.run((i, id.parse().ok()));
                            })
                        />
                        <SelectField
                            label="to"
                            testid="border-to"
                            value=val(border.end_node)
                            options=span_options(node_count, border.end_node)
                            on_change=Callback::new(move |id: String| {
                                on_end.run((i, id.parse().ok()));
                            })
                        />
                    </div>
                }
            });
            view! {
                <div class="border-row" data-testid=format!("border-row-{i}")>
                    <div class="border-row-top">
                        <SelectField
                            label=""
                            testid="border-material"
                            value=border.material_ref
                            options=options
                            on_change=Callback::new(move |id: String| on_material.run((i, id)))
                        />
                        <NumberField
                            label="ft"
                            testid="border-width"
                            value=border.width_ft
                            step=0.25
                            min=0.0
                            on_input=Callback::new(move |w: f64| on_width.run((i, w)))
                        />
                        <button
                            class="border-remove"
                            data-testid=format!("border-remove-{i}")
                            on:click=move |_| on_remove.run(i)
                        >
                            "×"
                        </button>
                    </div>
                    {span}
                </div>
            }
        })
        .collect::<Vec<_>>();
    view! {
        <div class="border-editor" data-testid="border-editor">
            <div class="border-editor-label">"Borders"</div>
            {rows}
            <button class="border-add" data-testid="border-add" on:click=move |_| on_add.run(())>
                "+ Border"
            </button>
        </div>
    }
}

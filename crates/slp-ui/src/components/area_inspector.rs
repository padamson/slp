//! The area inspector: a small window that floats in an empty yard corner when
//! a drawn area (a paver patio, a mulch bed, …) is selected — the area
//! equivalent of [`ObjectInspector`](super::ObjectInspector). It shows the
//! area's material, size (ft²), and cost, and lets you edit its elevation and
//! (for a volume-priced material like mulch) its depth, or remove it. Editing
//! is live: the parent owns the selected area and recomputes area/volume/cost
//! from the reactive plan, so the estimate updates as you type.

use leptos::prelude::*;
use slp_core::Corner;

use super::NumberField;

/// Short name for the corner the window floats in (for `data-corner`).
fn corner_name(corner: Corner) -> &'static str {
    match corner {
        Corner::Nw => "nw",
        Corner::Sw => "sw",
        Corner::Ne => "ne",
        Corner::Se => "se",
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
pub fn AreaInspector(
    /// The area's title — its material name (e.g. "Mulch", "Pavers") or a
    /// fallback like "Area" when it has no material.
    #[prop(into)]
    title: String,
    /// The area's category (e.g. `mulch-bed`, `paver`), shown as a label.
    #[prop(default = None)]
    category: Option<String>,
    /// The enclosed area, in ft².
    area_ft2: f64,
    /// The area's elevation (ft), editable.
    #[prop(into)]
    elevation: Signal<f64>,
    /// The area's material depth (in), editable — shown only when
    /// `show_depth` is set (a volume-priced material like mulch, where depth
    /// drives the cost); a per-ft² paver hides the field (pass any value).
    #[prop(into)]
    depth: Signal<f64>,
    /// Whether to show the editable depth field (true for a volume-priced
    /// material).
    #[prop(default = false)]
    show_depth: bool,
    /// The cost of this area's material, in dollars — `None` when the area has
    /// no priced material.
    #[prop(default = None)]
    cost: Option<f64>,
    /// Which yard corner it floats in (exposed as `data-corner`).
    corner: Corner,
    /// Inline position (top/left/right/bottom in px) computed by the parent from
    /// the measured canvas metrics, so it sits inside the grid corner.
    #[prop(optional, into)]
    style: String,
    /// Set the area's elevation (ft).
    on_elevation: Callback<f64>,
    /// Set the area's material depth (in).
    on_depth: Callback<f64>,
    /// Remove the area from the plan.
    on_delete: Callback<()>,
) -> impl IntoView {
    let dash = || "—".to_string();
    let area_label = format!("{area_ft2:.0} ft²");
    let category_display = category.unwrap_or_else(dash);
    let cost_display = cost.map_or_else(dash, |c| format!("${c:.2}"));

    view! {
        <aside
            class="area-inspector"
            data-corner=corner_name(corner)
            data-testid="area-inspector"
            style=style
        >
            <h3 class="inspector-name">{title}</h3>
            <dl class="inspector-meta">
                <dt>"Material"</dt>
                <dd>{category_display}</dd>
                <dt>"Area"</dt>
                <dd data-testid="area-inspector-area">{area_label}</dd>
                <dt>"Cost"</dt>
                <dd data-testid="area-inspector-cost">{cost_display}</dd>
            </dl>
            <div class="inspector-area-size">
                <NumberField
                    label="Elev (ft)"
                    testid="area-inspector-elevation"
                    value=elevation
                    step=0.5
                    on_input=on_elevation
                />
                {show_depth
                    .then(|| {
                        view! {
                            <NumberField
                                label="Depth (in)"
                                testid="area-inspector-depth"
                                value=depth
                                step=1.0
                                min=0.0
                                on_input=on_depth
                            />
                        }
                    })}
            </div>
            <button
                class="inspector-delete"
                data-testid="delete-area"
                on:click=move |_| on_delete.run(())
            >
                "Remove"
            </button>
        </aside>
    }
}

//! The area inspector: a small window that floats in an empty yard corner when
//! a drawn area (a paver patio, a mulch bed, …) or a structure (the house, a
//! deck level) is selected — the region equivalent of
//! [`ObjectInspector`](super::ObjectInspector). For a drawn area it shows the
//! material, size (ft²), and cost, and lets you edit its elevation and (for a
//! volume-priced material like mulch) its depth. For a structure it shows the
//! size and lets you set its structure status (existing/planned) and, for a
//! deck level, its elevation. Either can be removed. Editing is live: the
//! parent owns the selection and recomputes area/volume/cost from the reactive
//! plan, so the estimate updates as you type.

use leptos::prelude::*;
use slp_core::{Corner, ItemStatus};

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

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
#[component]
pub fn AreaInspector(
    /// The region's title — a drawn area's material name (e.g. "Mulch",
    /// "Pavers"), or a structure's name ("House", "Deck").
    #[prop(into)]
    title: String,
    /// A drawn area's material category (e.g. `mulch-bed`, `paver`), shown as a
    /// label. Absent for a structure (which shows a status control instead).
    #[prop(default = None)]
    category: Option<String>,
    /// The enclosed area, in ft².
    area_ft2: f64,
    /// The region's elevation (ft), editable — shown only when `show_elevation`
    /// is set (a drawn area or a deck level; the house sits at grade and hides
    /// it).
    #[prop(into)]
    elevation: Signal<f64>,
    /// Whether to show the editable elevation field.
    #[prop(default = true)]
    show_elevation: bool,
    /// The area's material depth (in), editable — shown only when `show_depth`
    /// is set (a volume-priced material like mulch, where depth drives the
    /// cost); a per-ft² paver or a structure hides the field (pass any value).
    #[prop(into)]
    depth: Signal<f64>,
    /// Whether to show the editable depth field (true for a volume-priced
    /// material).
    #[prop(default = false)]
    show_depth: bool,
    /// The cost of this area's material, in dollars — `None` when the area has
    /// no priced material. Ignored in structure mode (see `status`).
    #[prop(default = None)]
    cost: Option<f64>,
    /// A structure's build status (existing/planned). When set, the panel is in
    /// *structure* mode: it shows existing/planned buttons instead of the
    /// material and cost rows. `None` for a drawn area.
    #[prop(default = None)]
    status: Option<ItemStatus>,
    /// Which yard corner it floats in (exposed as `data-corner`).
    corner: Corner,
    /// Inline position (top/left/right/bottom in px) computed by the parent from
    /// the measured canvas metrics, so it sits inside the grid corner.
    #[prop(optional, into)]
    style: String,
    /// Set the region's elevation (ft).
    on_elevation: Callback<f64>,
    /// Set the area's material depth (in).
    on_depth: Callback<f64>,
    /// Set a structure's build status — only meaningful in structure mode.
    #[prop(default = Callback::new(|_: ItemStatus| {}))]
    on_status: Callback<ItemStatus>,
    /// Remove the region from the plan.
    on_delete: Callback<()>,
) -> impl IntoView {
    let dash = || "—".to_string();
    let area_label = format!("{area_ft2:.0} ft²");
    let cost_display = cost.map_or_else(dash, |c| format!("${c:.2}"));
    // Structure mode (house / deck level) swaps the material + cost rows for a
    // build-status control.
    let is_structure = status.is_some();
    // The material row appears only for a drawn area that has a material.
    let material_row = (!is_structure)
        .then_some(category)
        .flatten()
        .map(|c| view! { <dt>"Material"</dt><dd>{c}</dd> });
    let cost_row = (!is_structure).then(|| {
        view! {
            <dt>"Cost"</dt>
            <dd data-testid="area-inspector-cost">{cost_display}</dd>
        }
    });

    let selected_status = status.clone();
    let status_btn = move |value: ItemStatus, label: &'static str, testid: &'static str| {
        let active = Some(&value) == selected_status.as_ref();
        view! {
            <button
                class="status-btn"
                class:active=active
                data-testid=testid
                on:click=move |_| on_status.run(value.clone())
            >
                {label}
            </button>
        }
    };
    let status_row = status.map(|_| {
        view! {
            <div class="inspector-status" data-testid="area-status">
                {status_btn(ItemStatus::planned, "Planned", "area-status-planned")}
                {status_btn(ItemStatus::existing, "Existing", "area-status-existing")}
            </div>
        }
    });

    view! {
        <aside
            class="area-inspector"
            data-corner=corner_name(corner)
            data-testid="area-inspector"
            style=style
        >
            <h3 class="inspector-name">{title}</h3>
            <dl class="inspector-meta">
                {material_row}
                <dt>"Area"</dt>
                <dd data-testid="area-inspector-area">{area_label}</dd>
                {cost_row}
            </dl>
            {status_row}
            <div class="inspector-area-size">
                {show_elevation
                    .then(|| {
                        view! {
                            <NumberField
                                label="Elev (ft)"
                                testid="area-inspector-elevation"
                                value=elevation
                                step=0.5
                                on_input=on_elevation
                            />
                        }
                    })}
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

//! The object inspector: a small window that floats in an empty yard corner when
//! an object is selected. It shows the object's metadata (name, category,
//! footprint, height, unit price, position, rotation) and lets you set its cost
//! status and reset its rotation — rotation is otherwise turned with the drag
//! handle on the object itself.

use leptos::prelude::*;
use slp_core::{CatalogItem, Corner, ItemStatus, Object};

/// Short name for the corner the window floats in (for `data-corner`).
fn corner_name(corner: Corner) -> &'static str {
    match corner {
        Corner::Nw => "nw",
        Corner::Sw => "sw",
        Corner::Ne => "ne",
        Corner::Se => "se",
    }
}

#[component]
pub fn ObjectInspector(
    object: Object,
    /// The catalog item the object references, if it resolves.
    #[prop(default = None)]
    item: Option<CatalogItem>,
    /// Which yard corner it floats in (exposed as `data-corner`).
    corner: Corner,
    /// Inline position (top/left/right/bottom in px) computed by the parent from
    /// the measured canvas metrics, so it sits inside the grid corner.
    #[prop(optional, into)]
    style: String,
    /// Set the object's cost status.
    on_status: Callback<ItemStatus>,
    /// Reset the object's rotation to 0°.
    on_reset_rotation: Callback<()>,
) -> impl IntoView {
    let dash = || "—".to_string();
    let Object {
        catalog_ref,
        rot,
        status,
        x,
        y,
    } = object;
    let position = format!("({x:.1}, {y:.1}) ft");
    let rotation = format!("{:.0}°", rot.unwrap_or(0.0));
    // Metadata from the resolved catalog item (dashes when absent); consuming
    // `item` on the last use so it isn't a needless by-value borrow.
    let name = item
        .as_ref()
        .and_then(|i| i.name.clone())
        .unwrap_or_else(|| catalog_ref.clone());
    let category = item
        .as_ref()
        .and_then(|i| i.category.clone())
        .unwrap_or_else(dash);
    let footprint = item
        .as_ref()
        .and_then(|i| Some(format!("{} × {} ft", i.width_ft?, i.depth_ft?)))
        .unwrap_or_else(dash);
    let height = item
        .as_ref()
        .and_then(|i| i.height_ft)
        .map_or_else(dash, |h| format!("{h} ft"));
    let price = item
        .and_then(|i| i.unit_price)
        .map_or_else(dash, |p| format!("${p:.2}"));

    let status_btn = |value: ItemStatus, label: &'static str, testid: &'static str| {
        let active = value == status;
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

    view! {
        <aside
            class="object-inspector"
            data-corner=corner_name(corner)
            data-testid="object-inspector"
            style=style
        >
            <h3 class="inspector-name">{name}</h3>
            <dl class="inspector-meta">
                <dt>"Category"</dt>
                <dd>{category}</dd>
                <dt>"Footprint"</dt>
                <dd>{footprint}</dd>
                <dt>"Height"</dt>
                <dd>{height}</dd>
                <dt>"Unit price"</dt>
                <dd>{price}</dd>
                <dt>"Position"</dt>
                <dd>{position}</dd>
                <dt>"Rotation"</dt>
                <dd>
                    {rotation}
                    <button
                        class="inspector-reset"
                        data-testid="reset-rotation"
                        on:click=move |_| on_reset_rotation.run(())
                    >
                        "Reset"
                    </button>
                </dd>
            </dl>
            <div class="inspector-status" data-testid="inspector-status">
                {status_btn(ItemStatus::planned, "Planned", "status-planned")}
                {status_btn(ItemStatus::existing, "Existing", "status-existing")}
                {status_btn(ItemStatus::r#virtual, "Virtual", "status-virtual")}
            </div>
        </aside>
    }
}

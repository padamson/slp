//! The object palette: catalog items as click-to-place tiles, **grouped by
//! category**. Clicking a tile arms placement of that item (click the armed
//! tile again to disarm) — replacing the old catalog dropdown + "Place" button
//! with one click, and scaling to many object categories (furniture, fire pits,
//! trees, …). Each tile's mini-icon is drawn from the shared [`crate::style`]
//! palette, in the item's footprint shape, so it reads like what lands on the
//! canvas.

use leptos::prelude::*;
use slp_core::{CatalogItem, FootprintShape};

use crate::style::{FURNITURE_FILL, FURNITURE_STROKE};

#[component]
pub fn ObjectPalette(
    /// The catalog to place from.
    #[prop(into)]
    catalog: Signal<Vec<CatalogItem>>,
    /// The armed item's id, if the object tool is active (`None` = nothing
    /// armed). The matching tile renders highlighted.
    #[prop(into)]
    armed: Signal<Option<String>>,
    /// Arm (or, if already armed, toggle off) the item with this id.
    on_pick: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="object-palette" data-testid="object-palette">
            {move || {
                let armed_id = armed.get();
                group_by_category(&catalog.get())
                    .into_iter()
                    .map(|(category, group)| {
                        let tiles = group
                            .into_iter()
                            .map(|item| tile(item, armed_id.clone(), on_pick))
                            .collect::<Vec<_>>();
                        view! {
                            <div class="palette-group">
                                <div class="palette-group-label">{humanize(&category)}</div>
                                <div class="palette-tiles">{tiles}</div>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}

/// Group catalog items by `category` (absent → "other"), preserving each
/// category's first-seen order and item order within it.
fn group_by_category(items: &[CatalogItem]) -> Vec<(String, Vec<CatalogItem>)> {
    let mut groups: Vec<(String, Vec<CatalogItem>)> = Vec::new();
    for item in items {
        let cat = item.category.clone().unwrap_or_else(|| "other".to_string());
        if let Some(g) = groups.iter_mut().find(|(c, _)| *c == cat) {
            g.1.push(item.clone());
        } else {
            groups.push((cat, vec![item.clone()]));
        }
    }
    groups
}

/// A category id → a display label: `"fire-pit"` → `"Fire pit"`.
fn humanize(s: &str) -> String {
    let spaced = s.replace(['-', '_'], " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => spaced,
    }
}

/// One catalog item as a tile: a shape-matched mini-icon, name, and price.
#[allow(clippy::needless_pass_by_value)]
fn tile(item: CatalogItem, armed_id: Option<String>, on_pick: Callback<String>) -> impl IntoView {
    let id = item.id.clone();
    let is_armed = armed_id.as_deref() == Some(id.as_str());
    let name = item.name.clone().unwrap_or_else(|| id.clone());
    let price = item
        .unit_price
        .map_or_else(String::new, |p| format!("${p:.0}"));
    let circle = item.shape == FootprintShape::circle;
    let testid = format!("palette-{id}");
    let pick_id = id.clone();
    // A shape-matched mini-icon (16×16 px chrome), so a round item reads round.
    let icon = if circle {
        view! {
            <circle
                cx="8"
                cy="8"
                r="6"
                fill=FURNITURE_FILL
                fill-opacity="0.7"
                stroke=FURNITURE_STROKE
                stroke-width="1"
            />
        }
        .into_any()
    } else {
        view! {
            <rect
                x="2"
                y="4"
                width="12"
                height="8"
                fill=FURNITURE_FILL
                fill-opacity="0.7"
                stroke=FURNITURE_STROKE
                stroke-width="1"
            />
        }
        .into_any()
    };
    view! {
        <button
            class="palette-tile"
            class:armed=is_armed
            data-testid=testid
            on:click=move |_| on_pick.run(pick_id.clone())
        >
            <svg class="palette-icon" viewBox="0 0 16 16" width="16" height="16">
                {icon}
            </svg>
            <span class="palette-name">{name}</span>
            <span class="palette-price">{price}</span>
        </button>
    }
}

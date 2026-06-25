//! The deck/patio: one or more levels (footprints at an elevation), rendered to
//! scale as stacked polygons with corner markers and an elevation label. This
//! draws the *committed* deck from the `Plan`; the in-progress outline being
//! drawn is the `Placement` overlay. Stairs land in the next slice.

use leptos::prelude::*;
use slp_core::DeckLevel;

use super::Transform;

#[component]
pub fn Deck(t: Transform, levels: Vec<DeckLevel>) -> impl IntoView {
    // Lowest first so higher platforms paint on top.
    let mut levels = levels;
    levels.sort_by(|a, b| a.elevation.total_cmp(&b.elevation));
    let tiers = levels
        .into_iter()
        .filter(|lvl| lvl.corners.len() >= 2)
        .map(|lvl| level_view(t, lvl))
        .collect::<Vec<_>>();
    (!tiers.is_empty()).then(|| view! { <g class="deck">{tiers}</g> })
}

/// One platform: its footprint polygon, corner markers, and an elevation label.
fn level_view(t: Transform, lvl: DeckLevel) -> impl IntoView {
    let DeckLevel { corners, elevation } = lvl;
    let points = corners
        .iter()
        .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
        .collect::<Vec<_>>()
        .join(" ");
    let markers = corners
        .iter()
        .map(|c| view! { <circle class="deck-corner" cx=t.sx(c.x) cy=t.sy(c.y) r="3" fill="#8a6f4f" /> })
        .collect::<Vec<_>>();
    // Label at the footprint centroid.
    let n = f64::from(u32::try_from(corners.len()).unwrap_or(1).max(1));
    let cx = corners.iter().map(|c| t.sx(c.x)).sum::<f64>() / n;
    let cy = corners.iter().map(|c| t.sy(c.y)).sum::<f64>() / n;
    let label = format!("+{elevation:.1} ft");
    view! {
        <g class="deck-level">
            <polygon
                points=points
                fill="#c8a97e"
                fill-opacity="0.55"
                stroke="#8a6f4f"
                stroke-width="2"
            />
            {markers}
            <text
                class="deck-label"
                x=cx
                y=cy
                text-anchor="middle"
                font-size="11"
                fill="#5a4a33"
            >
                {label}
            </text>
        </g>
    }
}

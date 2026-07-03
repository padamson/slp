//! The deck/patio: one or more levels (footprints at an elevation) plus step
//! runs, rendered to scale. Levels are stacked polygons with corner markers and
//! an elevation label; each step run is composed via the `Steps` component
//! (running away from the deck centroid). This draws the *committed* deck from
//! the `Plan`; the in-progress outline being drawn is the `Placement` overlay.

use leptos::prelude::*;
use slp_core::{Coord, DeckLevel, StepRun};

use super::{Steps, Transform};
use crate::style::{DECK_FILL, DECK_FILL_OPACITY, DECK_STROKE};

#[component]
pub fn Deck(
    t: Transform,
    levels: Vec<DeckLevel>,
    /// Step runs on the deck's edges.
    #[prop(optional)]
    steps: Vec<StepRun>,
) -> impl IntoView {
    let mut levels = levels;
    // Lowest first so higher platforms paint on top.
    levels.sort_by(|a, b| a.elevation.total_cmp(&b.elevation));
    let centroid = deck_centroid(&levels);
    let tiers = levels
        .into_iter()
        .filter(|lvl| lvl.corners.len() >= 2)
        .map(|lvl| level_view(t, lvl))
        .collect::<Vec<_>>();
    let runs = steps
        .into_iter()
        .map(|run| view! { <Steps t=t run=run away_from=centroid.clone() /> })
        .collect::<Vec<_>>();
    (!tiers.is_empty() || !runs.is_empty()).then(|| {
        view! {
            <g class="deck">
                {tiers}
                {runs}
            </g>
        }
    })
}

/// Average of every level corner — the deck's centroid (steps run away from it).
fn deck_centroid(levels: &[DeckLevel]) -> Coord {
    let pts: Vec<&Coord> = levels.iter().flat_map(|l| l.corners.iter()).collect();
    let n = f64::from(u32::try_from(pts.len()).unwrap_or(1).max(1));
    Coord::new(
        pts.iter().map(|c| c.x).sum::<f64>() / n,
        pts.iter().map(|c| c.y).sum::<f64>() / n,
    )
}

/// One platform: its footprint polygon, corner markers, and an elevation label.
fn level_view(t: Transform, lvl: DeckLevel) -> impl IntoView {
    let DeckLevel {
        corners, elevation, ..
    } = lvl;
    let points = corners
        .iter()
        .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
        .collect::<Vec<_>>()
        .join(" ");
    let markers = corners
        .iter()
        .map(|c| view! { <circle class="deck-corner" cx=t.sx(c.x) cy=t.sy(c.y) r="3" fill=DECK_STROKE /> })
        .collect::<Vec<_>>();
    let n = f64::from(u32::try_from(corners.len()).unwrap_or(1).max(1));
    let cx = corners.iter().map(|c| t.sx(c.x)).sum::<f64>() / n;
    let cy = corners.iter().map(|c| t.sy(c.y)).sum::<f64>() / n;
    let label = format!("+{elevation:.1} ft");
    view! {
        <g class="deck-level">
            <polygon
                points=points
                fill=DECK_FILL
                fill-opacity=DECK_FILL_OPACITY
                stroke=DECK_STROKE
                stroke-width="2"
            />
            {markers}
            <text class="deck-label" x=cx y=cy text-anchor="middle" font-size="11" fill="#5a4a33">
                {label}
            </text>
        </g>
    }
}

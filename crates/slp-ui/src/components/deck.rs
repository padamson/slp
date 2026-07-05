//! The deck/patio: one or more levels (footprints at an elevation) plus step
//! runs, rendered to scale. Levels are stacked polygons with corner markers and
//! an elevation label; each step run is composed via the `Steps` component
//! (running away from the deck centroid). This draws the *committed* deck from
//! the `Plan`; the in-progress outline being drawn is the `Placement` overlay.
//!
//! A selected level's corners render as larger, interactive **node handles**
//! (mirrors a selected drawn area/house): press one to start a move drag.
//! Selection is by each level's *original* index (its position in the
//! `levels` prop as passed in), not its position after the paint-order sort
//! below — so the caller can address a level stably regardless of how it
//! currently sorts by elevation. Node insert/delete isn't offered here yet —
//! see `F1-select-move-delete.md`.

use leptos::prelude::*;
use slp_core::{Coord, DeckLevel, StepRun};

use super::{Steps, Transform};
use crate::style::{DECK_FILL, DECK_FILL_OPACITY, DECK_STROKE, SELECTED_STROKE};

/// A selected level's node-handle radius (px) — bigger than the plain corner
/// marker so it reads as a drag target.
const NODE_HANDLE_R: f64 = 5.0;

#[component]
pub fn Deck(
    t: Transform,
    levels: Vec<DeckLevel>,
    /// Step runs on the deck's edges.
    #[prop(optional)]
    steps: Vec<StepRun>,
    /// The selected level's original index (into the `levels` prop as passed
    /// in, before the paint-order sort), if any.
    #[prop(default = None)]
    selected: Option<usize>,
    /// A level's filled body was pressed (by its original index) — select it.
    #[prop(default = None)]
    on_level_press: Option<Callback<usize>>,
    /// A selected level's node handle was pressed (by corner index) — start a
    /// move drag.
    #[prop(default = None)]
    on_node_press: Option<Callback<usize>>,
) -> impl IntoView {
    // Pair each level with its original index before sorting, so paint order
    // (lowest first, so higher platforms paint on top) doesn't disturb which
    // index a caller's `selected`/press-callback addresses.
    let mut indexed: Vec<(usize, DeckLevel)> = levels.into_iter().enumerate().collect();
    indexed.sort_by(|a, b| a.1.elevation.total_cmp(&b.1.elevation));
    let all_corners: Vec<Coord> = indexed
        .iter()
        .flat_map(|(_, l)| l.corners.iter().cloned())
        .collect();
    let centroid = deck_centroid(&all_corners);
    let tiers = indexed
        .into_iter()
        .filter(|(_, lvl)| lvl.corners.len() >= 2)
        .map(|(i, lvl)| {
            let is_selected = selected == Some(i);
            level_view(t, lvl, i, is_selected, on_level_press, on_node_press)
        })
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
fn deck_centroid(corners: &[Coord]) -> Coord {
    let n = f64::from(u32::try_from(corners.len()).unwrap_or(1).max(1));
    Coord::new(
        corners.iter().map(|c| c.x).sum::<f64>() / n,
        corners.iter().map(|c| c.y).sum::<f64>() / n,
    )
}

/// One platform: its footprint polygon, corner markers (or, when selected,
/// interactive node handles), and an elevation label.
///
/// `lvl` is a by-value prop-like passthrough (matching `Furnishings`'s
/// `object_view`): Edition 2024's RPIT lifetime-capture rules mean a borrow
/// here would tie the returned `impl IntoView` to that borrow, which the
/// caller (a short-lived local in the iterator closure above) can't satisfy.
#[allow(clippy::too_many_arguments)]
fn level_view(
    t: Transform,
    lvl: DeckLevel,
    i: usize,
    is_selected: bool,
    on_level_press: Option<Callback<usize>>,
    on_node_press: Option<Callback<usize>>,
) -> impl IntoView {
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
        .enumerate()
        .map(|(ni, c)| {
            let (cx, cy) = (t.sx(c.x), t.sy(c.y));
            if is_selected {
                view! {
                    <circle
                        class="deck-node"
                        data-testid="deck-node"
                        cx=cx
                        cy=cy
                        r=NODE_HANDLE_R
                        fill=SELECTED_STROKE
                        on:mousedown=move |ev: leptos::ev::MouseEvent| {
                            ev.stop_propagation();
                            if let Some(cb) = on_node_press {
                                cb.run(ni);
                            }
                        }
                    />
                }
                .into_any()
            } else {
                view! { <circle class="deck-corner" cx=cx cy=cy r="3" fill=DECK_STROKE /> }
                    .into_any()
            }
        })
        .collect::<Vec<_>>();
    let n = f64::from(u32::try_from(corners.len()).unwrap_or(1).max(1));
    let cx = corners.iter().map(|c| t.sx(c.x)).sum::<f64>() / n;
    let cy = corners.iter().map(|c| t.sy(c.y)).sum::<f64>() / n;
    let label = format!("{elevation:+.1} ft");
    let mut class = String::from("deck-level");
    if is_selected {
        class.push_str(" deck-level--selected");
    }
    view! {
        <g
            class=class
            on:mousedown=move |_ev: leptos::ev::MouseEvent| {
                if let Some(cb) = on_level_press {
                    cb.run(i);
                }
            }
        >
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

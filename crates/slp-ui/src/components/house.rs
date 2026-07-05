//! The house outline: the user-drawn footprint, rendered to scale inside the
//! `Yard` as a closed SVG polygon (the walls) with a marker at each corner. The
//! house *composes* a `Wall` per edge, and each `Wall` composes its `Door`/
//! `Window` openings — so `house = outline + walls(+ doors + windows)`. This
//! draws the *committed* plan; the in-progress outline being drawn is the
//! `Placement` overlay. Nothing here is hardcoded — it all comes from the `Plan`.
//!
//! A selected house's corners render as larger, interactive **node handles**
//! (mirrors a selected drawn area/tree): press one to start a move drag. A
//! moved corner just changes wall geometry — each `Wall`/opening below already
//! reads its position live from the corners + its wall index each render, so
//! doors/windows follow without any extra bookkeeping. Node insert/delete
//! isn't offered here yet: it would renumber wall indices and could silently
//! reassign an opening to the wrong wall — see `F1-select-move-delete.md`.

use leptos::prelude::*;
use slp_core::{Coord, Opening};

use super::{Transform, Wall};
use crate::style::{HOUSE_FILL, HOUSE_FILL_OPACITY, HOUSE_STROKE, SELECTED_STROKE};

/// A selected house's node-handle radius (px) — bigger than the plain corner
/// marker so it reads as a drag target.
const NODE_HANDLE_R: f64 = 5.0;

#[component]
pub fn House(
    t: Transform,
    corners: Vec<Coord>,
    /// Doors and windows, each keyed to a wall (edge) by index.
    #[prop(optional)]
    openings: Vec<Opening>,
    /// Whether the house is selected — its corners become interactive node
    /// handles instead of plain markers.
    #[prop(default = false)]
    selected: bool,
    /// The house's body was pressed — select it.
    #[prop(default = None)]
    on_house_press: Option<Callback<()>>,
    /// A selected house's node handle was pressed (by corner index) — start a
    /// move drag.
    #[prop(default = None)]
    on_node_press: Option<Callback<usize>>,
) -> impl IntoView {
    render_house(
        t,
        corners,
        openings,
        selected,
        on_house_press,
        on_node_press,
    )
}

// The body lives in a plain fn (not the `#[component]`) so the line count —
// dominated by expanded `view!` macros for a composition root — can be allowed.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn render_house(
    t: Transform,
    corners: Vec<Coord>,
    openings: Vec<Opening>,
    selected: bool,
    on_house_press: Option<Callback<()>>,
    on_node_press: Option<Callback<usize>>,
) -> impl IntoView {
    // Nothing to draw → render nothing (no empty group/outline).
    (!corners.is_empty()).then(move || {
        let markers = corners
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let (cx, cy) = (t.sx(c.x), t.sy(c.y));
                if selected {
                    view! {
                        <circle
                            class="house-node"
                            data-testid="house-node"
                            cx=cx
                            cy=cy
                            r=NODE_HANDLE_R
                            fill=SELECTED_STROKE
                            on:mousedown=move |ev: leptos::ev::MouseEvent| {
                                ev.stop_propagation();
                                if let Some(cb) = on_node_press {
                                    cb.run(i);
                                }
                            }
                        />
                    }
                    .into_any()
                } else {
                    view! { <circle class="house-corner" cx=cx cy=cy r="3" fill=HOUSE_STROKE /> }
                        .into_any()
                }
            })
            .collect::<Vec<_>>();

        // The footprint fill (the walls themselves are drawn by the Wall
        // children, so this is fill-only — no stroke — to avoid double edges).
        let outline = (corners.len() >= 2).then(|| {
            let points = corners
                .iter()
                .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
                .collect::<Vec<_>>()
                .join(" ");
            view! { <polygon points=points fill=HOUSE_FILL fill-opacity=HOUSE_FILL_OPACITY stroke="none" /> }
        });

        // One Wall per edge. While drawing it's an open chain; once there are
        // three corners it's a closed ring (the last edge wraps to corner 0).
        let walls = (corners.len() >= 2).then(|| {
            let n = corners.len();
            let count = if n >= 3 { n } else { n - 1 };
            (0..count)
                .map(|i| {
                    let start = corners[i].clone();
                    let end = corners[(i + 1) % n].clone();
                    let on_wall: Vec<Opening> = openings
                        .iter()
                        .filter(|o| usize::try_from(o.wall).is_ok_and(|w| w == i))
                        .cloned()
                        .collect();
                    view! { <Wall t=t start=start end=end openings=on_wall /> }
                })
                .collect::<Vec<_>>()
        });

        let mut class = String::from("house");
        if selected {
            class.push_str(" house--selected");
        }

        view! {
            <g
                class=class
                on:mousedown=move |_ev: leptos::ev::MouseEvent| {
                    if let Some(cb) = on_house_press {
                        cb.run(());
                    }
                }
            >
                {outline}
                {walls}
                {markers}
            </g>
        }
    })
}

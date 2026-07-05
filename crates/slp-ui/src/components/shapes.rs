//! Drawn areas (paver patios, mulch beds, …): straight-edged closed outlines
//! at a given elevation, rendered filled with corner markers and an area +
//! elevation label — the area equivalent of `Furnishings`. This draws the
//! *committed* shapes from the `Plan`; the in-progress outline being drawn is
//! the `Placement` overlay. Category-specific look (paver vs. mulch) lands
//! with whichever story first needs it — every shape looks the same for now.
//!
//! A selected shape's corners render as larger, interactive **node handles**
//! (the same "press to start a drag" gesture a tree's canopy/trunk handles
//! use) instead of the plain markers: press one to select it and start a
//! move drag, press an adjacent second one to arm the **insert-between**
//! popup (Insert/Cancel, floating near their midpoint). All selection/drag
//! state is owned by the caller — this component only renders it and reports
//! presses.

use leptos::prelude::*;
use slp_core::{Point, Shape, area};

use super::Transform;
use crate::style::{SELECTED_FILL, SELECTED_STROKE, SHAPE_FILL, SHAPE_FILL_OPACITY, SHAPE_STROKE};

/// A selected shape's node-handle radius (px) — bigger than the plain corner
/// marker so it reads as a drag target.
const NODE_HANDLE_R: f64 = 5.0;

// `selected_nodes` is only ever cloned (once, for the selected shape), never
// moved-from — but it's a plain owned prop like every other `Vec` prop here,
// not worth a `&[usize]` + lifetime just to satisfy the lint.
#[allow(clippy::needless_pass_by_value)]
#[component]
pub fn Shapes(
    t: Transform,
    shapes: Vec<Shape>,
    /// The index (into `shapes`) of the currently selected shape, if any — its
    /// corners render as interactive node handles instead of plain markers.
    #[prop(default = None)]
    selected: Option<usize>,
    /// Indices (into the selected shape's corners) of the currently selected
    /// nodes: empty (none picked), one (a move/insert-pair start), or an
    /// adjacent pair (showing the insert-between popup).
    #[prop(optional)]
    selected_nodes: Vec<usize>,
    /// A shape's filled body was pressed (by `shapes` index) — select it.
    #[prop(default = None)]
    on_shape_press: Option<Callback<usize>>,
    /// A selected shape's node handle was pressed (by corner index) — select
    /// it and start a move drag.
    #[prop(default = None)]
    on_node_press: Option<Callback<usize>>,
    /// The insert-between popup's "Insert" button was pressed.
    #[prop(default = None)]
    on_insert_node: Option<Callback<()>>,
    /// The insert-between popup's "Cancel" button was pressed.
    #[prop(default = None)]
    on_cancel_nodes: Option<Callback<()>>,
) -> impl IntoView {
    let areas = shapes
        .into_iter()
        .enumerate()
        .filter(|(_, s)| s.corners.len() >= 3)
        .map(|(i, s)| {
            let is_selected = selected == Some(i);
            let nodes = if is_selected {
                selected_nodes.clone()
            } else {
                Vec::new()
            };
            shape_view(
                t,
                s,
                i,
                is_selected,
                nodes,
                on_shape_press,
                on_node_press,
                on_insert_node,
                on_cancel_nodes,
            )
        })
        .collect::<Vec<_>>();
    (!areas.is_empty()).then(|| {
        view! { <g class="shapes">{areas}</g> }
    })
}

/// One drawn area: its filled polygon, corner markers (or, when selected,
/// interactive node handles plus an insert-between popup), and an area (ft²)
/// + elevation label at its centroid.
///
/// `shape`/`selected_nodes` are by-value prop-like passthroughs (matching
/// `Furnishings`'s `object_view`): Edition 2024's RPIT lifetime-capture rules
/// mean a borrow here would tie the returned `impl IntoView` to that borrow,
/// which the caller (a short-lived local in the iterator closure above) can't
/// satisfy.
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::many_single_char_names,
    clippy::needless_pass_by_value
)]
fn shape_view(
    t: Transform,
    shape: Shape,
    i: usize,
    is_selected: bool,
    selected_nodes: Vec<usize>,
    on_shape_press: Option<Callback<usize>>,
    on_node_press: Option<Callback<usize>>,
    on_insert_node: Option<Callback<()>>,
    on_cancel_nodes: Option<Callback<()>>,
) -> impl IntoView {
    let Shape { corners, elevation } = shape;
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
                let node_selected = selected_nodes.contains(&ni);
                let fill = if node_selected {
                    SELECTED_STROKE
                } else {
                    SHAPE_STROKE
                };
                view! {
                    <circle
                        class="shape-node"
                        data-testid="shape-node"
                        cx=cx
                        cy=cy
                        r=NODE_HANDLE_R
                        fill=fill
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
                view! { <circle class="shape-corner" cx=cx cy=cy r="3" fill=SHAPE_STROKE /> }
                    .into_any()
            }
        })
        .collect::<Vec<_>>();
    let n = f64::from(u32::try_from(corners.len()).unwrap_or(1).max(1));
    let cx = corners.iter().map(|c| t.sx(c.x)).sum::<f64>() / n;
    let cy = corners.iter().map(|c| t.sy(c.y)).sum::<f64>() / n;
    let pts: Vec<Point> = corners.iter().map(|c| Point::new(c.x, c.y)).collect();
    let ft2 = area(&pts);
    let label = if elevation == 0.0 {
        format!("{ft2:.0} ft²")
    } else {
        format!("{ft2:.0} ft² · {elevation:+.1} ft")
    };
    let fill = if is_selected {
        SELECTED_FILL
    } else {
        SHAPE_FILL
    };
    let stroke = if is_selected {
        SELECTED_STROKE
    } else {
        SHAPE_STROKE
    };
    let mut class = String::from("shape");
    if is_selected {
        class.push_str(" shape--selected");
    }
    // The insert-between popup appears once an adjacent pair of nodes is
    // selected, floating near their midpoint.
    let popup = (selected_nodes.len() == 2).then(|| {
        let (mx, my) = match (corners.get(selected_nodes[0]), corners.get(selected_nodes[1])) {
            (Some(a), Some(b)) => (
                f64::midpoint(t.sx(a.x), t.sx(b.x)),
                f64::midpoint(t.sy(a.y), t.sy(b.y)),
            ),
            _ => (cx, cy),
        };
        view! {
            <g class="node-insert-popup" transform=format!("translate({mx},{my})")>
                <g
                    data-testid="insert-node"
                    on:mousedown=move |ev: leptos::ev::MouseEvent| {
                        ev.stop_propagation();
                        if let Some(cb) = on_insert_node {
                            cb.run(());
                        }
                    }
                >
                    <rect x="-34" y="-11" width="34" height="22" rx="3" fill="#fff" stroke=SELECTED_STROKE />
                    <text x="-17" y="4" text-anchor="middle" font-size="10" fill=SELECTED_STROKE>
                        "Insert"
                    </text>
                </g>
                <g
                    data-testid="cancel-node-select"
                    on:mousedown=move |ev: leptos::ev::MouseEvent| {
                        ev.stop_propagation();
                        if let Some(cb) = on_cancel_nodes {
                            cb.run(());
                        }
                    }
                >
                    <rect x="0" y="-11" width="34" height="22" rx="3" fill="#fff" stroke=SHAPE_STROKE />
                    <text x="17" y="4" text-anchor="middle" font-size="10" fill=SHAPE_STROKE>
                        "Cancel"
                    </text>
                </g>
            </g>
        }
    });
    view! {
        <g
            class=class
            on:mousedown=move |_ev: leptos::ev::MouseEvent| {
                if let Some(cb) = on_shape_press {
                    cb.run(i);
                }
            }
        >
            <polygon
                points=points
                fill=fill
                fill-opacity=SHAPE_FILL_OPACITY
                stroke=stroke
                stroke-width="2"
            />
            {markers}
            <text class="shape-label" x=cx y=cy text-anchor="middle" font-size="11" fill="#5a5540">
                {label}
            </text>
            {popup}
        </g>
    }
}

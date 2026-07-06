//! A standalone circular drawn area (a round paver patio or mulch bed):
//! center + radius at an elevation, rendered filled with an area + size +
//! elevation label — the circle equivalent of `Shapes`. This draws the
//! *committed* circles from the `Plan`; the in-progress circle being drawn is
//! the `Placement` overlay.
//!
//! A selected circle shows one resize handle on its edge (due east), reusing
//! the same "drag toward/away from center" gesture a tree's canopy/trunk
//! handles already use (`furnishings.rs`) — dragging it changes the radius
//! live. Its size reads as a diameter (⌀), matching how a round object's does.

use leptos::prelude::*;
use slp_core::{Circle, circle_area};

use super::Transform;
use crate::style::{SELECTED_FILL, SELECTED_STROKE, SHAPE_FILL, SHAPE_FILL_OPACITY, SHAPE_STROKE};

/// A selected circle's resize-handle radius (px).
const HANDLE_R: f64 = 5.0;

#[component]
pub fn Circles(
    t: Transform,
    circles: Vec<Circle>,
    /// The index (into `circles`) of the currently selected circle, if any.
    #[prop(default = None)]
    selected: Option<usize>,
    /// A circle's filled body was pressed (by `circles` index) — select it.
    #[prop(default = None)]
    on_circle_press: Option<Callback<usize>>,
    /// A selected circle's resize handle was pressed — start a resize drag.
    #[prop(default = None)]
    on_handle_press: Option<Callback<()>>,
) -> impl IntoView {
    let items = circles
        .into_iter()
        .enumerate()
        .map(|(i, c)| {
            let is_selected = selected == Some(i);
            circle_view(t, c, i, is_selected, on_circle_press, on_handle_press)
        })
        .collect::<Vec<_>>();
    (!items.is_empty()).then(|| {
        view! { <g class="circles">{items}</g> }
    })
}

/// One drawn circle: its filled disk, an area (ft²) + diameter + (when
/// non-zero) elevation label at its center, and — when selected — a resize
/// handle on its edge.
///
/// `circle` is a by-value prop-like passthrough (matching `Furnishings`'s
/// `object_view`): Edition 2024's RPIT lifetime-capture rules mean a borrow
/// here would tie the returned `impl IntoView` to that borrow, which the
/// caller (a short-lived local in the iterator closure above) can't satisfy.
#[allow(clippy::too_many_arguments)]
fn circle_view(
    t: Transform,
    circle: Circle,
    i: usize,
    is_selected: bool,
    on_circle_press: Option<Callback<usize>>,
    on_handle_press: Option<Callback<()>>,
) -> impl IntoView {
    let Circle {
        center,
        elevation,
        radius_ft,
    } = circle;
    let (cx, cy) = (t.sx(center.x), t.sy(center.y));
    let r_px = radius_ft * t.px_ft;
    let ft2 = circle_area(radius_ft);
    let diameter = radius_ft * 2.0;
    let label = if elevation == 0.0 {
        format!("{ft2:.0} ft² · ⌀{diameter:.0} ft")
    } else {
        format!("{ft2:.0} ft² · ⌀{diameter:.0} ft · {elevation:+.1} ft")
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
    let mut class = String::from("circle-area");
    if is_selected {
        class.push_str(" circle-area--selected");
    }
    let handle = is_selected.then(|| {
        view! {
            <circle
                class="circle-resize-handle"
                data-testid="circle-resize-handle"
                cx=cx + r_px
                cy=cy
                r=HANDLE_R
                fill=SELECTED_STROKE
                on:mousedown=move |ev: leptos::ev::MouseEvent| {
                    ev.stop_propagation();
                    if let Some(cb) = on_handle_press {
                        cb.run(());
                    }
                }
            />
        }
    });
    view! {
        <g
            class=class
            on:mousedown=move |_ev: leptos::ev::MouseEvent| {
                if let Some(cb) = on_circle_press {
                    cb.run(i);
                }
            }
        >
            <circle
                cx=cx
                cy=cy
                r=r_px
                fill=fill
                fill-opacity=SHAPE_FILL_OPACITY
                stroke=stroke
                stroke-width="2"
            />
            <text class="circle-label" x=cx y=cy text-anchor="middle" font-size="11" fill="#5a5540">
                {label}
            </text>
            {handle}
        </g>
    }
}

//! Drawn areas (paver patios, mulch beds, …): straight-edged closed outlines
//! at a given elevation, rendered filled with corner markers and an area +
//! elevation label — the area equivalent of `Furnishings`. This draws the
//! *committed* shapes from the `Plan`; the in-progress outline being drawn is
//! the `Placement` overlay. Category-specific look (paver vs. mulch) lands
//! with whichever story first needs it — every shape looks the same for now.

use leptos::prelude::*;
use slp_core::{Point, Shape, area};

use super::Transform;
use crate::style::{SHAPE_FILL, SHAPE_FILL_OPACITY, SHAPE_STROKE};

#[component]
pub fn Shapes(t: Transform, shapes: Vec<Shape>) -> impl IntoView {
    let areas = shapes
        .into_iter()
        .filter(|s| s.corners.len() >= 3)
        .map(|s| shape_view(t, s))
        .collect::<Vec<_>>();
    (!areas.is_empty()).then(|| {
        view! { <g class="shapes">{areas}</g> }
    })
}

/// One drawn area: its filled polygon, corner markers, and an
/// area (ft²) + elevation label at its centroid.
fn shape_view(t: Transform, shape: Shape) -> impl IntoView {
    let Shape { corners, elevation } = shape;
    let points = corners
        .iter()
        .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
        .collect::<Vec<_>>()
        .join(" ");
    let markers = corners
        .iter()
        .map(|c| view! { <circle class="shape-corner" cx=t.sx(c.x) cy=t.sy(c.y) r="3" fill=SHAPE_STROKE /> })
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
    view! {
        <g class="shape">
            <polygon
                points=points
                fill=SHAPE_FILL
                fill-opacity=SHAPE_FILL_OPACITY
                stroke=SHAPE_STROKE
                stroke-width="2"
            />
            {markers}
            <text class="shape-label" x=cx y=cy text-anchor="middle" font-size="11" fill="#5a5540">
                {label}
            </text>
        </g>
    }
}

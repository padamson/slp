//! theoria story for `Yard`. Compiled only under the `stories` feature.
//!
//! Uses the `#[story]` macro: the typed params become live controls (drag the
//! width/depth and the canvas reflows), and the body is captured for "show code".

use leptos::prelude::*;
use theoria::{Story, story};

use super::Yard;

/// The to-scale base canvas the rest of the plan draws onto (house, deck, beds).
/// One grid square is `1 ft`, so set the size to your lot's real dimensions
/// before placing anything; everything else snaps to this grid.
///
/// - **width** — the lot's east-west extent, in feet.
/// - **depth** — its north-south extent, in feet.
///
/// Drag either control and the canvas reflows to scale.
#[story(name = "Canvas/Yard", yard_w = 70.0, yard_d = 30.0)]
fn yard(yard_w: f64, yard_d: f64) -> impl IntoView {
    view! { <Yard yard_w=yard_w yard_d=yard_d px_ft=12.0 pad=40.0 /> }
}

pub fn stories() -> Vec<Story> {
    vec![yard()]
}

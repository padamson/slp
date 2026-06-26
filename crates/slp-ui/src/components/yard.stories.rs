//! theoria story for `Yard`. Compiled only under the `stories` feature.
//!
//! Uses the `#[story]` macro: the typed params become live controls (drag the
//! width/depth and the canvas reflows), and the body is captured for "show code".

use leptos::prelude::*;
use theoria::{Story, story};

use super::Yard;

/// The to-scale yard canvas. Adjust **width** and **depth** to resize it.
#[story(name = "Canvas/Yard", yard_w = 70.0, yard_d = 30.0)]
fn yard(yard_w: f64, yard_d: f64) -> impl IntoView {
    view! { <Yard yard_w=yard_w yard_d=yard_d px_ft=12.0 pad=40.0 /> }
}

pub fn stories() -> Vec<Story> {
    vec![yard()]
}

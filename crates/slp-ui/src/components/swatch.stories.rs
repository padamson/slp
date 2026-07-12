//! theoria stories for `MaterialSwatch`. Compiled only under the `stories`
//! feature.

use leptos::prelude::*;

use super::MaterialSwatch;
use theoria::Story;

/// An 8×8 gray checkerboard PNG (a stand-in paver photo) that visibly reads as
/// a thumbnail.
const TILE_PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAIAAABLbSncAAAAHElEQVR4nGOYNWcBHFVU1cERAxUlkDnIiqgoAQDsoGjB+2xT8QAAAABJRU5ErkJggg==";

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Controls/Material swatch · photo", || {
            view! {
                <MaterialSwatch
                    image=Some(TILE_PNG.to_string())
                    category=Some("paver".to_string())
                />
            }
        }),
        Story::new("Controls/Material swatch · paver color", || {
            view! { <MaterialSwatch category=Some("paver".to_string()) /> }
        }),
        Story::new("Controls/Material swatch · mulch color", || {
            view! { <MaterialSwatch category=Some("mulch-bed".to_string()) /> }
        }),
        Story::new("Controls/Material swatch · default color", || {
            view! { <MaterialSwatch /> }
        }),
    ]
}

//! theoria story for `Legend`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::Legend;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Canvas/Legend", || {
        view! {
            <svg viewBox="0 0 560 60" width="560">
                <Legend start_x=10.0 baseline_y=40.0 />
            </svg>
        }
    })]
}

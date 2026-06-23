//! theoria story for `Door`. Compiled only under the `stories` feature. Shown on
//! a wall segment so the gap + leaf + swing read in context.

use leptos::prelude::*;

use super::Door;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Door", || {
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 160 120" width="160">
                <line x1=20.0 y1=70.0 x2=140.0 y2=70.0 stroke="#8a7f6a" stroke-width="2" />
                <Door x1=55.0 y1=70.0 x2=95.0 y2=70.0 />
            </svg>
        }
    })]
}

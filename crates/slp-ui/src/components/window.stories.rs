//! theoria story for `Window`. Compiled only under the `stories` feature. Shown
//! on a wall segment so the gap + glass read in context.

use leptos::prelude::*;

use super::Window;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Window", || {
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 160 120" width="160">
                <line x1=20.0 y1=70.0 x2=140.0 y2=70.0 stroke="#8a7f6a" stroke-width="2" />
                <Window x1=55.0 y1=70.0 x2=105.0 y2=70.0 />
            </svg>
        }
    })]
}

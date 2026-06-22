//! theoria stories for `YardControls`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::YardControls;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("YardControls", || {
        let (width, set_width) = signal(70.0_f64);
        let (depth, set_depth) = signal(30.0_f64);
        view! { <YardControls width=width set_width=set_width depth=depth set_depth=set_depth /> }
    })]
}

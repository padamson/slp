//! theoria stories for `Yard`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::Yard;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Canvas/Yard", || {
        view! { <Yard yard_w=70.0 yard_d=30.0 px_ft=12.0 pad=40.0 /> }
    })]
}

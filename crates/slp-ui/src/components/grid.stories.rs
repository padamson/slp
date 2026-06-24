//! theoria stories for `Grid`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::{Grid, Transform};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    let t = Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 12.0,
    };
    vec![Story::new("Canvas/Grid", move || {
        view! {
            <svg viewBox="0 0 320 220" width="340">
                <Grid t=t yard_w=20.0 yard_d=12.0 />
            </svg>
        }
    })]
}

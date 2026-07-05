//! theoria stories for `ScaleBar`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::{ScaleBar, Transform};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    let t = Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 12.0,
    };
    vec![
        Story::new("Canvas/Scale bar", move || {
            view! {
                <svg viewBox="0 0 220 60" width="240">
                    <ScaleBar t=t baseline_y=40.0 />
                </svg>
            }
        }),
        Story::new("Canvas/Scale bar (20 ft)", move || {
            view! {
                <svg viewBox="0 0 320 60" width="340">
                    <ScaleBar t=t baseline_y=40.0 length_ft=20.0 />
                </svg>
            }
        }),
    ]
}

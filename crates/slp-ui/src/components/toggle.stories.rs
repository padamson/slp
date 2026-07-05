//! theoria story for `Toggle`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::Toggle;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Controls/Toggle", || {
            view! {
                <Toggle
                    label="Snap to grid"
                    testid="snap-grid"
                    checked=Signal::derive(|| true)
                    on_toggle=Callback::new(|_b| {})
                />
            }
        }),
        Story::new("Controls/Toggle (unchecked)", || {
            view! {
                <Toggle
                    label="Snap to grid"
                    testid="snap-grid"
                    checked=Signal::derive(|| false)
                    on_toggle=Callback::new(|_b| {})
                />
            }
        }),
    ]
}

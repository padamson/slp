//! theoria story for `ToolGroup`. Compiled only under the `stories` feature.
//! Shows the layout primitive grouping a couple of controls under a heading.

use leptos::prelude::*;

use super::{Toggle, ToolButton, ToolGroup};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Controls/Tool group", || {
        view! {
            <ToolGroup label="House">
                <ToolButton
                    label="Draw house"
                    testid="draw-house"
                    active=Signal::derive(|| true)
                    on_pick=Callback::new(|()| {})
                />
                <ToolButton
                    label="Add door"
                    testid="add-door"
                    active=Signal::derive(|| false)
                    on_pick=Callback::new(|()| {})
                />
                <Toggle
                    label="Snap"
                    testid="snap"
                    checked=Signal::derive(|| true)
                    on_toggle=Callback::new(|_b| {})
                />
            </ToolGroup>
        }
    })]
}

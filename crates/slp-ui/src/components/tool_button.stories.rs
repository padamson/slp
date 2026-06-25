//! theoria stories for `ToolButton`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::ToolButton;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Controls/Tool button", || {
            view! {
                <ToolButton
                    label="Draw house"
                    testid="x"
                    active=Signal::derive(|| false)
                    on_pick=Callback::new(|()| {})
                />
            }
        }),
        Story::new("Controls/Tool button · active", || {
            view! {
                <ToolButton
                    label="Draw house"
                    testid="x"
                    active=Signal::derive(|| true)
                    on_pick=Callback::new(|()| {})
                />
            }
        }),
    ]
}

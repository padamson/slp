//! theoria story for `TextField`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::TextField;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Controls/Text field", || {
            view! {
                <TextField
                    label="Name"
                    testid="name"
                    value=Signal::derive(|| "Lounge chair".to_string())
                    on_input=Callback::new(|_v| {})
                />
            }
        }),
        Story::new("Controls/Text field (empty, placeholder)", || {
            view! {
                <TextField
                    label="Category"
                    testid="category"
                    value=Signal::derive(String::new)
                    on_input=Callback::new(|_v| {})
                    placeholder="e.g. furniture"
                />
            }
        }),
    ]
}

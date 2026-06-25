//! theoria story for `NumberField`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::NumberField;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Controls/Number field", || {
        view! {
            <NumberField
                label="Width (ft)"
                testid="width"
                value=Signal::derive(|| 12.0)
                on_input=Callback::new(|_v| {})
                step=0.5
                min=1.0
            />
        }
    })]
}

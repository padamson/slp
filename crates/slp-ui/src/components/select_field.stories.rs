//! theoria story for `SelectField`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::SelectField;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Controls/Select field", || {
        view! {
            <SelectField
                label="Price unit"
                testid="price-unit"
                value=Signal::derive(|| "per_cubic_yard".to_string())
                options=vec![
                    ("per_item".to_string(), "Per item".to_string()),
                    ("per_square_foot".to_string(), "Per ft²".to_string()),
                    ("per_cubic_yard".to_string(), "Per yd³".to_string()),
                    ("per_linear_foot".to_string(), "Per linear ft".to_string()),
                ]
                on_change=Callback::new(|_| {})
            />
        }
    })]
}

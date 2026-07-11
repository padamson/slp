//! theoria story for `FileInput`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::FileInput;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Controls/File input", || {
        view! {
            <FileInput
                label="Upload image"
                testid="upload"
                accept="image/*"
                on_file=Callback::new(|_| {})
            />
        }
    })]
}

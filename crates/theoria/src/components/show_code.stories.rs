//! theoria stories for the show-code block — theoria dogfooding its own component
//! as a fixture (one level; not a Gallery-in-Gallery). Compiled only under the
//! `stories` feature.

use leptos::prelude::*;

use super::ShowCode;
use crate::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("ShowCode · block", || {
        view! {
            <ShowCode source="view! { <Yard yard_w=70.0 yard_d=30.0 px_ft=12.0 /> }" />
        }
    })]
}

//! theoria stories for `StoryNav` — theoria dogfooding its own component as a
//! fixture (one level; not a Gallery-in-Gallery). Compiled only under the
//! `stories` feature. Two stories so the gallery's switching is e2e-testable.

use leptos::prelude::*;

use super::StoryNav;
use crate::Story;

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("StoryNav · three", || {
            let (selected, set_selected) = signal(0usize);
            view! {
                <div data-testid="fixture-three">
                    <StoryNav
                        names=vec!["One", "Two", "Three"]
                        selected=selected
                        set_selected=set_selected
                    />
                </div>
            }
        }),
        Story::new("StoryNav · single", || {
            let (selected, set_selected) = signal(0usize);
            view! {
                <div data-testid="fixture-single">
                    <StoryNav names=vec!["Only"] selected=selected set_selected=set_selected />
                </div>
            }
        }),
    ]
}

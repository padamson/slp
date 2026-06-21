//! The gallery: a [`StoryNav`](super::StoryNav) sidebar plus the selected
//! story's view on a stage.

use leptos::prelude::*;

use super::StoryNav;
use crate::Story;

#[component]
pub fn Gallery(stories: Vec<Story>) -> impl IntoView {
    let (selected, set_selected) = signal(0usize);
    let names: Vec<&'static str> = stories.iter().map(|s| s.name).collect();
    let current = move || stories.get(selected.get()).map(|s| (s.view)());

    view! {
        <div class="theoria">
            <StoryNav names=names selected=selected set_selected=set_selected />
            <main class="theoria-stage">{current}</main>
        </div>
    }
}

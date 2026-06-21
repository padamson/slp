//! dokime component tests for `StoryNav`.

use leptos::prelude::*;

use super::StoryNav;

#[test]
fn lists_every_name() {
    let html = dokime::render(|| {
        let (selected, set_selected) = signal(0usize);
        view! { <StoryNav names=vec!["Alpha", "Beta"] selected=selected set_selected=set_selected /> }
    });
    assert!(html.contains("Alpha"));
    assert!(html.contains("Beta"));
    assert_eq!(dokime::count(&html, "<button"), 2);
}

#[test]
fn marks_the_selected_item_active() {
    let html = dokime::render(|| {
        let (selected, set_selected) = signal(1usize);
        view! { <StoryNav names=vec!["Alpha", "Beta"] selected=selected set_selected=set_selected /> }
    });
    // Exactly one button carries the `active` class (the selected one).
    assert_eq!(dokime::count(&html, "active"), 1);
}

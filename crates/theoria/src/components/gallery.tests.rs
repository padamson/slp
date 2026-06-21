//! dokime component tests for `Gallery`.

use leptos::prelude::*;

use super::Gallery;
use crate::Story;

#[test]
fn lists_names_and_shows_the_first_story() {
    let stories = vec![
        Story::new("Alpha", || view! { <p>"alpha-body"</p> }),
        Story::new("Beta", || view! { <p>"beta-body"</p> }),
    ];
    let html = dokime::render(move || view! { <Gallery stories=stories /> });
    assert!(html.contains("Alpha"));
    assert!(html.contains("Beta"));
    assert!(html.contains("alpha-body"), "first story renders on load");
}

//! dokime component tests for `Gallery`.

use leptos::prelude::*;

use super::Gallery;
use crate::{ArgControl, Story};

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

#[test]
fn shows_controls_description_and_code_for_a_story_with_args() {
    let html = dokime::render(move || {
        let stories = vec![Story::__from_macro(
            "Gamma",
            || view! { <p>"gamma-body"</p> }.into_any(),
            vec![("flag", ArgControl::Bool(RwSignal::new(true)))],
            "view! { <Gamma /> }",
            Some("A gamma story."),
        )];
        view! { <Gallery stories=stories /> }
    });
    assert!(html.contains("gamma-body"), "the stage renders the view");
    assert!(html.contains("A gamma story."), "the description renders");
    assert!(
        html.contains("theoria-controls") && html.contains("flag"),
        "controls render"
    );
    assert!(html.contains("theoria-code"), "show-code renders");
}

//! dokime component test for `Planner` (the composed root UI).

use leptos::prelude::*;

use super::Planner;

#[test]
fn renders_header_controls_and_yard() {
    let html = dokime::render(|| view! { <Planner /> });
    assert!(
        html.contains("Simple Landscape Planner"),
        "the header renders"
    );
    assert_eq!(
        dokime::count(&html, "<input"),
        2,
        "the two yard-size inputs"
    );
    assert!(
        html.contains(r#"data-testid="yard""#),
        "the yard canvas renders"
    );
}

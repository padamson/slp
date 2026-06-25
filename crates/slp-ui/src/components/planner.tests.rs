//! dokime component test for `Planner` (the composed root UI). The pure
//! placement logic is unit-tested in `slp_core::place`.

use leptos::prelude::*;

use super::Planner;

#[test]
fn renders_header_controls_tools_and_yard() {
    let html = dokime::render(|| view! { <Planner /> });
    assert!(
        html.contains("Simple Landscape Planner"),
        "the header renders"
    );
    assert_eq!(
        dokime::count(&html, "<input"),
        5,
        "two yard-size inputs + deck elevation + two snap toggles"
    );
    assert!(
        html.contains(r#"data-testid="draw-house""#)
            && html.contains(r#"data-testid="draw-deck""#)
            && html.contains(r#"data-testid="add-door""#)
            && html.contains(r#"data-testid="add-window""#),
        "the drawing-tool buttons render"
    );
    assert!(
        html.contains(r#"data-testid="yard""#),
        "the yard canvas renders"
    );
}

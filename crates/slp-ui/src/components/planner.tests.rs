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
        6,
        "two yard-size inputs + deck elevation + shape elevation + two snap toggles"
    );
    assert!(
        html.contains(r#"data-testid="draw-house""#)
            && html.contains(r#"data-testid="draw-deck""#)
            && html.contains(r#"data-testid="add-door""#)
            && html.contains(r#"data-testid="add-window""#)
            && html.contains(r#"data-testid="add-steps""#)
            && html.contains(r#"data-testid="draw-shape""#),
        "the drawing-tool buttons render"
    );
    assert!(
        html.contains(r#"data-testid="yard""#),
        "the yard canvas renders"
    );
}

#[test]
fn the_starter_catalog_seeds_on_load_so_the_palette_and_estimate_appear_immediately() {
    // The starter catalog seeds unconditionally on load (a tree/fire pit
    // doesn't need a deck drawn first) — the palette and estimate panel,
    // both gated on a non-empty catalog, are already up on a fresh plan.
    let html = dokime::render(|| view! { <Planner /> });
    assert!(
        html.contains(r#"data-testid="object-palette""#),
        "the object palette appears without drawing anything first"
    );
    assert!(
        html.contains(r#"data-testid="estimate""#),
        "the estimate panel appears without drawing anything first"
    );
}

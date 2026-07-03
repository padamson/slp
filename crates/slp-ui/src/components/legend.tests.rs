//! dokime component tests for `Legend`.

use leptos::prelude::*;

use super::Legend;

#[test]
fn renders_one_item_per_visual_convention() {
    let html = dokime::render(move || view! { <Legend start_x=10.0 baseline_y=40.0 /> });
    for testid in [
        "legend-item-house",
        "legend-item-deck",
        "legend-item-planned",
        "legend-item-existing",
        "legend-item-virtual",
        "legend-item-selected",
        "legend-item-overflow",
    ] {
        assert!(
            html.contains(&format!(r#"data-testid="{testid}""#)),
            "missing legend entry: {testid}"
        );
    }
    assert!(html.contains("House"), "labels are readable text");
    assert!(html.contains("Doesn't fit"), "labels are readable text");
}

#[test]
fn house_and_deck_icons_carry_node_corner_markers() {
    // House/deck are user-drawn outlines: their icon has 4 corner dots.
    let html = dokime::render(move || view! { <Legend start_x=10.0 baseline_y=40.0 /> });
    assert_eq!(
        dokime::count(&html, "<circle"),
        8,
        "4 corner dots each for the house and deck icons"
    );
}

#[test]
fn furniture_status_icons_match_the_shared_canvas_styling() {
    // The legend reads from the same `furniture_style` the canvas uses, so a
    // status's dash pattern + opacity are identical in both places.
    let html = dokime::render(move || view! { <Legend start_x=10.0 baseline_y=40.0 /> });
    assert!(
        html.contains(r#"stroke-dasharray="none""#),
        "planned: solid outline"
    );
    assert!(
        html.contains(r#"stroke-dasharray="6,3""#),
        "existing: the same dash as the canvas"
    );
    assert!(
        html.contains(r#"stroke-dasharray="3,3""#),
        "virtual: the same tighter dash as the canvas"
    );
    assert!(
        html.contains(r#"fill-opacity="0.55""#),
        "existing: the same reduced opacity as the canvas"
    );
    assert!(
        html.contains(r#"fill-opacity="0.3""#),
        "virtual: the same ghost opacity as the canvas"
    );
}

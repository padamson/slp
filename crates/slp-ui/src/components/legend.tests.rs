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
        "legend-item-planned-virtual",
        "legend-item-existing-virtual",
        "legend-item-selected",
        "legend-item-overflow",
        "legend-item-clearance",
    ] {
        assert!(
            html.contains(&format!(r#"data-testid="{testid}""#)),
            "missing legend entry: {testid}"
        );
    }
    assert!(html.contains("House"), "labels are readable text");
    assert!(html.contains("Doesn't fit"), "labels are readable text");
    assert!(html.contains("Keep-clear zone"), "labels are readable text");
}

#[test]
fn house_and_deck_icons_carry_node_corner_markers() {
    // House/deck are user-drawn outlines: their icon has 4 corner dots. Plus
    // one more `<circle>` for the clearance-ring icon itself.
    let html = dokime::render(move || view! { <Legend start_x=10.0 baseline_y=40.0 /> });
    assert_eq!(
        dokime::count(&html, "<circle"),
        9,
        "4 corner dots each for house/deck, + the ring icon"
    );
}

#[test]
fn the_clearance_entry_is_an_unfilled_dashed_ring() {
    let html = dokime::render(move || view! { <Legend start_x=10.0 baseline_y=40.0 /> });
    assert!(
        html.contains(r#"data-testid="legend-item-clearance""#),
        "the clearance entry renders"
    );
    assert!(html.contains(r#"stroke-dasharray="5,3""#), "dashed ring");
    assert!(
        html.contains(r##"stroke="#8a8275""##),
        "the quiet clearance color, not overflow-red"
    );
}

#[test]
fn furniture_entries_match_the_shared_canvas_styling() {
    // The legend reads from the same `furniture_style` the canvas uses, so a
    // combination's dash pattern + opacity are identical in both places.
    let html = dokime::render(move || view! { <Legend start_x=10.0 baseline_y=40.0 /> });
    assert!(
        html.contains(r#"stroke-dasharray="none""#),
        "real: solid outline, same as the canvas"
    );
    assert!(
        html.contains(r#"stroke-dasharray="4,3""#),
        "virtual: the same dash as the canvas"
    );
    assert!(
        html.contains(r#"fill-opacity="0.7""#),
        "real: the same full opacity as the canvas"
    );
    assert!(
        html.contains(r#"fill-opacity="0.35""#),
        "virtual: the same ghost opacity as the canvas"
    );
}

#[test]
fn existing_entries_carry_a_double_outline() {
    // Existing (double-line) icons get a second, inset stroke rect; planned
    // (single-line) icons don't.
    let html = dokime::render(move || view! { <Legend start_x=10.0 baseline_y=40.0 /> });
    // House, Deck (2), Planned (1), Existing (2), Planned-virtual (1),
    // Existing-virtual (2), Selected (1), Doesn't-fit (1) = 10 furniture-icon
    // + house/deck rects.
    assert_eq!(
        dokime::count(&html, "<rect"),
        10,
        "existing entries render a second inset rect"
    );
}

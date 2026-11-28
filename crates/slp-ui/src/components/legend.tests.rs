//! leptotes component tests for `Legend`.

use leptos::prelude::*;

use super::Legend;

fn legend() -> leptotes::Dom {
    leptotes::dom(move || view! { <Legend start_x=10.0 baseline_y=40.0 /> })
}

#[test]
fn renders_one_item_per_visual_convention() {
    let dom = legend();

    let mut testids = dom.attrs("g.legend > g.legend-item", "data-testid");
    testids.sort_unstable();
    let mut expected = [
        "legend-item-house",
        "legend-item-deck",
        "legend-item-planned",
        "legend-item-existing",
        "legend-item-planned-virtual",
        "legend-item-existing-virtual",
        "legend-item-selected",
        "legend-item-overflow",
        "legend-item-clearance",
    ];
    expected.sort_unstable();

    assert_eq!(testids, expected, "{dom}");
}

#[test]
fn every_item_carries_a_readable_label() {
    let dom = legend();

    let labels = dom.texts("g.legend-item > text");

    assert_eq!(
        (
            labels.len(),
            labels.iter().all(|l| !l.is_empty()),
            labels.iter().any(|l| l == "House"),
            labels.iter().any(|l| l == "Doesn't fit"),
            labels.iter().any(|l| l == "Keep-clear zone"),
        ),
        (9, true, true, true, true),
        "{labels:?}"
    );
}

/// The legend's entries, in display order.
const ENTRIES: [&str; 9] = [
    "house",
    "deck",
    "planned",
    "existing",
    "planned-virtual",
    "existing-virtual",
    "selected",
    "overflow",
    "clearance",
];

fn per_entry(dom: &leptotes::Dom, selector: &str) -> Vec<usize> {
    ENTRIES
        .iter()
        .map(|e| dom.count(&format!(r#"[data-testid="legend-item-{e}"] {selector}"#)))
        .collect()
}

#[test]
fn house_and_deck_icons_carry_node_corner_markers() {
    let dom = legend();

    // House/deck are user-drawn outlines: their icon has 4 corner dots, and
    // the square icons have none (the contrast is the point). The clearance
    // entry's one circle is its ring icon.
    assert_eq!(
        per_entry(&dom, "circle"),
        [4, 4, 0, 0, 0, 0, 0, 0, 1],
        "{dom}"
    );
}

#[test]
fn the_clearance_entry_is_an_unfilled_dashed_ring() {
    let dom = legend();

    let ring = dom.find(r#"[data-testid="legend-item-clearance"] circle"#);

    // The quiet clearance color, not overflow-red.
    assert_eq!(
        (
            ring.attr("fill"),
            ring.attr("stroke-dasharray"),
            ring.attr("stroke")
        ),
        (Some("none"), Some("5,3"), Some("#8a8275")),
        "{dom}"
    );
}

#[test]
fn furniture_entries_match_the_shared_canvas_styling() {
    // The legend reads from the same `furniture_style` the canvas uses, so a
    // combination's dash pattern + opacity are identical in both places.
    let dom = legend();
    let icon = |testid: &str| dom.find(&format!(r#"[data-testid="{testid}"] > rect"#));

    let (real, virt) = (
        icon("legend-item-planned"),
        icon("legend-item-planned-virtual"),
    );

    assert_eq!(
        [
            (real.attr("stroke-dasharray"), real.attr("fill-opacity")),
            (virt.attr("stroke-dasharray"), virt.attr("fill-opacity")),
        ],
        [(Some("none"), Some("0.7")), (Some("4,3"), Some("0.35"))],
        "{dom}"
    );
}

#[test]
fn existing_entries_carry_a_double_outline() {
    let dom = legend();

    // Existing (double-line) icons get a second, inset stroke rect; every
    // other rect icon has one, and the clearance ring has none.
    assert_eq!(
        per_entry(&dom, "rect"),
        [1, 1, 1, 2, 1, 2, 1, 1, 0],
        "{dom}"
    );
}

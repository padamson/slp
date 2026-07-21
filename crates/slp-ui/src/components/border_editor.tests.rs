//! dokime component tests for `BorderEditor`.

use leptos::prelude::*;
use slp_core::Border;

use super::BorderEditor;

fn noop_us() -> Callback<usize> {
    Callback::new(|_| {})
}
fn noop_um() -> Callback<(usize, String)> {
    Callback::new(|_| {})
}
fn noop_uw() -> Callback<(usize, f64)> {
    Callback::new(|_| {})
}
fn noop() -> Callback<()> {
    Callback::new(|()| {})
}

fn options() -> Vec<(String, String)> {
    vec![
        ("cobble".to_string(), "Border cobble".to_string()),
        ("edging-stone".to_string(), "Edging stones".to_string()),
    ]
}

fn editor(borders: Vec<Border>) -> String {
    editor_with_nodes(borders, 0)
}

fn editor_with_nodes(borders: Vec<Border>, node_count: usize) -> String {
    dokime::render(move || {
        let borders = borders.clone();
        view! {
            <BorderEditor
                borders=borders
                material_options=options()
                node_count=node_count
                on_material=noop_um()
                on_width=noop_uw()
                on_add=noop()
                on_remove=noop_us()
            />
        }
    })
}

#[test]
fn renders_a_row_per_ring_with_material_width_and_remove() {
    let html = editor(vec![
        Border::new("cobble".to_string(), 0.5),
        Border::new("edging-stone".to_string(), 0.25),
    ]);
    assert!(html.contains(r#"data-testid="border-editor""#));
    assert_eq!(dokime::count(&html, r#"data-testid="border-material""#), 2);
    assert!(html.contains(r#"data-testid="border-row-0""#));
    assert!(html.contains(r#"data-testid="border-row-1""#));
    assert!(
        html.contains(r#"data-testid="border-remove-1""#),
        "each ring can be removed"
    );
    // Each row's material is preselected (the width is a DOM prop, not an
    // SSR attribute, so it isn't asserted here).
    assert!(
        html.contains(r#"<option value="cobble" selected>"#),
        "ring 0's material selected: {html}"
    );
    assert!(
        html.contains(r#"<option value="edging-stone" selected>"#),
        "ring 1's material selected"
    );
}

#[test]
fn an_empty_list_still_offers_add() {
    let html = editor(vec![]);
    assert_eq!(dokime::count(&html, r#"data-testid="border-material""#), 0);
    assert!(
        html.contains(r#"data-testid="border-add""#),
        "the + Border button is the entry point"
    );
}

#[test]
fn a_nodal_area_offers_from_to_span_selects() {
    // 4 boundary nodes: each row gets From/To selects with "—" (whole
    // perimeter) plus one option per node; a span border preselects its nodes.
    let mut b = Border::new("edging-stone".to_string(), 0.25);
    b.start_node = Some(0.0);
    b.end_node = Some(2.0);
    let html = editor_with_nodes(vec![b], 4);
    assert_eq!(dokime::count(&html, r#"data-testid="border-from""#), 1);
    assert_eq!(dokime::count(&html, r#"data-testid="border-to""#), 1);
    assert!(
        html.contains(r#"<option value="0" selected>"#),
        "from n0: {html}"
    );
    assert!(html.contains(r#"<option value="2" selected>"#), "to n2");
    assert!(
        html.contains(r#"<option value="3">"#),
        "one option per node"
    );
}

#[test]
fn a_circle_hides_the_span_selects() {
    // node_count 0 (a circle): no From/To — circles only ring the perimeter.
    let html = editor_with_nodes(vec![Border::new("cobble".to_string(), 0.5)], 0);
    assert_eq!(dokime::count(&html, r#"data-testid="border-from""#), 0);
    assert_eq!(dokime::count(&html, r#"data-testid="border-to""#), 0);
}

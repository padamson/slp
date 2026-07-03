//! dokime component tests for `ObjectPalette`.

use leptos::prelude::*;
use slp_core::{CatalogItem, FootprintShape};

use super::ObjectPalette;

fn item(id: &str, name: &str, category: &str, circle: bool) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c.category = Some(category.to_string());
    c.unit_price = Some(199.0);
    if circle {
        c.shape = FootprintShape::circle;
    }
    c
}

fn catalog() -> Vec<CatalogItem> {
    vec![
        item("lounge-chair", "Lounge chair", "furniture", false),
        item("dining-table", "Dining table", "furniture", false),
        item("fire-pit", "Fire pit", "fire-pit", true),
    ]
}

#[test]
fn renders_a_tile_per_item_grouped_by_category() {
    let html = dokime::render(move || {
        view! { <ObjectPalette catalog=catalog() armed=Signal::derive(|| None::<String>) on_pick=Callback::new(|_| {}) /> }
    });
    // One tile per catalog item.
    assert!(html.contains(r#"data-testid="palette-lounge-chair""#));
    assert!(html.contains(r#"data-testid="palette-dining-table""#));
    assert!(html.contains(r#"data-testid="palette-fire-pit""#));
    // Grouped, with humanized category labels.
    assert!(html.contains("Furniture"), "furniture group label");
    assert!(html.contains("Fire pit"), "humanized fire-pit group label");
    assert_eq!(
        dokime::count(&html, r#"class="palette-group""#),
        2,
        "two category groups"
    );
    // Names + prices show on the tiles.
    assert!(html.contains("Lounge chair"));
    assert!(html.contains("$199"));
}

#[test]
fn the_armed_item_is_flagged() {
    let html = dokime::render(move || {
        view! {
            <ObjectPalette
                catalog=catalog()
                armed=Signal::derive(|| Some("fire-pit".to_string()))
                on_pick=Callback::new(|_| {})
            />
        }
    });
    assert_eq!(dokime::count(&html, "armed"), 1, "exactly one armed tile");
}

#[test]
fn nothing_is_flagged_when_nothing_is_armed() {
    let html = dokime::render(move || {
        view! { <ObjectPalette catalog=catalog() armed=Signal::derive(|| None::<String>) on_pick=Callback::new(|_| {}) /> }
    });
    assert_eq!(dokime::count(&html, "armed"), 0, "no armed tile");
}

#[test]
fn a_round_item_gets_a_circle_icon() {
    let html = dokime::render(move || {
        view! { <ObjectPalette catalog=catalog() armed=Signal::derive(|| None::<String>) on_pick=Callback::new(|_| {}) /> }
    });
    // Two rectangular furniture icons + one round fire-pit icon.
    assert_eq!(
        dokime::count(&html, "<circle"),
        1,
        "the fire pit's round icon"
    );
    assert_eq!(dokime::count(&html, "<rect"), 2, "the two furniture icons");
}

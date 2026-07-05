//! dokime component tests for `Furnishings` (placed-object footprints).

use leptos::prelude::*;
use slp_core::{CatalogItem, Coord, FootprintShape, ItemStatus, Object};

use super::{Furnishings, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 20.0,
    }
}

fn item(id: &str, w_ft: Option<f64>, d_ft: Option<f64>) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.width_ft = w_ft;
    c.depth_ft = d_ft;
    c
}

fn round_item(id: &str, diameter: f64) -> CatalogItem {
    let mut c = item(id, Some(diameter), Some(diameter));
    c.shape = FootprintShape::circle;
    c
}

fn fire_pit(diameter: f64, clearance_ft: f64) -> CatalogItem {
    let mut c = round_item("fire-pit", diameter);
    c.category = Some("fire-pit".to_string());
    c.clearance_ft = Some(clearance_ft);
    c
}

fn tree(id: &str, canopy_ft: f64, trunk_ft: f64) -> CatalogItem {
    let mut c = round_item(id, canopy_ft);
    c.category = Some("tree".to_string());
    c.trunk_diameter_ft = Some(trunk_ft);
    c
}

#[test]
fn renders_a_footprint_to_scale() {
    // A 3 ft × 1.5 ft item at (5,5): 10 px/ft → a 30 × 15 px rectangle centered
    // at sx(5)=50, sy(5)=150.
    let catalog = vec![item("chair", Some(3.0), Some(1.5))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        html.contains(r#"class="furnishings""#),
        "tagged for queries"
    );
    assert!(html.contains("<rect"), "the footprint is a rect");
    assert!(html.contains(r#"width="30""#), "3 ft → 30 px wide");
    assert!(html.contains(r#"height="15""#), "1.5 ft → 15 px deep");
    assert!(
        html.contains("translate(50,150)"),
        "centered at the object's position"
    );
}

#[test]
fn a_round_item_renders_a_circle_not_a_rect() {
    // A ⌀4 ft fire pit at (5,5): 10 px/ft → a circle of radius 20 px centered
    // at sx(5)=50, sy(5)=150.
    let catalog = vec![round_item("fire-pit", 4.0)];
    let objects = vec![Object::new("fire-pit".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(html.contains("<circle"), "the footprint is a circle");
    assert!(!html.contains("<rect"), "no rectangle for a round item");
    assert!(html.contains(r#"r="20""#), "4 ft diameter → 20 px radius");
    assert!(
        html.contains("translate(50,150)"),
        "centered at the object's position"
    );
}

#[test]
fn an_existing_round_item_is_a_double_ring() {
    let catalog = vec![round_item("fire-pit", 4.0)];
    let mut obj = Object::new("fire-pit".to_string(), 5.0, 5.0);
    obj.status = ItemStatus::existing;
    let objects = vec![obj];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    // Existing → a second, inset ring; still circles, never rects.
    assert_eq!(dokime::count(&html, "<circle"), 2, "a double ring");
    assert!(!html.contains("<rect"));
}

#[test]
fn a_round_item_with_no_clearance_shows_no_ring() {
    let catalog = vec![round_item("fire-pit", 4.0)]; // no clearance_ft
    let objects = vec![Object::new("fire-pit".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        !html.contains(r#"data-testid="clearance-ring""#),
        "no ring without a clearance guideline"
    );
}

#[test]
fn an_isolated_fire_pit_shows_a_quiet_dashed_ring() {
    // ⌀4 ft (radius 2 ft) + 3 ft clearance = 5 ft ring radius = 50 px.
    let catalog = vec![fire_pit(4.0, 3.0)];
    let objects = vec![Object::new("fire-pit".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        html.contains(r#"data-testid="clearance-ring""#),
        "the ring renders"
    );
    assert!(html.contains(r#"r="50""#), "radius + clearance in px");
    assert!(
        html.contains(r##"stroke="#8a8275""##),
        "quiet color — nothing intrudes"
    );
    assert!(
        !html.contains("furniture-item--intrudes"),
        "no intrusion class"
    );
}

#[test]
fn a_nearby_object_turns_the_ring_red() {
    let catalog = vec![fire_pit(4.0, 3.0), item("chair", Some(2.0), Some(2.0))];
    let objects = vec![
        Object::new("fire-pit".to_string(), 5.0, 5.0),
        Object::new("chair".to_string(), 7.0, 5.0), // 2 ft away — inside the 5 ft ring
    ];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        html.contains("furniture-item--intrudes"),
        "the fire pit is flagged as intruded"
    );
    assert!(
        html.contains(r##"stroke="#7a1216""##),
        "the ring turns its own darker-red intrusion color"
    );
}

#[test]
fn a_distant_object_does_not_turn_the_ring_red() {
    let catalog = vec![fire_pit(4.0, 3.0), item("chair", Some(2.0), Some(2.0))];
    let objects = vec![
        Object::new("fire-pit".to_string(), 5.0, 5.0),
        Object::new("chair".to_string(), 20.0, 5.0), // far outside the ring
    ];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(!html.contains("furniture-item--intrudes"));
    assert!(html.contains(r##"stroke="#8a8275""##));
}

#[test]
fn a_nearby_structure_edge_turns_the_ring_red() {
    let catalog = vec![fire_pit(4.0, 3.0)];
    let objects = vec![Object::new("fire-pit".to_string(), 5.0, 5.0)];
    // A wall-like outline with an edge at x=9 — 4 ft from the fire pit's
    // center, inside the 5 ft clearance ring.
    let wall = vec![
        Coord::new(9.0, 0.0),
        Coord::new(9.0, 20.0),
        Coord::new(15.0, 20.0),
        Coord::new(15.0, 0.0),
    ];
    let html = dokime::render(move || {
        view! {
            <Furnishings
                t=t()
                objects=objects
                catalog=catalog
                structure_outlines=vec![wall]
            />
        }
    });
    assert!(
        html.contains("furniture-item--intrudes"),
        "a nearby structure edge also counts as an intrusion"
    );
}

#[test]
fn renders_one_group_per_object() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![
        Object::new("chair".to_string(), 2.0, 2.0),
        Object::new("chair".to_string(), 8.0, 8.0),
    ];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    // Prefix match: every object also carries a status class (e.g.
    // `furniture-item--planned`) alongside the base "furniture-item".
    assert_eq!(
        dokime::count(&html, r#"class="furniture-item"#),
        2,
        "one group per placed object"
    );
}

#[test]
fn rotation_is_applied_clockwise() {
    let catalog = vec![item("chair", Some(2.0), Some(1.0))];
    let mut obj = Object::new("chair".to_string(), 5.0, 5.0);
    obj.rot = Some(90.0);
    let objects = vec![obj];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        html.contains("rotate(90)"),
        "clockwise-from-north maps to SVG rotate(+rot)"
    );
}

#[test]
fn unresolved_catalog_ref_is_not_drawn() {
    // An object referencing an id absent from the catalog has no footprint to
    // draw, so nothing renders (mirrors the cost take-off's exclusion).
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![Object::new("ghost-id".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(!html.contains(r#"class="furnishings""#));
}

#[test]
fn existing_and_virtual_objects_still_render() {
    // status/is_virtual affect only the cost take-off, not visibility — every
    // placed object is shown on the plan.
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let mut existing = Object::new("chair".to_string(), 2.0, 2.0);
    existing.status = ItemStatus::existing;
    let mut ghost = Object::new("chair".to_string(), 8.0, 8.0);
    ghost.is_virtual = true;
    let objects = vec![existing, ghost];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    // Prefix match (not the exact class attribute): every object carries a
    // status class alongside "furniture-item".
    assert_eq!(
        dokime::count(&html, r#"class="furniture-item"#),
        2,
        "existing + virtual both render"
    );
}

#[test]
fn planned_real_is_a_single_solid_full_opacity_outline() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)]; // default: planned, real
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        html.contains("furniture-item--planned"),
        "planned status class"
    );
    assert!(!html.contains("furniture-item--virtual"));
    assert!(
        html.contains(r#"stroke-dasharray="none""#),
        "a solid outline"
    );
    assert!(html.contains(r#"fill-opacity="0.7""#), "full opacity");
    // Single outline: no second, inset stroke rect.
    assert_eq!(dokime::count(&html, "<rect"), 1, "no double-outline rect");
}

#[test]
fn existing_real_is_a_double_solid_outline() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let mut obj = Object::new("chair".to_string(), 5.0, 5.0);
    obj.status = ItemStatus::existing;
    let objects = vec![obj];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        html.contains("furniture-item--existing"),
        "existing status class"
    );
    assert!(
        html.contains(r#"stroke-dasharray="none""#),
        "solid, not dashed"
    );
    assert!(
        html.contains(r#"fill-opacity="0.7""#),
        "full opacity — it's real"
    );
    // Double outline: an outer rect plus a second, inset stroke rect.
    assert_eq!(
        dokime::count(&html, "<rect"),
        2,
        "a double-outline rect pair"
    );
}

#[test]
fn planned_virtual_is_a_single_dashed_ghost() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let mut obj = Object::new("chair".to_string(), 5.0, 5.0);
    obj.is_virtual = true;
    let objects = vec![obj];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(html.contains("furniture-item--planned"));
    assert!(html.contains("furniture-item--virtual"));
    assert!(html.contains(r#"stroke-dasharray="4,3""#), "dashed outline");
    assert!(html.contains(r#"fill-opacity="0.35""#), "ghosted opacity");
    assert_eq!(dokime::count(&html, "<rect"), 1, "single outline — planned");
}

#[test]
fn existing_virtual_is_a_double_dashed_ghost() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let mut obj = Object::new("chair".to_string(), 5.0, 5.0);
    obj.status = ItemStatus::existing;
    obj.is_virtual = true;
    let objects = vec![obj];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(html.contains("furniture-item--existing"));
    assert!(html.contains("furniture-item--virtual"));
    assert!(html.contains(r#"stroke-dasharray="4,3""#), "dashed outline");
    assert!(html.contains(r#"fill-opacity="0.35""#), "ghosted opacity");
    assert_eq!(
        dokime::count(&html, "<rect"),
        2,
        "double outline — existing"
    );
}

#[test]
fn a_selected_existing_object_keeps_its_double_outline_under_the_selection_tint() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let mut obj = Object::new("chair".to_string(), 5.0, 5.0);
    obj.status = ItemStatus::existing;
    let objects = vec![obj];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog selected=Some(0) /> }
    });
    assert!(
        html.contains(r##"fill="#7ea9d4""##),
        "the selection tint still wins the fill color"
    );
    assert_eq!(
        dokime::count(&html, "<rect"),
        2,
        "the double outline still reads through a selection"
    );
}

#[test]
fn missing_dimensions_fall_back_to_a_default_footprint() {
    // A catalog item with no width/depth still places a visible 1 ft (= 10 px)
    // square so the object can be seen and selected.
    let catalog = vec![item("mystery", None, None)];
    let objects = vec![Object::new("mystery".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(html.contains(r#"width="10""#), "1 ft default → 10 px");
}

fn deck(w: f64, d: f64) -> Vec<Coord> {
    vec![
        Coord::new(0.0, 0.0),
        Coord::new(w, 0.0),
        Coord::new(w, d),
        Coord::new(0.0, d),
    ]
}

#[test]
fn an_object_overhanging_its_surface_is_highlighted() {
    // A 4×4 ft chair centered at (5,5) pokes past a 6×6 ft deck (corners reach
    // x=7, y=7) — it does not fit on a single surface.
    let catalog = vec![item("chair", Some(4.0), Some(4.0))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog surfaces=vec![deck(6.0, 6.0)] /> }
    });
    assert!(
        html.contains("furniture-item--overflows"),
        "an overhanging object is highlighted"
    );
}

#[test]
fn an_object_fully_on_a_surface_is_not_highlighted() {
    // A 2×2 ft chair well inside a 10×10 ft deck fits.
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog surfaces=vec![deck(10.0, 10.0)] /> }
    });
    assert!(
        !html.contains("furniture-item--overflows"),
        "a contained object keeps the normal outline"
    );
}

#[test]
fn the_selected_object_is_highlighted() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![
        Object::new("chair".to_string(), 2.0, 2.0),
        Object::new("chair".to_string(), 8.0, 8.0),
    ];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog selected=Some(1) /> }
    });
    assert_eq!(
        dokime::count(&html, "furniture-item--selected"),
        1,
        "only the selected object carries the selection class"
    );
}

#[test]
fn the_selected_object_shows_a_rotation_handle() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog selected=Some(0) /> }
    });
    assert!(
        html.contains(r#"data-testid="rotate-handle""#),
        "the selected object carries a rotation handle"
    );
}

#[test]
fn unselected_objects_have_no_rotation_handle() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog /> }
    });
    assert!(
        !html.contains("rotate-handle"),
        "no handle without a selection"
    );
}

#[test]
fn a_selected_round_item_shows_no_rotation_handle() {
    // Rotating a circle is a visual no-op, so the handle would be a pointless
    // affordance — unlike a rectangular item, which still gets one.
    let catalog = vec![round_item("oak-tree", 20.0)];
    let objects = vec![Object::new("oak-tree".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog selected=Some(0) /> }
    });
    assert!(
        !html.contains("rotate-handle"),
        "no handle on a selected round item"
    );
}

#[test]
fn a_selected_tree_shows_canopy_and_trunk_resize_handles() {
    let catalog = vec![tree("oak-tree", 8.0, 2.0)];
    let objects = vec![Object::new("oak-tree".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog selected=Some(0) /> }
    });
    assert!(
        html.contains(r#"data-testid="canopy-handle""#),
        "a canopy resize handle"
    );
    assert!(
        html.contains(r#"data-testid="trunk-handle""#),
        "a trunk resize handle"
    );
}

#[test]
fn an_unselected_tree_shows_no_resize_handles() {
    let catalog = vec![tree("oak-tree", 8.0, 2.0)];
    let objects = vec![Object::new("oak-tree".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(!html.contains("canopy-handle"));
    assert!(!html.contains("trunk-handle"));
}

#[test]
fn a_selected_fire_pit_shows_no_resize_handles() {
    // Resize handles are a tree-specific affordance (canopy/trunk); a fire pit
    // has neither, so it gets none.
    let catalog = vec![fire_pit(3.0, 1.5)];
    let objects = vec![Object::new("fire-pit".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog selected=Some(0) /> }
    });
    assert!(!html.contains("canopy-handle"));
    assert!(!html.contains("trunk-handle"));
}

#[test]
fn no_surfaces_means_no_fit_check() {
    // Without surfaces there is nothing to fit within, so nothing is highlighted.
    let catalog = vec![item("chair", Some(4.0), Some(4.0))];
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog /> }
    });
    assert!(!html.contains("furniture-item--overflows"));
}

#[test]
fn a_tree_renders_a_canopy_and_a_trunk() {
    // ⌀8 ft canopy (radius 40 px) + ⌀2 ft trunk (radius 10 px) at (5,5).
    let catalog = vec![tree("oak-tree", 8.0, 2.0)];
    let objects = vec![Object::new("oak-tree".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert_eq!(
        dokime::count(&html, "<circle"),
        2,
        "a canopy circle and a trunk circle"
    );
    assert!(html.contains(r#"r="40""#), "the canopy radius");
    assert!(html.contains(r#"r="10""#), "the trunk radius");
    assert!(
        html.contains(r##"fill="#a8d5a0""##),
        "the canopy's light green"
    );
    assert!(
        html.contains(r##"fill="#5a3a22""##),
        "the trunk's dark brown"
    );
}

#[test]
fn a_fire_pit_fills_silver() {
    let catalog = vec![fire_pit(3.0, 1.5)];
    let objects = vec![Object::new("fire-pit".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(
        html.contains(r##"fill="#b8b8bc""##),
        "a fire pit fills silver, not the shared furniture brown"
    );
}

#[test]
fn a_tree_trunk_is_fine_on_bare_yard() {
    let catalog = vec![tree("oak-tree", 8.0, 2.0)];
    let objects = vec![Object::new("oak-tree".to_string(), 5.0, 5.0)];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(!html.contains("furniture-item--trunk-invalid"));
    assert!(
        html.contains(r##"fill="#5a3a22""##),
        "the trunk keeps its normal color"
    );
}

#[test]
fn a_tree_trunk_flags_on_the_house() {
    let catalog = vec![tree("oak-tree", 8.0, 2.0)];
    let objects = vec![Object::new("oak-tree".to_string(), 5.0, 5.0)];
    let house = deck(10.0, 10.0); // any polygon containing (5,5)
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog house_outline=house /> }
    });
    assert!(html.contains("furniture-item--trunk-invalid"));
    assert!(
        html.contains(r##"fill="#d4351c""##),
        "the trunk itself turns the overflow red"
    );
}

#[test]
fn a_tree_trunk_flags_when_its_disk_straddles_the_house_edge() {
    // ⌀2 ft trunk (radius 1 ft) centered 0.5 ft outside a house wall at x=5.5:
    // the center is off the house, but the trunk's disk still overlaps the
    // wall, so it should flag just like a fully-contained center would.
    let catalog = vec![tree("oak-tree", 8.0, 2.0)];
    let objects = vec![Object::new("oak-tree".to_string(), 5.5, 5.0)];
    let house = deck(5.0, 10.0); // spans x:[0,5] — the tree's center (5.5) is outside
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog house_outline=house /> }
    });
    assert!(
        html.contains("furniture-item--trunk-invalid"),
        "the trunk's disk overlaps the house edge even though its center doesn't"
    );
}

#[test]
fn a_tree_trunk_flags_on_the_deck_too() {
    // A tree's canopy may overhang the deck freely, but its trunk still
    // shouldn't be standing on it.
    let catalog = vec![tree("oak-tree", 8.0, 2.0)];
    let objects = vec![Object::new("oak-tree".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! {
            <Furnishings t=t() objects=objects catalog=catalog surfaces=vec![deck(10.0, 10.0)] />
        }
    });
    assert!(html.contains("furniture-item--trunk-invalid"));
    // The canopy itself is never flagged with the generic overflow class.
    assert!(!html.contains("furniture-item--overflows"));
}

#[test]
fn a_fire_pit_is_fine_off_a_deck_but_flags_on_the_house() {
    // Unlike furniture, a fire pit doesn't need to be on a deck — the yard
    // (bare ground) is a valid surface for it.
    let catalog = vec![fire_pit(3.0, 1.5)];
    let objects = vec![Object::new("fire-pit".to_string(), 5.0, 5.0)];
    let far_deck = deck(2.0, 2.0); // doesn't contain (5,5)
    let html = dokime::render(move || {
        view! {
            <Furnishings t=t() objects=objects catalog=catalog surfaces=vec![far_deck] />
        }
    });
    assert!(
        !html.contains("furniture-item--overflows"),
        "a fire pit off the deck, on bare yard, is fine"
    );

    let catalog = vec![fire_pit(3.0, 1.5)];
    let objects = vec![Object::new("fire-pit".to_string(), 5.0, 5.0)];
    let house = deck(10.0, 10.0); // contains (5,5)
    let html = dokime::render(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog house_outline=house /> }
    });
    assert!(
        html.contains("furniture-item--overflows"),
        "a fire pit on the house is flagged"
    );
}

#[test]
fn a_per_object_canopy_and_trunk_override_wins_over_the_catalog_default() {
    let catalog = vec![tree("oak-tree", 8.0, 2.0)];
    let mut obj = Object::new("oak-tree".to_string(), 5.0, 5.0);
    obj.canopy_diameter_ft = Some(20.0); // → r=100 px
    obj.trunk_diameter_ft = Some(4.0); // → r=20 px
    let objects = vec![obj];
    let html =
        dokime::render(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> });
    assert!(html.contains(r#"r="100""#), "the overridden canopy radius");
    assert!(html.contains(r#"r="20""#), "the overridden trunk radius");
}

#[test]
fn no_objects_renders_nothing() {
    let html = dokime::render(move || view! { <Furnishings t=t() objects=Vec::new() /> });
    assert!(!html.contains(r#"class="furnishings""#));
}

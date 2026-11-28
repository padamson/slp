//! leptotes component tests for `Furnishings` (placed-object footprints).

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

fn bush(id: &str, spread_ft: f64) -> CatalogItem {
    let mut c = round_item(id, spread_ft);
    c.category = Some("bush".to_string());
    c
}

fn grill(id: &str, w: f64, d: f64, clearance_ft: f64) -> CatalogItem {
    let mut c = item(id, Some(w), Some(d));
    c.category = Some("grill".to_string());
    c.clearance_ft = Some(clearance_ft);
    c
}

fn hot_tub_round(id: &str, diameter: f64) -> CatalogItem {
    let mut c = round_item(id, diameter);
    c.category = Some("hot-tub".to_string());
    c
}

/// The footprint and, for an existing item, its inset second outline.
const OUTLINES: &str =
    "g.furniture-item > rect:not([class]), g.furniture-item > circle:not([class])";
/// An item's footprint: the first of its outlines, since it renders before an
/// existing item's inset copy. The pad, trunk, clearance ring, and handles all
/// carry a class.
fn footprint(dom: &leptotes::Dom) -> leptotes::Element<'_> {
    let outlines = dom.find_all(OUTLINES);
    assert!(!outlines.is_empty(), "no footprint in\n{dom}");
    outlines[0]
}

const RING: &str = r#"[data-testid="clearance-ring"]"#;
const TRUNK: &str = r#"circle[data-testid="tree-trunk"]"#;

fn render(objects: Vec<Object>, catalog: Vec<CatalogItem>) -> leptotes::Dom {
    leptotes::dom(move || view! { <Furnishings t=t() objects=objects catalog=catalog /> })
}

fn selected(objects: Vec<Object>, catalog: Vec<CatalogItem>, i: usize) -> leptotes::Dom {
    leptotes::dom(
        move || view! { <Furnishings t=t() objects=objects catalog=catalog selected=Some(i) /> },
    )
}

fn on_surface(
    objects: Vec<Object>,
    catalog: Vec<CatalogItem>,
    surface: Vec<Coord>,
) -> leptotes::Dom {
    leptotes::dom(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog surfaces=vec![surface] /> }
    })
}

fn on_house(objects: Vec<Object>, catalog: Vec<CatalogItem>, house: Vec<Coord>) -> leptotes::Dom {
    leptotes::dom(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog house_outline=house /> }
    })
}

fn near_structure(
    objects: Vec<Object>,
    catalog: Vec<CatalogItem>,
    outline: Vec<Coord>,
) -> leptotes::Dom {
    leptotes::dom(move || {
        view! { <Furnishings t=t() objects=objects catalog=catalog structure_outlines=vec![outline] /> }
    })
}

/// How many items carry a `furniture-item--{flag}` class.
fn flagged(dom: &leptotes::Dom, flag: &str) -> usize {
    dom.count(&format!("g.furniture-item.furniture-item--{flag}"))
}

fn at(x: f64, y: f64, id: &str) -> Object {
    Object::new(id.to_string(), x, y)
}

#[test]
fn renders_a_footprint_to_scale() {
    // A 3 ft × 1.5 ft item at (5,5): 10 px/ft → a 30 × 15 px rectangle centered
    // at sx(5)=50, sy(5)=150.
    let dom = render(
        vec![at(5.0, 5.0, "chair")],
        vec![item("chair", Some(3.0), Some(1.5))],
    );

    let group = dom.find("g.furnishings > g.furniture-item");
    let rect = footprint(&dom);

    assert_eq!(
        (
            rect.to_string().starts_with("<rect"),
            rect.attr("width"),
            rect.attr("height"),
            group
                .attr("transform")
                .is_some_and(|t| t.starts_with("translate(50,150)")),
        ),
        (true, Some("30"), Some("15"), true),
        "{dom}"
    );
}

#[test]
fn a_round_item_renders_a_circle_not_a_rect() {
    // A ⌀4 ft fire pit at (5,5): 10 px/ft → a circle of radius 20 px centered
    // at sx(5)=50, sy(5)=150.
    let dom = render(
        vec![at(5.0, 5.0, "fire-pit")],
        vec![round_item("fire-pit", 4.0)],
    );

    let footprint = footprint(&dom);

    assert_eq!(
        (
            footprint.to_string().starts_with("<circle"),
            footprint.attr("r"),
            dom.count("rect"),
            dom.find("g.furniture-item")
                .attr("transform")
                .is_some_and(|t| t.starts_with("translate(50,150)")),
        ),
        (true, Some("20"), 0, true),
        "{dom}"
    );
}

#[test]
fn an_existing_round_item_is_a_double_ring() {
    let mut obj = at(5.0, 5.0, "fire-pit");
    obj.status = ItemStatus::existing;
    let dom = render(vec![obj], vec![round_item("fire-pit", 4.0)]);

    // Existing → a second, inset ring; still circles, never rects.
    assert_eq!(
        (dom.count("g.furniture-item > circle"), dom.count("rect")),
        (2, 0),
        "{dom}"
    );
}

#[test]
fn a_round_item_with_no_clearance_shows_no_ring() {
    let dom = render(
        vec![at(5.0, 5.0, "fire-pit")],
        vec![round_item("fire-pit", 4.0)],
    );

    assert_eq!(dom.count(RING), 0, "{dom}");
}

#[test]
fn an_isolated_fire_pit_shows_a_quiet_dashed_ring() {
    // ⌀4 ft (radius 2 ft) + 3 ft clearance = 5 ft ring radius = 50 px, in the
    // quiet color: nothing intrudes.
    let dom = render(vec![at(5.0, 5.0, "fire-pit")], vec![fire_pit(4.0, 3.0)]);

    let ring = dom.find(RING);

    assert_eq!(
        (
            ring.attr("r"),
            ring.attr("stroke"),
            flagged(&dom, "intrudes")
        ),
        (Some("50"), Some("#8a8275"), 0),
        "{dom}"
    );
}

#[test]
fn a_nearby_object_turns_the_ring_red() {
    let catalog = vec![fire_pit(4.0, 3.0), item("chair", Some(2.0), Some(2.0))];
    // The chair is 2 ft away — inside the 5 ft ring.
    let dom = render(
        vec![at(5.0, 5.0, "fire-pit"), at(7.0, 5.0, "chair")],
        catalog,
    );

    let pit = dom.find("g.furniture-item.furniture-item--intrudes");

    assert_eq!(
        (flagged(&dom, "intrudes"), pit.find(RING).attr("stroke")),
        (1, Some("#7a1216")),
        "{dom}"
    );
}

#[test]
fn a_distant_object_does_not_turn_the_ring_red() {
    let catalog = vec![fire_pit(4.0, 3.0), item("chair", Some(2.0), Some(2.0))];
    let dom = render(
        vec![at(5.0, 5.0, "fire-pit"), at(20.0, 5.0, "chair")],
        catalog,
    );

    assert_eq!(
        (flagged(&dom, "intrudes"), dom.find(RING).attr("stroke")),
        (0, Some("#8a8275")),
        "{dom}"
    );
}

#[test]
fn a_nearby_structure_edge_turns_the_ring_red() {
    // A wall-like outline with an edge at x=9 — 4 ft from the fire pit's
    // center, inside the 5 ft clearance ring.
    let wall = vec![
        Coord::new(9.0, 0.0),
        Coord::new(9.0, 20.0),
        Coord::new(15.0, 20.0),
        Coord::new(15.0, 0.0),
    ];
    let dom = near_structure(
        vec![at(5.0, 5.0, "fire-pit")],
        vec![fire_pit(4.0, 3.0)],
        wall,
    );

    assert_eq!(flagged(&dom, "intrudes"), 1, "{dom}");
}

#[test]
fn renders_one_group_per_object() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let dom = render(vec![at(2.0, 2.0, "chair"), at(8.0, 8.0, "chair")], catalog);

    assert_eq!(dom.count("g.furnishings > g.furniture-item"), 2, "{dom}");
}

#[test]
fn rotation_is_applied_clockwise() {
    let mut obj = at(5.0, 5.0, "chair");
    obj.rot = Some(90.0);
    let dom = render(vec![obj], vec![item("chair", Some(2.0), Some(1.0))]);

    // Clockwise-from-north maps to SVG rotate(+rot).
    assert_eq!(
        dom.find("g.furniture-item").attr("transform"),
        Some("translate(50,150) rotate(90)"),
        "{dom}"
    );
}

#[test]
fn unresolved_catalog_ref_is_not_drawn() {
    // An object referencing an id absent from the catalog has no footprint to
    // draw, so nothing renders (mirrors the cost take-off's exclusion).
    let dom = render(
        vec![at(5.0, 5.0, "ghost-id")],
        vec![item("chair", Some(2.0), Some(2.0))],
    );

    assert_eq!(dom.count("*"), 0, "{dom}");
}

#[test]
fn existing_and_virtual_objects_still_render() {
    // status/is_virtual affect only the cost take-off, not visibility — every
    // placed object is shown on the plan.
    let mut existing = at(2.0, 2.0, "chair");
    existing.status = ItemStatus::existing;
    let mut ghost = at(8.0, 8.0, "chair");
    ghost.is_virtual = true;
    let dom = render(
        vec![existing, ghost],
        vec![item("chair", Some(2.0), Some(2.0))],
    );

    assert_eq!(dom.count("g.furniture-item"), 2, "{dom}");
}

/// One 2×2 ft chair with the given status, rendered alone.
fn chair(status: ItemStatus, is_virtual: bool) -> leptotes::Dom {
    let mut obj = at(5.0, 5.0, "chair");
    obj.status = status;
    obj.is_virtual = is_virtual;
    render(vec![obj], vec![item("chair", Some(2.0), Some(2.0))])
}

/// A rendered item's look: its status classes, the footprint's dash and fill
/// opacity, and how many outlines it draws.
fn look(dom: &leptotes::Dom) -> (Vec<&'static str>, Option<&str>, Option<&str>, usize) {
    let group = dom.find("g.furniture-item");
    let classes = [
        "furniture-item--planned",
        "furniture-item--existing",
        "furniture-item--virtual",
    ]
    .into_iter()
    .filter(|c| {
        group
            .attr("class")
            .is_some_and(|all| all.split_whitespace().any(|w| w == *c))
    })
    .collect();
    let outline = footprint(dom);
    (
        classes,
        outline.attr("stroke-dasharray"),
        outline.attr("fill-opacity"),
        dom.count(OUTLINES),
    )
}

#[test]
fn planned_real_is_a_single_solid_full_opacity_outline() {
    let dom = chair(ItemStatus::planned, false);

    assert_eq!(
        look(&dom),
        (
            vec!["furniture-item--planned"],
            Some("none"),
            Some("0.7"),
            1
        ),
        "{dom}"
    );
}

#[test]
fn existing_real_is_a_double_solid_outline() {
    let dom = chair(ItemStatus::existing, false);

    assert_eq!(
        look(&dom),
        (
            vec!["furniture-item--existing"],
            Some("none"),
            Some("0.7"),
            2
        ),
        "{dom}"
    );
}

#[test]
fn planned_virtual_is_a_single_dashed_ghost() {
    let dom = chair(ItemStatus::planned, true);

    assert_eq!(
        look(&dom),
        (
            vec!["furniture-item--planned", "furniture-item--virtual"],
            Some("4,3"),
            Some("0.35"),
            1
        ),
        "{dom}"
    );
}

#[test]
fn existing_virtual_is_a_double_dashed_ghost() {
    let dom = chair(ItemStatus::existing, true);

    assert_eq!(
        look(&dom),
        (
            vec!["furniture-item--existing", "furniture-item--virtual"],
            Some("4,3"),
            Some("0.35"),
            2
        ),
        "{dom}"
    );
}

#[test]
fn a_selected_existing_object_keeps_its_double_outline_under_the_selection_tint() {
    let mut obj = at(5.0, 5.0, "chair");
    obj.status = ItemStatus::existing;
    let dom = selected(vec![obj], vec![item("chair", Some(2.0), Some(2.0))], 0);

    assert_eq!(
        (footprint(&dom).attr("fill"), dom.count(OUTLINES)),
        (Some("#7ea9d4"), 2),
        "{dom}"
    );
}

#[test]
fn missing_dimensions_fall_back_to_a_default_footprint() {
    // A catalog item with no width/depth still places a visible 1 ft (= 10 px)
    // square so the object can be seen and selected.
    let dom = render(
        vec![at(5.0, 5.0, "mystery")],
        vec![item("mystery", None, None)],
    );

    let rect = footprint(&dom);

    assert_eq!(
        (rect.attr("width"), rect.attr("height")),
        (Some("10"), Some("10")),
        "{dom}"
    );
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
    let dom = on_surface(
        vec![at(5.0, 5.0, "chair")],
        vec![item("chair", Some(4.0), Some(4.0))],
        deck(6.0, 6.0),
    );

    assert_eq!(flagged(&dom, "overflows"), 1, "{dom}");
}

#[test]
fn an_object_fully_on_a_surface_is_not_highlighted() {
    // A 2×2 ft chair well inside a 10×10 ft deck fits.
    let dom = on_surface(
        vec![at(5.0, 5.0, "chair")],
        vec![item("chair", Some(2.0), Some(2.0))],
        deck(10.0, 10.0),
    );

    assert_eq!(flagged(&dom, "overflows"), 0, "{dom}");
}

#[test]
fn the_selected_object_is_highlighted() {
    let catalog = vec![item("chair", Some(2.0), Some(2.0))];
    let dom = selected(
        vec![at(2.0, 2.0, "chair"), at(8.0, 8.0, "chair")],
        catalog,
        1,
    );

    let marks: Vec<bool> = dom
        .find_all("g.furniture-item")
        .iter()
        .map(|g| {
            g.attr("class").is_some_and(|c| {
                c.split_whitespace()
                    .any(|w| w == "furniture-item--selected")
            })
        })
        .collect();

    assert_eq!(marks, [false, true], "{dom}");
}

#[test]
fn the_selected_object_shows_a_rotation_handle() {
    let dom = selected(
        vec![at(5.0, 5.0, "chair")],
        vec![item("chair", Some(2.0), Some(2.0))],
        0,
    );

    assert_eq!(
        dom.count(r#"g.furniture-item > g[data-testid="rotate-handle"]"#),
        1,
        "{dom}"
    );
}

#[test]
fn unselected_objects_have_no_rotation_handle() {
    let dom = render(
        vec![at(5.0, 5.0, "chair")],
        vec![item("chair", Some(2.0), Some(2.0))],
    );

    assert_eq!(dom.count(r#"[data-testid="rotate-handle"]"#), 0, "{dom}");
}

#[test]
fn a_selected_round_item_shows_no_rotation_handle() {
    // Rotating a circle is a visual no-op, so the handle would be a pointless
    // affordance — unlike a rectangular item, which still gets one.
    let dom = selected(
        vec![at(5.0, 5.0, "oak-tree")],
        vec![round_item("oak-tree", 20.0)],
        0,
    );

    assert_eq!(dom.count(r#"[data-testid="rotate-handle"]"#), 0, "{dom}");
}

/// The canopy and trunk resize handles an item shows.
fn resize_handles(dom: &leptotes::Dom) -> (usize, usize) {
    (
        dom.count(r#"circle[data-testid="canopy-handle"]"#),
        dom.count(r#"circle[data-testid="trunk-handle"]"#),
    )
}

#[test]
fn a_selected_tree_shows_canopy_and_trunk_resize_handles() {
    let dom = selected(
        vec![at(5.0, 5.0, "oak-tree")],
        vec![tree("oak-tree", 8.0, 2.0)],
        0,
    );

    assert_eq!(resize_handles(&dom), (1, 1), "{dom}");
}

#[test]
fn an_unselected_tree_shows_no_resize_handles() {
    let dom = render(
        vec![at(5.0, 5.0, "oak-tree")],
        vec![tree("oak-tree", 8.0, 2.0)],
    );

    assert_eq!(resize_handles(&dom), (0, 0), "{dom}");
}

#[test]
fn a_selected_fire_pit_shows_no_resize_handles() {
    // Resize handles are a tree-specific affordance (canopy/trunk); a fire pit
    // has neither, so it gets none.
    let dom = selected(vec![at(5.0, 5.0, "fire-pit")], vec![fire_pit(3.0, 1.5)], 0);

    assert_eq!(resize_handles(&dom), (0, 0), "{dom}");
}

// --- Grills: rectangular footprint + a shape-following clearance zone (D4) ---

fn a_grill() -> Vec<CatalogItem> {
    vec![grill("gas-grill", 4.0, 2.0, 1.5)]
}

#[test]
fn a_grill_renders_a_rounded_rect_clearance_zone_not_a_circle() {
    // 4×2 ft grill (40×20 px) at (5,5), 1.5 ft clearance (15 px): the zone is
    // the footprint grown by 15 px on every side (70×50) with 15 px rounded
    // corners — a rounded <rect>, never a <circle>.
    let dom = render(vec![at(5.0, 5.0, "gas-grill")], a_grill());

    let zone = dom.find(&format!("rect{RING}"));

    assert_eq!(
        (
            dom.count("circle"),
            zone.attr("rx"),
            zone.attr("width"),
            zone.attr("height"),
            footprint(&dom).attr("fill"),
        ),
        (0, Some("15"), Some("70"), Some("50"), Some("#3f4145")),
        "{dom}"
    );
}

#[test]
fn an_isolated_grill_zone_is_quiet() {
    let dom = render(vec![at(5.0, 5.0, "gas-grill")], a_grill());

    assert_eq!(
        (flagged(&dom, "intrudes"), dom.find(RING).attr("stroke")),
        (0, Some("#8a8275")),
        "{dom}"
    );
}

#[test]
fn a_grill_zone_flags_an_object_within_its_clearance() {
    // Grill rect x∈[3,7]. A 2×2 chair at (9,5) spans x∈[8,10]; its left edge is
    // 1 ft from the grill's right edge — inside the 1.5 ft keep-clear.
    let mut catalog = a_grill();
    catalog.push(item("chair", Some(2.0), Some(2.0)));
    let dom = render(
        vec![at(5.0, 5.0, "gas-grill"), at(9.0, 5.0, "chair")],
        catalog,
    );

    let grill = dom.find("g.furniture-item.furniture-item--intrudes");

    assert_eq!(
        (flagged(&dom, "intrudes"), grill.find(RING).attr("stroke")),
        (1, Some("#7a1216")),
        "{dom}"
    );
}

#[test]
fn a_distant_object_leaves_the_grill_zone_quiet() {
    let mut catalog = a_grill();
    catalog.push(item("chair", Some(2.0), Some(2.0)));
    let dom = render(
        vec![at(5.0, 5.0, "gas-grill"), at(15.0, 5.0, "chair")],
        catalog,
    );

    assert_eq!(flagged(&dom, "intrudes"), 0, "{dom}");
}

#[test]
fn a_grill_zone_flags_a_nearby_structure_edge() {
    // A wall edge at x=8 runs 1 ft from the grill's right edge (x=7) — inside
    // the 1.5 ft keep-clear.
    let wall = vec![
        Coord::new(8.0, 0.0),
        Coord::new(8.0, 20.0),
        Coord::new(15.0, 20.0),
        Coord::new(15.0, 0.0),
    ];
    let dom = near_structure(vec![at(5.0, 5.0, "gas-grill")], a_grill(), wall);

    assert_eq!(flagged(&dom, "intrudes"), 1, "{dom}");
}

// --- Hot tubs: a heavy unit that belongs on a surface (D5) ---

#[test]
fn a_hot_tub_fills_water_blue_and_must_sit_on_a_surface() {
    // A deck at (0,0)-(3,3) that does NOT contain the tub at (5,5): a hot tub
    // off a surface flags, since it belongs on a deck or paver.
    let dom = on_surface(
        vec![at(5.0, 5.0, "tub")],
        vec![hot_tub_round("tub", 4.0)],
        deck(3.0, 3.0),
    );

    assert_eq!(
        (footprint(&dom).attr("fill"), flagged(&dom, "overflows")),
        (Some("#6faec5"), 1),
        "{dom}"
    );
}

fn hot_tub_with_slab(id: &str, diameter: f64, thickness_in: f64, overhang_in: f64) -> CatalogItem {
    let mut c = hot_tub_round(id, diameter);
    c.slab_material_ref = Some("concrete".to_string());
    c.slab_thickness_in = Some(thickness_in);
    c.slab_overhang_in = Some(overhang_in);
    c
}

const PAD: &str = r#"rect[data-testid="hot-tub-pad"]"#;

#[test]
fn a_hot_tub_pours_a_gray_pad_larger_than_its_footprint() {
    // A 4 ft tub with a 12 in (1 ft) overhang at 10 px/ft → a 6×6 ft = 60 px
    // gray pad behind the 40 px footprint.
    let dom = render(
        vec![at(5.0, 5.0, "tub")],
        vec![hot_tub_with_slab("tub", 4.0, 4.0, 12.0)],
    );

    let pad = dom.find(PAD);

    assert_eq!(
        (
            pad.attr("fill"),
            pad.attr("width"),
            footprint(&dom).attr("r")
        ),
        (Some("#cfccc4"), Some("60"), Some("20")),
        "{dom}"
    );
}

#[test]
fn a_per_object_overhang_override_widens_the_pad() {
    // Overriding the overhang to 24 in (2 ft) grows the 4 ft tub's pad to 8×8 ft
    // = 80 px, over the catalog's 12 in default.
    let mut o = at(5.0, 5.0, "tub");
    o.slab_overhang_in = Some(24.0);
    let dom = render(vec![o], vec![hot_tub_with_slab("tub", 4.0, 4.0, 12.0)]);

    assert_eq!(dom.find(PAD).attr("width"), Some("80"), "{dom}");
}

#[test]
fn a_unit_without_a_slab_pours_no_pad() {
    let dom = render(
        vec![at(5.0, 5.0, "chair")],
        vec![item("chair", Some(4.0), Some(4.0))],
    );

    assert_eq!(dom.count(PAD), 0, "{dom}");
}

#[test]
fn a_hot_tub_fully_on_a_surface_is_quiet() {
    let dom = on_surface(
        vec![at(5.0, 5.0, "tub")],
        vec![hot_tub_round("tub", 4.0)],
        deck(10.0, 10.0),
    );

    assert_eq!(flagged(&dom, "overflows"), 0, "{dom}");
}

#[test]
fn no_surfaces_means_no_fit_check() {
    // Without surfaces there is nothing to fit within, so nothing is highlighted.
    let dom = render(
        vec![at(5.0, 5.0, "chair")],
        vec![item("chair", Some(4.0), Some(4.0))],
    );

    assert_eq!(flagged(&dom, "overflows"), 0, "{dom}");
}

#[test]
fn a_tree_renders_a_canopy_and_a_trunk() {
    // ⌀8 ft canopy (radius 40 px) in light green + ⌀2 ft trunk (radius 10 px)
    // in dark brown, at (5,5).
    let dom = render(
        vec![at(5.0, 5.0, "oak-tree")],
        vec![tree("oak-tree", 8.0, 2.0)],
    );

    let (canopy, trunk) = (footprint(&dom), dom.find(TRUNK));

    assert_eq!(
        (
            dom.count("circle"),
            [canopy.attr("r"), canopy.attr("fill")],
            [trunk.attr("r"), trunk.attr("fill")],
        ),
        (
            2,
            [Some("40"), Some("#a8d5a0")],
            [Some("10"), Some("#5a3a22")]
        ),
        "{dom}"
    );
}

#[test]
fn a_bush_renders_one_green_canopy_and_no_trunk() {
    // ⌀6 ft spread (radius 30 px) at (5,5): a single filled green disk — a bush
    // is a solid mass of foliage, so (unlike a tree) there's no trunk circle.
    let dom = render(vec![at(5.0, 5.0, "boxwood")], vec![bush("boxwood", 6.0)]);

    let canopy = footprint(&dom);

    assert_eq!(
        (dom.count("circle"), canopy.attr("r"), canopy.attr("fill")),
        (1, Some("30"), Some("#7cae83")),
        "{dom}"
    );
}

#[test]
fn a_bush_is_fine_on_bare_yard() {
    let dom = render(vec![at(5.0, 5.0, "boxwood")], vec![bush("boxwood", 6.0)]);

    assert_eq!(flagged(&dom, "overflows"), 0, "{dom}");
}

#[test]
fn a_bush_flags_its_whole_footprint_on_the_house() {
    // Any polygon containing (5,5) will do for the house; a bush on it reads
    // red, outlined in the overflow color.
    let dom = on_house(
        vec![at(5.0, 5.0, "boxwood")],
        vec![bush("boxwood", 6.0)],
        deck(10.0, 10.0),
    );

    assert_eq!(
        (flagged(&dom, "overflows"), footprint(&dom).attr("stroke")),
        (1, Some("#d4351c")),
        "{dom}"
    );
}

#[test]
fn a_bush_flags_on_a_deck_or_paver_surface() {
    // A shrub belongs on open ground, so a bush on a deck/paver reads red.
    let dom = on_surface(
        vec![at(5.0, 5.0, "boxwood")],
        vec![bush("boxwood", 6.0)],
        deck(10.0, 10.0),
    );

    assert_eq!(flagged(&dom, "overflows"), 1, "{dom}");
}

#[test]
fn a_fire_pit_fills_silver() {
    // Silver, not the shared furniture brown.
    let dom = render(vec![at(5.0, 5.0, "fire-pit")], vec![fire_pit(3.0, 1.5)]);

    assert_eq!(footprint(&dom).attr("fill"), Some("#b8b8bc"), "{dom}");
}

#[test]
fn a_tree_trunk_is_fine_on_bare_yard() {
    let dom = render(
        vec![at(5.0, 5.0, "oak-tree")],
        vec![tree("oak-tree", 8.0, 2.0)],
    );

    assert_eq!(
        (flagged(&dom, "trunk-invalid"), dom.find(TRUNK).attr("fill")),
        (0, Some("#5a3a22")),
        "{dom}"
    );
}

#[test]
fn a_tree_trunk_flags_on_the_house() {
    // The trunk itself turns the overflow red.
    let dom = on_house(
        vec![at(5.0, 5.0, "oak-tree")],
        vec![tree("oak-tree", 8.0, 2.0)],
        deck(10.0, 10.0),
    );

    assert_eq!(
        (flagged(&dom, "trunk-invalid"), dom.find(TRUNK).attr("fill")),
        (1, Some("#d4351c")),
        "{dom}"
    );
}

#[test]
fn a_tree_trunk_flags_when_its_disk_straddles_the_house_edge() {
    // ⌀2 ft trunk (radius 1 ft) centered 0.5 ft outside a house wall at x=5:
    // the center is off the house, but the trunk's disk still overlaps the
    // wall, so it should flag just like a fully-contained center would.
    let dom = on_house(
        vec![at(5.5, 5.0, "oak-tree")],
        vec![tree("oak-tree", 8.0, 2.0)],
        deck(5.0, 10.0),
    );

    assert_eq!(flagged(&dom, "trunk-invalid"), 1, "{dom}");
}

#[test]
fn a_tree_trunk_flags_on_the_deck_too() {
    // A tree's canopy may overhang the deck freely, but its trunk still
    // shouldn't be standing on it. The canopy is never flagged with the
    // generic overflow class.
    let dom = on_surface(
        vec![at(5.0, 5.0, "oak-tree")],
        vec![tree("oak-tree", 8.0, 2.0)],
        deck(10.0, 10.0),
    );

    assert_eq!(
        (flagged(&dom, "trunk-invalid"), flagged(&dom, "overflows")),
        (1, 0),
        "{dom}"
    );
}

#[test]
fn a_fire_pit_is_fine_off_a_deck() {
    // Unlike furniture, a fire pit doesn't need to be on a deck — the yard
    // (bare ground) is a valid surface for it.
    let dom = on_surface(
        vec![at(5.0, 5.0, "fire-pit")],
        vec![fire_pit(3.0, 1.5)],
        deck(2.0, 2.0),
    );

    assert_eq!(flagged(&dom, "overflows"), 0, "{dom}");
}

#[test]
fn a_fire_pit_flags_on_the_house() {
    let dom = on_house(
        vec![at(5.0, 5.0, "fire-pit")],
        vec![fire_pit(3.0, 1.5)],
        deck(10.0, 10.0),
    );

    assert_eq!(flagged(&dom, "overflows"), 1, "{dom}");
}

#[test]
fn a_per_object_canopy_and_trunk_override_wins_over_the_catalog_default() {
    let mut obj = at(5.0, 5.0, "oak-tree");
    obj.canopy_diameter_ft = Some(20.0); // → r=100 px
    obj.trunk_diameter_ft = Some(4.0); // → r=20 px
    let dom = render(vec![obj], vec![tree("oak-tree", 8.0, 2.0)]);

    assert_eq!(
        (footprint(&dom).attr("r"), dom.find(TRUNK).attr("r")),
        (Some("100"), Some("20")),
        "{dom}"
    );
}

#[test]
fn no_objects_renders_nothing() {
    let dom = leptotes::dom(move || view! { <Furnishings t=t() objects=Vec::new() /> });

    assert_eq!(dom.count("*"), 0, "{dom}");
}

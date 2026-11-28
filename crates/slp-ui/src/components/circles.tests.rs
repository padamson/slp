//! leptotes component tests for `Circles` (standalone round drawn areas).

use leptos::prelude::*;
use slp_core::{CatalogItem, Circle, Coord};

use super::{Circles, Transform};
use crate::style::SELECTED_FILL;

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 20.0,
    }
}

fn circle(elevation: f64) -> Circle {
    Circle {
        center: Box::new(Coord::new(5.0, 5.0)),
        elevation,
        radius_ft: 2.0,
        material_ref: None,
        depth_in: None,
        pattern: None,
        courses: Vec::new(),
        borders: Vec::new(),
    }
}

/// An area's filled disk: the circle in its group that is neither a border
/// ring nor the resize handle, both of which carry a test id.
const DISK: &str = "g.circle-area > circle:not([data-testid])";
const BORDER: &str = r#"circle[data-testid="circle-border"]"#;
const HANDLE: &str = r#"circle[data-testid="circle-resize-handle"]"#;

fn label(dom: &leptotes::Dom) -> String {
    dom.find("text.circle-label").text()
}

#[test]
fn renders_a_circle_with_area_diameter_and_no_elevation_suffix() {
    let dom = leptotes::dom(move || view! { <Circles t=t() circles=vec![circle(0.0)] /> });

    // radius 2 ft -> area = π·4 ≈ 13 ft², diameter 4 ft.
    assert_eq!(
        (dom.count("g.circles > g.circle-area > circle"), label(&dom)),
        (1, "13 ft² · ⌀4 ft".to_string()),
        "{dom}"
    );
}

#[test]
fn a_nonzero_elevation_appends_to_the_label() {
    let dom = leptotes::dom(move || view! { <Circles t=t() circles=vec![circle(1.5)] /> });

    assert_eq!(label(&dom), "13 ft² · ⌀4 ft · +1.5 ft", "{dom}");
}

#[test]
fn a_negative_elevation_shows_a_single_minus_sign() {
    let dom = leptotes::dom(move || view! { <Circles t=t() circles=vec![circle(-0.5)] /> });

    assert_eq!(label(&dom), "13 ft² · ⌀4 ft · -0.5 ft", "{dom}");
}

#[test]
fn renders_one_circle_per_entry() {
    let dom = leptotes::dom(move || {
        view! { <Circles t=t() circles=vec![circle(0.0), circle(2.0)] /> }
    });

    // Unselected, so each area is its disk alone, with no handle.
    assert_eq!(
        (dom.count("g.circle-area"), dom.count("circle")),
        (2, 2),
        "{dom}"
    );
}

#[test]
fn no_circles_renders_nothing() {
    let dom = leptotes::dom(move || view! { <Circles t=t() circles=Vec::new() /> });

    assert_eq!(dom.count("*"), 0, "{dom}");
}

#[test]
fn an_unselected_circle_has_no_resize_handle() {
    let dom = leptotes::dom(move || view! { <Circles t=t() circles=vec![circle(0.0)] /> });

    assert_eq!(
        (dom.count(HANDLE), dom.count(".circle-area--selected")),
        (0, 0),
        "{dom}"
    );
}

#[test]
fn a_selected_circle_shows_a_resize_handle() {
    let dom = leptotes::dom(move || {
        view! { <Circles t=t() circles=vec![circle(0.0)] selected=Some(0) /> }
    });

    assert_eq!(
        dom.find("g.circle-area.circle-area--selected")
            .count(HANDLE),
        1,
        "{dom}"
    );
}

#[test]
fn only_the_selected_circle_gets_a_handle() {
    let dom = leptotes::dom(move || {
        view! { <Circles t=t() circles=vec![circle(0.0), circle(0.0)] selected=Some(1) /> }
    });

    let handles: Vec<usize> = dom
        .find_all("g.circle-area")
        .iter()
        .map(|area| area.count(HANDLE))
        .collect();

    assert_eq!(handles, [0, 1], "{dom}");
}

/// A paver material carrying a small photo, for the tiling tests.
fn textured_paver() -> CatalogItem {
    let mut paver = CatalogItem::new("paver".to_string());
    paver.category = Some("paver".to_string());
    paver.image = Some("data:image/png;base64,AAAA".to_string());
    paver.tile_width_ft = Some(2.0);
    paver.tile_depth_ft = Some(2.0);
    paver
}

fn textured_circle() -> Circle {
    let mut c = circle(0.0);
    c.material_ref = Some("paver".to_string());
    c
}

// Like `Shapes`, a bare `<g>` root, so leptotes parses the fragment as SVG and
// the `<image>` and `patternUnits` survive.
#[test]
fn a_material_with_an_image_emits_a_user_space_pattern_of_its_photo() {
    let dom = leptotes::dom(move || {
        view! { <Circles t=t() circles=vec![textured_circle()] catalog=vec![textured_paver()] /> }
    });

    let pattern = dom.find("g.circles > defs > pattern#circle-mat-paver");

    assert_eq!(
        (
            pattern.attr("patternUnits"),
            pattern.find("image").attr("href")
        ),
        (Some("userSpaceOnUse"), Some("data:image/png;base64,AAAA")),
        "{dom}"
    );
}

#[test]
fn a_textured_disk_is_filled_by_its_material_s_pattern() {
    let dom = leptotes::dom(move || {
        view! { <Circles t=t() circles=vec![textured_circle()] catalog=vec![textured_paver()] /> }
    });

    assert_eq!(
        dom.find(DISK).attr("fill"),
        Some("url(#circle-mat-paver)"),
        "{dom}"
    );
}

#[test]
fn selection_overrides_the_texture() {
    // A selected textured circle shows the translucent selection tint, not the
    // photo — same precedence as the polygon areas.
    let dom = leptotes::dom(move || {
        view! {
            <Circles t=t() circles=vec![textured_circle()] catalog=vec![textured_paver()] selected=Some(0) />
        }
    });

    assert_eq!(dom.find(DISK).attr("fill"), Some(SELECTED_FILL), "{dom}");
}

#[test]
fn a_circle_border_renders_an_exact_annulus_stroke() {
    // Radius 2 ft (20 px) with a 0.5 ft ring: centerline radius
    // 2 − 0.25 = 1.75 ft → r=17.5 px, stroke-width 5 px.
    let mut c = circle(0.0);
    c.borders = vec![slp_core::Border::new("cobble".to_string(), 0.5)];
    let dom = leptotes::dom(move || view! { <Circles t=t() circles=vec![c.clone()] /> });

    let ring = dom.find(BORDER);

    assert_eq!(
        (ring.attr("r"), ring.attr("stroke-width")),
        (Some("17.5"), Some("5")),
        "{dom}"
    );
}

#[test]
fn an_oversized_circle_border_is_skipped() {
    // A ring wider than the radius has no positive centerline — nothing draws.
    let mut c = circle(0.0);
    c.borders = vec![slp_core::Border::new("cobble".to_string(), 5.0)];
    let dom = leptotes::dom(move || view! { <Circles t=t() circles=vec![c.clone()] /> });

    assert_eq!(dom.count(BORDER), 0, "{dom}");
}

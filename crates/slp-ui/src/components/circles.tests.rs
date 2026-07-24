//! dokime component tests for `Circles` (standalone round drawn areas).

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

#[test]
fn renders_a_circle_with_area_diameter_and_no_elevation_suffix() {
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![circle(0.0)] /> });
    assert!(html.contains(r#"class="circles""#), "tagged for queries");
    assert!(html.contains("<circle"), "the area is a circle");
    // radius 2 ft -> area = π·4 ≈ 13 ft², diameter 4 ft.
    assert!(
        html.contains("13 ft² · ⌀4 ft"),
        "area + diameter, no suffix"
    );
    assert!(!html.contains("ft · +"), "no elevation suffix at grade");
}

#[test]
fn a_nonzero_elevation_appends_to_the_label() {
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![circle(1.5)] /> });
    assert!(html.contains("13 ft² · ⌀4 ft · +1.5 ft"));
}

#[test]
fn a_negative_elevation_shows_a_single_minus_sign() {
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![circle(-0.5)] /> });
    assert!(
        html.contains("13 ft² · ⌀4 ft · -0.5 ft"),
        "single minus sign"
    );
    assert!(!html.contains("+-"), "never a doubled sign");
}

#[test]
fn renders_one_circle_per_entry() {
    let html = dokime::render(move || {
        view! { <Circles t=t() circles=vec![circle(0.0), circle(2.0)] /> }
    });
    // 2 filled circles + no handles (unselected) = 2 <circle> elements.
    assert_eq!(dokime::count(&html, "<circle"), 2);
}

#[test]
fn no_circles_renders_nothing() {
    let html = dokime::render(move || view! { <Circles t=t() circles=Vec::new() /> });
    assert!(!html.contains(r#"class="circles""#));
}

#[test]
fn an_unselected_circle_has_no_resize_handle() {
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![circle(0.0)] /> });
    assert_eq!(
        dokime::count(&html, r#"data-testid="circle-resize-handle""#),
        0
    );
    assert!(!html.contains("circle-area--selected"));
}

#[test]
fn a_selected_circle_shows_a_resize_handle() {
    let html = dokime::render(move || {
        view! { <Circles t=t() circles=vec![circle(0.0)] selected=Some(0) /> }
    });
    assert!(html.contains("circle-area--selected"));
    assert_eq!(
        dokime::count(&html, r#"data-testid="circle-resize-handle""#),
        1
    );
}

#[test]
fn only_the_selected_circle_gets_a_handle() {
    let html = dokime::render(move || {
        view! { <Circles t=t() circles=vec![circle(0.0), circle(0.0)] selected=Some(1) /> }
    });
    assert_eq!(
        dokime::count(&html, r#"data-testid="circle-resize-handle""#),
        1
    );
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

#[test]
fn a_material_with_an_image_tiles_the_disk_as_a_pattern() {
    // Like the polygon areas, a round area whose material carries a photo fills
    // with an SVG <pattern> at real-world scale and references it by
    // url(#circle-mat-{material id}), not the flat category color.
    let mut c = circle(0.0);
    c.material_ref = Some("paver".to_string());
    let html = dokime::render(move || {
        view! { <Circles t=t() circles=vec![c] catalog=vec![textured_paver()] /> }
    });

    assert!(html.contains("<pattern"), "emits an SVG pattern: {html}");
    assert!(
        html.contains(r#"patternUnits="userSpaceOnUse""#),
        "the pattern tiles in user (scaled) space"
    );
    assert!(
        html.contains("data:image/png;base64,AAAA"),
        "the pattern references the material image"
    );
    assert!(
        html.contains(r#"fill="url(#circle-mat-paver)""#),
        "the disk is filled by its material's pattern: {html}"
    );
}

#[test]
fn selection_overrides_the_texture() {
    // A selected textured circle shows the translucent selection tint, not the
    // photo — same precedence as the polygon areas.
    let mut c = circle(0.0);
    c.material_ref = Some("paver".to_string());
    let html = dokime::render(move || {
        view! { <Circles t=t() circles=vec![c] catalog=vec![textured_paver()] selected=Some(0) /> }
    });
    assert!(
        html.contains(SELECTED_FILL),
        "the selection tint wins: {html}"
    );
    assert!(
        !html.contains(r#"fill="url("#),
        "no pattern fill while selected"
    );
}

#[test]
fn a_circle_border_renders_an_exact_annulus_stroke() {
    // Radius 2 ft (20 px) with a 0.5 ft ring: centerline radius
    // 2 − 0.25 = 1.75 ft → r=17.5 px, stroke-width 5 px.
    let mut c = circle(0.0);
    c.borders = vec![slp_core::Border::new("cobble".to_string(), 0.5)];
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![c.clone()] /> });
    assert_eq!(
        dokime::count(&html, r#"data-testid="circle-border""#),
        1,
        "{html}"
    );
    assert!(html.contains(r#"r="17.5""#), "centerline radius: {html}");
    assert!(html.contains(r#"stroke-width="5""#), "ring width: {html}");
}

#[test]
fn an_oversized_circle_border_is_skipped() {
    // A ring wider than the radius has no positive centerline — nothing draws.
    let mut c = circle(0.0);
    c.borders = vec![slp_core::Border::new("cobble".to_string(), 5.0)];
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![c.clone()] /> });
    assert_eq!(dokime::count(&html, r#"data-testid="circle-border""#), 0);
}

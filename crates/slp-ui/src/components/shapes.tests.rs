//! leptotes component tests for `Shapes` (drawn paver/mulch/… areas).

use leptos::prelude::*;
use slp_core::{CatalogItem, Coord, CurveEdge, Shape};

use super::{Shapes, Transform};
use crate::style::{MULCH_FILL, PAVER_FILL, SELECTED_FILL, SHAPE_FILL};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 20.0,
    }
}

fn square(elevation: f64) -> Shape {
    Shape {
        corners: vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(4.0, 3.0),
            Coord::new(0.0, 3.0),
        ],
        elevation,
        bulges: Vec::new(),
        curves: Vec::new(),
        material_ref: None,
        depth_in: None,
        pattern: None,
        courses: Vec::new(),
        borders: Vec::new(),
    }
}

/// An area's filled body: the polygon (or, once an edge curves, the path)
/// directly under its group. A border's ribbons and joints sit there too, and
/// its clip path holds a second copy of the boundary one level down.
const BODY: &str = r#"g.shape > polygon:not([data-testid="shape-border-joint"]), g.shape > path:not([data-testid="shape-border"])"#;
const RIBBON: &str = r#"path[data-testid="shape-border"]"#;
const JOINT: &str = r#"polygon[data-testid="shape-border-joint"]"#;

fn label(dom: &leptotes::Dom) -> String {
    dom.find("text.shape-label").text()
}

/// The area (ft²) a label reports, the number before its unit.
fn labelled_area(dom: &leptotes::Dom) -> f64 {
    let label = label(dom);
    label
        .split_whitespace()
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("no area in label {label:?}"))
}

/// The coordinates of the one border ribbon's path, flattened.
fn ribbon_coords(dom: &leptotes::Dom) -> Vec<f64> {
    dom.find(RIBBON)
        .attr("d")
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|tok| tok.parse().ok())
        .collect()
}

#[test]
fn renders_a_shape_with_markers_and_area_label() {
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });

    // 4 ft x 3 ft = 12 ft², elevation 0 -> no elevation suffix.
    assert_eq!(
        (
            dom.count("g.shapes > g.shape > polygon"),
            dom.count("circle.shape-corner"),
            label(&dom)
        ),
        (1, 4, "12 ft²".to_string()),
        "{dom}"
    );
}

#[test]
fn a_nonzero_elevation_appends_to_the_label() {
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(1.5)] /> });

    assert_eq!(label(&dom), "12 ft² · +1.5 ft", "{dom}");
}

#[test]
fn a_negative_elevation_shows_a_single_minus_sign() {
    // A below-grade area (e.g. a sunken paver patio) reads "-0.5 ft", not the
    // doubled-up "+-0.5 ft" a naive `+{elevation}` format would produce.
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(-0.5)] /> });

    assert_eq!(label(&dom), "12 ft² · -0.5 ft", "{dom}");
}

#[test]
fn renders_one_polygon_per_shape() {
    let dom =
        leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(0.0), square(2.0)] /> });

    assert_eq!(dom.count("g.shape > polygon"), 2, "{dom}");
}

#[test]
fn no_shapes_renders_nothing() {
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=Vec::new() /> });

    assert_eq!(dom.count("*"), 0, "{dom}");
}

#[test]
fn a_bowed_edge_renders_a_path_with_an_arc_command_not_a_polygon() {
    // Bow the first edge (0,0)->(4,0) into an arc — the whole boundary becomes
    // a <path> with one `A` arc command (the one bowed edge), no <polygon>.
    let mut s = square(0.0);
    s.bulges = vec![0.5, 0.0, 0.0, 0.0];
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![s] /> });

    let body = dom.find(BODY);
    let arcs = body
        .attr("d")
        .unwrap_or_default()
        .split_whitespace()
        .filter(|tok| *tok == "A")
        .count();

    assert_eq!(
        (body.to_string().starts_with("<path"), arcs),
        (true, 1),
        "{dom}"
    );
}

#[test]
fn a_bowed_edge_changes_the_reported_area() {
    // Bowing an edge outward (negative bulge = away from a CCW interior) grows
    // the area past the straight 12 ft²; the label reflects it.
    let mut s = square(0.0);
    s.bulges = vec![-1.0, 0.0, 0.0, 0.0]; // bottom edge bows out into a semicircle
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![s] /> });

    let area = labelled_area(&dom);

    assert!(
        area > 12.0,
        "an outward bow grows the area; got {area} in\n{dom}"
    );
}

#[test]
fn a_mulch_bed_renders_in_the_mulch_color() {
    // A shape whose material resolves (through the catalog) to a "mulch-bed"
    // category fills mulch brown.
    let mut mulch = CatalogItem::new("mulch".to_string());
    mulch.category = Some("mulch-bed".to_string());
    let mut bed = square(0.0);
    bed.material_ref = Some("mulch".to_string());
    let dom =
        leptotes::dom(move || view! { <Shapes t=t() shapes=vec![bed] catalog=vec![mulch] /> });

    assert_eq!(dom.find(BODY).attr("fill"), Some(MULCH_FILL), "{dom}");
}

#[test]
fn an_uncategorized_area_keeps_the_neutral_fill() {
    let mut mulch = CatalogItem::new("mulch".to_string());
    mulch.category = Some("mulch-bed".to_string());
    let dom = leptotes::dom(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0)] catalog=vec![mulch] /> }
    });

    assert_eq!(dom.find(BODY).attr("fill"), Some(SHAPE_FILL), "{dom}");
}

#[test]
fn a_paver_area_renders_in_the_paver_color() {
    let mut paver = CatalogItem::new("paver".to_string());
    paver.category = Some("paver".to_string());
    let mut area = square(0.0);
    area.material_ref = Some("paver".to_string());
    let dom =
        leptotes::dom(move || view! { <Shapes t=t() shapes=vec![area] catalog=vec![paver] /> });

    assert_eq!(dom.find(BODY).attr("fill"), Some(PAVER_FILL), "{dom}");
}

/// A paver material carrying a small photo, for the tiling tests.
fn textured_paver() -> CatalogItem {
    let mut paver = CatalogItem::new("paver".to_string());
    paver.category = Some("paver".to_string());
    paver.image = Some("data:image/png;base64,AAAA".to_string());
    paver.tile_width_ft = Some(2.0);
    paver.tile_depth_ft = Some(3.0);
    paver
}

fn textured_area() -> Shape {
    let mut area = square(0.0);
    area.material_ref = Some("paver".to_string());
    area
}

// The `<pattern>`'s `patternUnits` and the `<image>` inside it are what an
// HTML parse would have changed (to `patternunits` and `<img>`): `Shapes`
// renders a bare `<g>`, so these queries rely on leptotes parsing it as SVG.
#[test]
fn a_material_with_an_image_tiles_the_surface_as_a_pattern() {
    let dom = leptotes::dom(move || {
        view! { <Shapes t=t() shapes=vec![textured_area()] catalog=vec![textured_paver()] /> }
    });

    let pattern = dom.find("g.shapes > defs > pattern#area-mat-paver");
    let image = pattern.find("image");

    // A 2 ft × 3 ft tile at 10 px/ft is 20 × 30 px, anchored at the world
    // origin (sx(0)=pad=0, sy(0)=pad+yard_d·px_ft=200) so the tile grid stays
    // glued to world coordinates as the yard resizes.
    assert_eq!(
        (
            pattern.attr("patternUnits"),
            [pattern.attr("x"), pattern.attr("y")],
            [pattern.attr("width"), pattern.attr("height")],
            image.attr("href"),
            [image.attr("width"), image.attr("height")],
            image.attr("preserveAspectRatio"),
        ),
        (
            Some("userSpaceOnUse"),
            [Some("0"), Some("200")],
            [Some("20"), Some("30")],
            Some("data:image/png;base64,AAAA"),
            [Some("20"), Some("30")],
            Some("none"),
        ),
        "{dom}"
    );
}

#[test]
fn a_textured_area_is_filled_by_its_material_s_pattern() {
    // The pattern, not the flat paver color.
    let dom = leptotes::dom(move || {
        view! { <Shapes t=t() shapes=vec![textured_area()] catalog=vec![textured_paver()] /> }
    });

    assert_eq!(
        dom.find(BODY).attr("fill"),
        Some("url(#area-mat-paver)"),
        "{dom}"
    );
}

#[test]
fn areas_sharing_a_material_share_one_pattern() {
    // Two paver areas → a single <pattern> (per material, not per area), so a
    // photo's data-URI is embedded once no matter how many areas tile it.
    let dom = leptotes::dom(move || {
        view! {
            <Shapes t=t() shapes=vec![textured_area(), textured_area()] catalog=vec![textured_paver()] />
        }
    });

    assert_eq!(
        (dom.count("pattern"), dom.attrs(BODY, "fill")),
        (1, vec!["url(#area-mat-paver)"; 2]),
        "{dom}"
    );
}

#[test]
fn selection_overrides_the_texture() {
    // A selected textured area shows the translucent selection tint, not the
    // photo — the user needs selection feedback (and to see the grid/deck
    // beneath) while editing.
    let dom = leptotes::dom(move || {
        view! {
            <Shapes t=t() shapes=vec![textured_area()] catalog=vec![textured_paver()] selected=Some(0) />
        }
    });

    assert_eq!(dom.find(BODY).attr("fill"), Some(SELECTED_FILL), "{dom}");
}

fn bezier_square() -> Shape {
    let mut s = square(0.0);
    s.curves = vec![CurveEdge {
        edge: 0,
        control1: Box::new(Coord::new(1.0, -2.0)),
        control2: Box::new(Coord::new(3.0, -2.0)),
    }];
    s
}

#[test]
fn a_curved_edge_renders_a_path_with_a_bezier_command() {
    // Give edge 0 a cubic bezier — the boundary becomes a <path> with a `C`
    // command, no <polygon>.
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![bezier_square()] /> });

    let body = dom.find(BODY);
    let beziers = body
        .attr("d")
        .unwrap_or_default()
        .split_whitespace()
        .filter(|tok| *tok == "C")
        .count();

    assert_eq!(
        (body.to_string().starts_with("<path"), beziers),
        (true, 1),
        "{dom}"
    );
}

#[test]
fn a_curved_edge_changes_the_reported_area() {
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![bezier_square()] /> });

    let area = labelled_area(&dom);

    assert!(
        (area - 12.0).abs() > 0.1,
        "the area accounts for the curve; got {area} in\n{dom}"
    );
}

#[test]
fn skips_a_degenerate_shape_with_too_few_corners() {
    let degenerate = Shape {
        corners: vec![Coord::new(5.0, 5.0), Coord::new(6.0, 5.0)],
        ..square(0.0)
    };
    let dom = leptotes::dom(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0), degenerate] /> }
    });

    // A shape with under 3 corners has no area to render.
    assert_eq!(dom.count("g.shape"), 1, "{dom}");
}

#[test]
fn an_unselected_shape_has_plain_corner_markers() {
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });

    assert_eq!(
        (
            dom.count("circle.shape-corner"),
            dom.count(r#"[data-testid="shape-node"]"#),
            dom.count(".shape--selected"),
        ),
        (4, 0, 0),
        "{dom}"
    );
}

#[test]
fn a_selected_shape_shows_interactive_node_handles_instead() {
    let dom =
        leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> });

    assert_eq!(
        (
            dom.count("g.shape.shape--selected"),
            dom.count("circle.shape-corner"),
            dom.count(r#"circle[data-testid="shape-node"]"#),
        ),
        (1, 0, 4),
        "{dom}"
    );
}

#[test]
fn a_selected_shape_shows_a_bulge_handle_per_edge() {
    // The square has 4 edges, so 4 edge (bulge) handles when selected.
    let dom =
        leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> });

    assert_eq!(
        dom.count(r#"[data-testid="shape-edge-handle"]"#),
        4,
        "{dom}"
    );
}

#[test]
fn an_unselected_shape_shows_no_bulge_handles() {
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });

    assert_eq!(
        dom.count(r#"[data-testid="shape-edge-handle"]"#),
        0,
        "{dom}"
    );
}

#[test]
fn a_selected_shape_shows_two_control_handles_per_straight_edge() {
    // A straight square: 4 edges × 2 control handles = 8. Straight edges draw
    // no guide line (the controls sit on the chord).
    let dom =
        leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> });

    assert_eq!(
        (
            dom.count(r#"[data-testid="shape-control-handle"]"#),
            dom.count("line.shape-control-guide"),
        ),
        (8, 0),
        "{dom}"
    );
}

#[test]
fn a_bezier_edge_shows_control_handles_with_guides_and_no_apex() {
    // The bezier edge shows 2 control handles + 2 guide lines and NO apex
    // handle; the other 3 straight edges keep their apex + 2 controls each.
    // So: apex handles 3, control handles 2 + 6 = 8, guide lines 2.
    let dom = leptotes::dom(
        move || view! { <Shapes t=t() shapes=vec![bezier_square()] selected=Some(0) /> },
    );

    assert_eq!(
        (
            dom.count(r#"[data-testid="shape-edge-handle"]"#),
            dom.count(r#"[data-testid="shape-control-handle"]"#),
            dom.count("line.shape-control-guide"),
        ),
        (3, 8, 2),
        "{dom}"
    );
}

#[test]
fn only_the_selected_shape_gets_node_handles() {
    let dom = leptotes::dom(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0), square(2.0)] selected=Some(1) /> }
    });

    let shapes = dom.find_all("g.shape");

    assert_eq!(
        [
            (
                shapes[0].count("circle.shape-corner"),
                shapes[0].count(r#"[data-testid="shape-node"]"#)
            ),
            (
                shapes[1].count("circle.shape-corner"),
                shapes[1].count(r#"[data-testid="shape-node"]"#)
            ),
        ],
        [(4, 0), (0, 4)],
        "{dom}"
    );
}

#[test]
fn no_popup_with_fewer_than_two_selected_nodes() {
    let dom = leptotes::dom(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) selected_nodes=vec![1] /> }
    });

    assert_eq!(dom.count(".node-insert-popup"), 0, "{dom}");
}

#[test]
fn a_pair_of_selected_nodes_shows_the_insert_cancel_popup() {
    let dom = leptotes::dom(move || {
        view! {
            <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) selected_nodes=vec![0, 1] />
        }
    });

    let popup = dom.find("g.node-insert-popup");

    assert_eq!(
        (
            popup.count(r#"[data-testid="insert-node"]"#),
            popup.count(r#"[data-testid="cancel-node-select"]"#),
        ),
        (1, 1),
        "{dom}"
    );
}

#[test]
fn border_rings_render_clipped_ribbons() {
    // A 4×3 area (10 px/ft) with two full rings — outer 0.5 ft, inner
    // 0.25 ft: each renders as its own closed annulus ribbon, filled with the
    // band paint and clipped to the area. Only the body outline strokes.
    let mut s = square(0.0);
    s.borders = vec![
        slp_core::Border::new("cobble".to_string(), 0.5),
        slp_core::Border::new("edging".to_string(), 0.25),
    ];
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![s.clone()] /> });

    let clip = dom
        .find("g.shape > clipPath")
        .attr("id")
        .unwrap_or_default();

    assert_eq!(
        (dom.attrs(RIBBON, "clip-path"), dom.count("[stroke-width]"),),
        (vec![format!("url(#{clip})").as_str(); 2], 1),
        "{dom}"
    );
}

#[test]
fn a_borderless_shape_renders_no_ring_strokes() {
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });

    assert_eq!((dom.count(RIBBON), dom.count("clipPath")), (0, 0), "{dom}");
}

#[test]
fn a_span_border_renders_a_one_sided_ribbon() {
    // A border from node 0 to node 2 renders as a filled ribbon covering only
    // those edges, offset to the interior side. The square's interior is
    // north of edge 0 (screen y < 200) and west of edge 1 (x < 40): every
    // ribbon coordinate stays on/inside the boundary — never south or east
    // of it, which the old centered stroke would have painted.
    let mut shape = square(0.0);
    let mut band = slp_core::Border::new("edging".to_string(), 0.5);
    band.start_node = Some(0.0);
    band.end_node = Some(2.0);
    shape.borders = vec![band];
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![shape.clone()] /> });

    let coords = ribbon_coords(&dom);

    assert!(
        coords.len() >= 8
            && coords
                .chunks(2)
                .all(|p| p[0] <= 40.0 + 1e-6 && p[1] <= 200.0 + 1e-6),
        "the ribbon has real geometry, all on the interior side: {coords:?}"
    );
}

#[test]
fn a_degenerate_span_border_draws_nothing() {
    let mut s = square(0.0);
    let mut b = slp_core::Border::new("edging".to_string(), 0.5);
    b.start_node = Some(0.0);
    b.end_node = Some(9.0); // out of range
    s.borders = vec![b];
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![s.clone()] /> });

    assert_eq!(dom.count(RIBBON), 0, "{dom}");
}

#[test]
fn a_selected_shape_labels_its_node_indices() {
    // The border editor's From/To fields refer to node numbers, so a selected
    // shape shows each handle's index.
    let dom = leptotes::dom(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> }
    });

    assert_eq!(
        dom.texts(r#"text[data-testid="shape-node-index"]"#),
        ["0", "1", "2", "3"],
        "{dom}"
    );
}

#[test]
fn an_unselected_shape_leaves_its_nodes_unlabeled() {
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });

    assert_eq!(dom.count(r#"[data-testid="shape-node-index"]"#), 0, "{dom}");
}

#[test]
fn disjoint_span_bands_do_not_inherit_each_others_depth() {
    // The manual-testing bug: two 0.5 ft bands on edges 3→4 and a third
    // 0.5 ft band on edges 2→3 (a 5-node boundary). The 2→3 band must stroke
    // at its own depth (0.5), not the cumulative 1.5 — nothing covers its
    // edges to overpaint the excess.
    let borders = [
        (0.5, Some((3.0, 4.0))),
        (0.5, Some((3.0, 4.0))),
        (0.5, Some((2.0, 3.0))),
    ];
    let strokes = super::shapes::border_strokes(&borders, 5);
    assert_eq!(strokes.len(), 3);
    // Deepest first: the nested second 3→4 band (0.5 offset + 0.5).
    assert_eq!(strokes[0].border, 1);
    assert!((strokes[0].depth_ft - 1.0).abs() < 1e-9);
    let third = strokes.iter().find(|s| s.border == 2).unwrap();
    assert!(
        (third.depth_ft - 0.5).abs() < 1e-9,
        "the 2→3 band stays at its own width: {}",
        third.depth_ft
    );
    assert_eq!(third.run, Some((2.0, 3.0)));
}

#[test]
fn a_ring_after_a_span_splits_where_the_offset_changes() {
    // A 0.5 ft span on edge 0→1, then a 0.25 ft full ring: the ring nests
    // under the span only on edge 0, so it splits into a (0→1) run at depth
    // 0.75 and a (1→0) run at depth 0.25.
    let borders = [(0.5, Some((0.0, 1.0))), (0.25, None)];
    let strokes = super::shapes::border_strokes(&borders, 4);
    let ring_runs: Vec<_> = strokes.iter().filter(|s| s.border == 1).collect();
    assert_eq!(ring_runs.len(), 2, "the ring splits into two runs");
    let deep = ring_runs
        .iter()
        .find(|s| s.run == Some((0.0, 1.0)))
        .unwrap();
    assert!((deep.depth_ft - 0.75).abs() < 1e-9, "nested under the span");
    let shallow = ring_runs
        .iter()
        .find(|s| s.run == Some((1.0, 0.0)))
        .unwrap();
    assert!(
        (shallow.depth_ft - 0.25).abs() < 1e-9,
        "at the edge elsewhere"
    );
}

#[test]
fn nested_full_rings_stay_single_closed_strokes() {
    let borders = [(0.5, None), (0.25, None)];
    let strokes = super::shapes::border_strokes(&borders, 4);
    assert_eq!(strokes.len(), 2);
    assert!(strokes.iter().all(|s| s.run.is_none()), "closed rings");
    assert!(
        (strokes[0].depth_ft - 0.75).abs() < 1e-9,
        "inner painted first"
    );
    assert!((strokes[1].depth_ft - 0.5).abs() < 1e-9);
}

#[test]
fn spans_meeting_at_a_convex_corner_need_no_junction_patch() {
    // Two spans meeting at node 1 of the plain rectangle: a convex corner —
    // butt-ended bands overlap naturally, so no joint patch is emitted (a
    // square cap here would have overshot; the strokes are plain butt).
    // Ribbons are filled regions, so only the body outline strokes.
    let mut shape = square(0.0);
    let mut south = slp_core::Border::new("edging".to_string(), 0.5);
    south.start_node = Some(0.0);
    south.end_node = Some(1.0);
    let mut east = slp_core::Border::new("edging".to_string(), 0.5);
    east.start_node = Some(1.0);
    east.end_node = Some(2.0);
    shape.borders = vec![south, east];
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![shape.clone()] /> });

    assert_eq!(
        (
            dom.count(RIBBON),
            dom.count("[stroke-linecap]"),
            dom.count(JOINT),
            dom.count("[stroke-width]"),
        ),
        (2, 0, 0, 1),
        "{dom}"
    );
}

/// An L-shaped area (reflex corner at node 3, (2,1)).
fn ell(elevation: f64) -> Shape {
    Shape {
        corners: vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(4.0, 1.0),
            Coord::new(2.0, 1.0),
            Coord::new(2.0, 3.0),
            Coord::new(0.0, 3.0),
        ],
        ..square(elevation)
    }
}

/// Two bands of `width` on the L's edges 2→3 and 3→4, which meet at its
/// reflex corner.
fn ell_with_notch_bands(width: f64) -> Shape {
    let mut s = ell(0.0);
    let mut a = slp_core::Border::new("edging".to_string(), width);
    a.start_node = Some(2.0);
    a.end_node = Some(3.0);
    let mut b = slp_core::Border::new("edging".to_string(), width);
    b.start_node = Some(3.0);
    b.end_node = Some(4.0);
    s.borders = vec![a, b];
    s
}

#[test]
fn spans_meeting_at_a_reflex_corner_get_a_miter_joint_patch() {
    // Butt caps leave a pie gap at the reflex corner, filled by exactly one
    // junction quad.
    let s = ell_with_notch_bands(0.5);
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![s.clone()] /> });

    assert_eq!((dom.count(RIBBON), dom.count(JOINT)), (2, 1), "{dom}");
}

#[test]
fn junction_quad_fills_reflex_corners_only() {
    use super::shapes::junction_quad;
    // Screen space, y down. A reflex corner as seen from the bands: edge prev
    // travels up (0,-1) with its band to the left (inward (-1,0)); edge cur
    // travels right (1,0) with its band above (inward (0,-1)).
    let quad = junction_quad(
        (100.0, 100.0),
        (0.0, -1.0),
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, -1.0),
        5.0,
        10.0,
    )
    .expect("a reflex corner needs a patch");
    assert_eq!(quad[0], (100.0, 100.0), "anchored at the node");
    assert_eq!(quad[1], (95.0, 100.0), "prev band's inner corner");
    assert_eq!(quad[3], (100.0, 90.0), "cur band's inner corner");
    assert_eq!(quad[2], (95.0, 90.0), "the miter point");
    // A convex corner (bands wrap the inside of a rectangle corner): no gap.
    assert!(
        junction_quad(
            (100.0, 100.0),
            (1.0, 0.0),
            (0.0, 1.0),
            (0.0, 1.0),
            (-1.0, 0.0),
            5.0,
            5.0,
        )
        .is_none(),
        "convex corners overlap already"
    );
    // Collinear: the butt faces coincide.
    assert!(
        junction_quad(
            (100.0, 100.0),
            (1.0, 0.0),
            (1.0, 0.0),
            (0.0, 1.0),
            (0.0, 1.0),
            5.0,
            5.0,
        )
        .is_none(),
        "collinear needs no patch"
    );
}

#[test]
fn edge_band_depths_sum_stacked_widths_per_edge() {
    use super::shapes::edge_band_depths;
    // The manual-testing scenario: two bands on edge 3, one on edge 2 (n=5).
    let borders = [
        (0.5, Some((3.0, 4.0))),
        (0.5, Some((3.0, 4.0))),
        (0.5, Some((2.0, 3.0))),
    ];
    let d = edge_band_depths(&borders, 5);
    assert!((d[3].0 - 1.0).abs() < 1e-9, "edge 3 stacks both bands");
    assert_eq!(d[3].1, Some(1), "its innermost band is the second");
    assert!((d[2].0 - 0.5).abs() < 1e-9);
    assert_eq!(d[2].1, Some(2));
    assert!(d[0].0.abs() < 1e-9, "uncovered edges are zero");
    assert_eq!(d[0].1, None);
}

#[test]
fn edge_end_tangents_follow_the_edge_kind() {
    use super::shapes::edge_end_tangents;
    let approx =
        |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9;
    // Straight: both tangents are the chord (screen y flips world y).
    let corners = vec![
        Coord::new(0.0, 0.0),
        Coord::new(4.0, 0.0),
        Coord::new(4.0, 3.0),
    ];
    let (s0, e0) = edge_end_tangents(&corners, &[], &[], 0).unwrap();
    assert!(approx(s0, (1.0, 0.0)) && approx(e0, (1.0, 0.0)));
    // A semicircle (|b| = 1, θ = π): tangents turn ±90° off the chord. A
    // positive bulge bows left of travel in world (up here) = screen −y.
    let (s1, e1) = edge_end_tangents(&corners, &[1.0], &[], 0).unwrap();
    assert!(
        approx(s1, (0.0, -1.0)),
        "start turns toward the bow: {s1:?}"
    );
    assert!(approx(e1, (0.0, 1.0)), "end turns back: {e1:?}");
    // A Bézier: tangents point along the control arms.
    let curves = vec![CurveEdge::new(
        Box::new(Coord::new(0.0, -3.0)),
        Box::new(Coord::new(4.0, -3.0)),
        0,
    )];
    let (s2, e2) = edge_end_tangents(&corners, &[], &curves, 0).unwrap();
    assert!(
        approx(s2, (0.0, 1.0)),
        "start dives toward control1 (world −y = screen +y): {s2:?}"
    );
    assert!(approx(e2, (0.0, -1.0)), "end climbs from control2: {e2:?}");
}

#[test]
fn a_curved_edge_junction_uses_the_curve_tangent_not_its_chord() {
    // The manual-testing bug: curve the L's notch edge (2→3, sagging into the
    // strip) and the junction patch at node 3 must follow the curve's END
    // tangent. Edge 2 runs (4,1) → (2,1); controls pulled up to y=2 sag it
    // into the interior of the notch strip above.
    let mut s = ell_with_notch_bands(0.25);
    s.curves = vec![CurveEdge::new(
        Box::new(Coord::new(3.5, 2.0)),
        Box::new(Coord::new(2.5, 2.0)),
        2,
    )];
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![s.clone()] /> });

    let vertices: Vec<(f64, f64)> = dom
        .find(JOINT)
        .attr("points")
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|p| {
            let (x, y) = p.split_once(',')?;
            Some((x.parse().ok()?, y.parse().ok()?))
        })
        .collect();
    let has = |(ex, ey): (f64, f64)| {
        vertices
            .iter()
            .any(|&(x, y)| (x - ex).abs() < 1e-6 && (y - ey).abs() < 1e-6)
    };

    // The corner stays reflex, so the patch remains, but at the band's inner
    // corner v + inward(tangent)·depth ≈ (22.236, 191.118), not the chord's
    // (20, 192.5), the misplaced stub seen in manual testing.
    assert_eq!(
        (
            dom.count(RIBBON),
            has((22.236_067_977_499_79, 191.118_033_988_749_9)),
            has((20.0, 192.5)),
        ),
        (2, true, false),
        "{vertices:?} in\n{dom}"
    );
}

#[test]
fn an_arc_edge_band_hugs_the_real_arc_not_its_mirror() {
    // Edge 0 of the square bulges into the interior as a semicircle (bulge 1,
    // apex world (2,2) → screen (20,180)). The band's ribbon must trace that
    // arc — every coordinate stays at or above the square's base (screen
    // y ≤ 200) and reaches the apex; the mirrored sweep seen in manual testing
    // put the arc at world (2,−2) → screen y ≈ 220, sashing outside/below.
    let mut shape = square(0.0);
    shape.bulges = vec![1.0];
    let mut band = slp_core::Border::new("edging".to_string(), 0.5);
    band.start_node = Some(0.0);
    band.end_node = Some(1.0);
    shape.borders = vec![band];
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![shape.clone()] /> });

    let ys: Vec<f64> = ribbon_coords(&dom).chunks(2).map(|p| p[1]).collect();
    let (min_y, max_y) = ys
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), &y| (lo.min(y), hi.max(y)));

    assert!(
        max_y <= 200.5 && min_y < 182.0,
        "the ribbon spans the true arc, base to apex; got y in [{min_y}, {max_y}]"
    );
}

#[test]
fn a_mid_edge_span_renders_a_fractional_ribbon() {
    // A span from position 0.5 (midpoint of edge 0) to 1.5 (midpoint of edge
    // 1) renders one ribbon whose outer path starts at the edge-0 midpoint
    // (screen (20, 200) on the 4×3 square at 10 px/ft) — a mid-edge start a
    // node-granular span could not express.
    let mut shape = square(0.0);
    let mut band = slp_core::Border::new("edging".to_string(), 0.5);
    band.start_node = Some(0.5);
    band.end_node = Some(1.5);
    shape.borders = vec![band];
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![shape.clone()] /> });

    let d = dom.find(RIBBON).attr("d").unwrap_or_default();

    assert!(
        d.starts_with("M 20 200 "),
        "starts at the edge-0 midpoint: {d}"
    );
}

/// A square with a band from mid-edge 0 (0.5) to node 2.
fn square_with_span() -> Shape {
    let mut shape = square(0.0);
    let mut band = slp_core::Border::new("edging".to_string(), 0.5);
    band.start_node = Some(0.5);
    band.end_node = Some(2.0);
    shape.borders = vec![band];
    shape
}

#[test]
fn a_selected_shape_with_a_span_shows_draggable_seam_handles() {
    let dom = leptotes::dom(move || {
        view! { <Shapes t=t() shapes=vec![square_with_span()] selected=Some(0) /> }
    });

    let dots: Vec<(f64, f64)> = dom
        .find_all(r#"circle[data-testid="shape-seam-handle"]"#)
        .iter()
        .map(|dot| {
            let at = |name| {
                dot.attr(name)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(f64::NAN)
            };
            (at("cx"), at("cy"))
        })
        .collect();
    let near =
        |(x, y): (f64, f64), (ex, ey): (f64, f64)| (x - ex).abs() < 1e-6 && (y - ey).abs() < 1e-6;
    let d = 5.0 / 2.0_f64.sqrt();

    // One handle per span endpoint. A mid-edge start (0.5 along edge 0,
    // boundary screen (20, 200)) insets straight inward (up) by the 0.5 ft
    // band × 10 px/ft = 5 px → (20, 195). A corner end (node 2, screen
    // (40, 170)) insets along the corner's interior bisector, diagonally into
    // the shape by 5/√2 each way, not along one edge's normal (which would
    // land it on the adjacent edge).
    assert!(
        dots.len() == 2 && near(dots[0], (20.0, 195.0)) && near(dots[1], (40.0 - d, 170.0 + d)),
        "seam dots on the band's inner edge: {dots:?}"
    );
}

#[test]
fn an_unselected_shape_shows_no_seam_handles() {
    let dom = leptotes::dom(move || view! { <Shapes t=t() shapes=vec![square_with_span()] /> });

    assert_eq!(
        dom.count(r#"[data-testid="shape-seam-handle"]"#),
        0,
        "{dom}"
    );
}

#[test]
fn a_full_ring_border_has_no_seam_handles() {
    // A full ring has no span endpoints to drag.
    let mut ring = square(0.0);
    ring.borders = vec![slp_core::Border::new("edging".to_string(), 0.5)];
    let dom = leptotes::dom(move || {
        view! { <Shapes t=t() shapes=vec![ring.clone()] selected=Some(0) /> }
    });

    assert_eq!(
        dom.count(r#"[data-testid="shape-seam-handle"]"#),
        0,
        "{dom}"
    );
}

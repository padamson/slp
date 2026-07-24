//! dokime component tests for `Shapes` (drawn paver/mulch/… areas).

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

#[test]
fn renders_a_shape_with_markers_and_area_label() {
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });
    assert!(html.contains(r#"class="shapes""#), "tagged for queries");
    assert!(html.contains("<polygon"), "the area is a polygon");
    assert_eq!(
        dokime::count(&html, r#"class="shape-corner""#),
        4,
        "a marker per corner"
    );
    // 4 ft x 3 ft = 12 ft², elevation 0 -> no elevation suffix.
    assert!(html.contains(">12 ft²<"), "the bare area label, no suffix");
}

#[test]
fn a_nonzero_elevation_appends_to_the_label() {
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(1.5)] /> });
    assert!(html.contains("12 ft² · +1.5 ft"), "area + elevation label");
}

#[test]
fn a_negative_elevation_shows_a_single_minus_sign() {
    // A below-grade area (e.g. a sunken paver patio) reads "-0.5 ft", not the
    // doubled-up "+-0.5 ft" a naive `+{elevation}` format would produce.
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(-0.5)] /> });
    assert!(html.contains("12 ft² · -0.5 ft"), "single minus sign");
    assert!(!html.contains("+-"), "never a doubled sign");
}

#[test]
fn renders_one_polygon_per_shape() {
    let html =
        dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0), square(2.0)] /> });
    assert_eq!(dokime::count(&html, "<polygon"), 2, "one polygon per shape");
}

#[test]
fn no_shapes_renders_nothing() {
    let html = dokime::render(move || view! { <Shapes t=t() shapes=Vec::new() /> });
    assert!(!html.contains(r#"class="shapes""#));
}

#[test]
fn a_bowed_edge_renders_a_path_with_an_arc_command_not_a_polygon() {
    // Bow the first edge (0,0)->(4,0) into an arc — the whole boundary becomes
    // a <path> with an `A` arc command, no <polygon>.
    let mut s = square(0.0);
    s.bulges = vec![0.5, 0.0, 0.0, 0.0];
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![s] /> });
    assert!(!html.contains("<polygon"), "an arced boundary is a path");
    assert!(
        html.contains("<path"),
        "the arced boundary renders as a path"
    );
    // The path has exactly one arc command (the one bowed edge).
    assert_eq!(dokime::count(&html, " A "), 1, "one arc command");
}

#[test]
fn a_bowed_edge_changes_the_reported_area() {
    // Bowing an edge outward (negative bulge = away from a CCW interior) grows
    // the area past the straight 12 ft²; the label reflects it.
    let mut s = square(0.0);
    s.bulges = vec![-1.0, 0.0, 0.0, 0.0]; // bottom edge bows out into a semicircle
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![s] /> });
    assert!(
        !html.contains(">12 ft²<"),
        "the area is no longer the straight 12"
    );
}

#[test]
fn a_mulch_bed_renders_in_the_mulch_color() {
    // A shape whose material resolves (through the catalog) to a "mulch-bed"
    // category fills mulch brown; an uncategorized area fills the default.
    let mut mulch = CatalogItem::new("mulch".to_string());
    mulch.category = Some("mulch-bed".to_string());
    let catalog = vec![mulch];

    let mut bed = square(0.0);
    bed.material_ref = Some("mulch".to_string());
    let cat = catalog.clone();
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![bed] catalog=cat /> });
    assert!(html.contains(MULCH_FILL), "the mulch bed fills mulch brown");

    // An uncategorized area (no material_ref) keeps the neutral default fill.
    let plain = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0)] catalog=catalog /> }
    });
    assert!(
        plain.contains(SHAPE_FILL),
        "an uncategorized area is neutral"
    );
}

#[test]
fn a_paver_area_renders_in_the_paver_color() {
    // A shape whose material resolves to a "paver" category fills paver gray,
    // distinct from mulch's brown.
    let mut paver = CatalogItem::new("paver".to_string());
    paver.category = Some("paver".to_string());

    let mut area = square(0.0);
    area.material_ref = Some("paver".to_string());
    let html =
        dokime::render(move || view! { <Shapes t=t() shapes=vec![area] catalog=vec![paver] /> });
    assert!(html.contains(PAVER_FILL), "the paver area fills paver gray");
    assert!(!html.contains(MULCH_FILL), "not the mulch color");
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

#[test]
fn a_material_with_an_image_tiles_the_surface_as_a_pattern() {
    // A material carrying a photo fills its area with an SVG <pattern> tiled at
    // real-world scale (tile-size-ft × px_ft), and the polygon references it via
    // fill="url(#area-mat-{material id})" — not the flat category color.
    let mut area = square(0.0);
    area.material_ref = Some("paver".to_string());
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![area] catalog=vec![textured_paver()] /> }
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
        html.contains(r#"fill="url(#area-mat-paver)""#),
        "the polygon is filled by its material's pattern: {html}"
    );
    // 2 ft wide × 10 px/ft = 20 px tile; 3 ft deep × 10 = 30 px.
    assert!(
        html.contains(r#"width="20""#),
        "tile width is real-world scaled"
    );
    assert!(
        html.contains(r#"height="30""#),
        "tile depth is real-world scaled"
    );
    // Anchored at the world origin — sx(0)=pad=0, sy(0)=pad+yard_d·px_ft=200 —
    // so the tile grid stays glued to world coordinates as the yard resizes.
    assert!(html.contains(r#"x="0""#), "anchored at world-origin x");
    assert!(html.contains(r#"y="200""#), "anchored at world-origin y");
    assert!(
        !html.contains(PAVER_FILL),
        "the flat paver color is replaced by the texture"
    );
}

#[test]
fn areas_sharing_a_material_share_one_pattern() {
    // Two paver areas → a single <pattern> (per material, not per area), so a
    // photo's data-URI is embedded once no matter how many areas tile it.
    let mut a = square(0.0);
    a.material_ref = Some("paver".to_string());
    let mut b = square(1.0);
    b.material_ref = Some("paver".to_string());
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![a, b] catalog=vec![textured_paver()] /> }
    });
    assert_eq!(dokime::count(&html, "<pattern"), 1, "one shared pattern");
    assert_eq!(
        dokime::count(&html, r#"fill="url(#area-mat-paver)""#),
        2,
        "both areas reference it"
    );
}

#[test]
fn selection_overrides_the_texture() {
    // A selected textured area shows the translucent selection tint, not the
    // photo — the user needs selection feedback (and to see the grid/deck
    // beneath) while editing.
    let mut area = square(0.0);
    area.material_ref = Some("paver".to_string());
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![area] catalog=vec![textured_paver()] selected=Some(0) /> }
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
fn a_curved_edge_renders_a_path_with_a_bezier_command() {
    // Give edge 0 a cubic bezier — the boundary becomes a <path> with a `C`
    // command, no <polygon>, and the reported area shifts off the straight 12.
    let mut s = square(0.0);
    s.curves = vec![CurveEdge {
        edge: 0,
        control1: Box::new(Coord::new(1.0, -2.0)),
        control2: Box::new(Coord::new(3.0, -2.0)),
    }];
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![s] /> });
    assert!(!html.contains("<polygon"), "a curved boundary is a path");
    assert_eq!(dokime::count(&html, " C "), 1, "one cubic-bezier command");
    assert!(
        !html.contains(">12 ft²<"),
        "the area accounts for the curve"
    );
}

#[test]
fn skips_a_degenerate_shape_with_too_few_corners() {
    let degenerate = Shape {
        corners: vec![Coord::new(5.0, 5.0), Coord::new(6.0, 5.0)],
        elevation: 0.0,
        bulges: Vec::new(),
        curves: Vec::new(),
        material_ref: None,
        depth_in: None,
        pattern: None,
        courses: Vec::new(),
        borders: Vec::new(),
    };
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0), degenerate] /> }
    });
    assert_eq!(
        dokime::count(&html, "<polygon"),
        1,
        "a shape with under 3 corners has no area to render"
    );
}

#[test]
fn an_unselected_shape_has_plain_corner_markers() {
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });
    assert_eq!(dokime::count(&html, r#"class="shape-corner""#), 4);
    assert_eq!(dokime::count(&html, r#"data-testid="shape-node""#), 0);
    assert!(!html.contains(r#"class="shape shape--selected""#));
}

#[test]
fn a_selected_shape_shows_interactive_node_handles_instead() {
    let html = dokime::render(
        move || view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> },
    );
    assert!(html.contains(r#"class="shape shape--selected""#));
    assert_eq!(dokime::count(&html, r#"class="shape-corner""#), 0);
    assert_eq!(dokime::count(&html, r#"data-testid="shape-node""#), 4);
}

#[test]
fn a_selected_shape_shows_a_bulge_handle_per_edge() {
    // The square has 4 edges, so 4 edge (bulge) handles when selected — none
    // when unselected.
    let selected = dokime::render(
        move || view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> },
    );
    assert_eq!(
        dokime::count(&selected, r#"data-testid="shape-edge-handle""#),
        4
    );
    let plain = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });
    assert_eq!(
        dokime::count(&plain, r#"data-testid="shape-edge-handle""#),
        0
    );
}

#[test]
fn a_selected_shape_shows_two_control_handles_per_straight_edge() {
    // A straight square: 4 edges × 2 control handles = 8, plus the 4 apex
    // (bulge) handles. Straight edges draw no guide line (the controls sit on
    // the chord).
    let html = dokime::render(
        move || view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> },
    );
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-control-handle""#),
        8
    );
    assert_eq!(dokime::count(&html, r#"class="shape-control-guide""#), 0);
}

#[test]
fn a_bezier_edge_shows_control_handles_with_guides_and_no_apex() {
    // Make edge 0 a bezier. That edge shows 2 control handles + 2 guide lines
    // and NO apex handle; the other 3 straight edges keep their apex + 2
    // controls each. So: apex handles 3, control handles 2 (bezier) + 6
    // (straight) = 8, guide lines 2.
    let mut s = square(0.0);
    s.curves = vec![CurveEdge {
        edge: 0,
        control1: Box::new(Coord::new(1.0, -2.0)),
        control2: Box::new(Coord::new(3.0, -2.0)),
    }];
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![s] selected=Some(0) /> });
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-edge-handle""#),
        3,
        "the bezier edge has no apex handle"
    );
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-control-handle""#),
        8
    );
    assert_eq!(
        dokime::count(&html, r#"class="shape-control-guide""#),
        2,
        "the bezier edge's two controls each draw a guide line"
    );
}

#[test]
fn only_the_selected_shape_gets_node_handles() {
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0), square(2.0)] selected=Some(1) /> }
    });
    // One shape (index 0) stays plain; the other (index 1) gets node handles.
    assert_eq!(dokime::count(&html, r#"class="shape-corner""#), 4);
    assert_eq!(dokime::count(&html, r#"data-testid="shape-node""#), 4);
}

#[test]
fn no_popup_with_fewer_than_two_selected_nodes() {
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) selected_nodes=vec![1] /> }
    });
    assert!(!html.contains("node-insert-popup"));
}

#[test]
fn a_pair_of_selected_nodes_shows_the_insert_cancel_popup() {
    let html = dokime::render(move || {
        view! {
            <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) selected_nodes=vec![0, 1] />
        }
    });
    assert!(html.contains("node-insert-popup"));
    assert!(html.contains(r#"data-testid="insert-node""#));
    assert!(html.contains(r#"data-testid="cancel-node-select""#));
}

#[test]
fn border_rings_render_clipped_ribbons() {
    // A 4×3 area (10 px/ft) with two full rings — outer 0.5 ft, inner
    // 0.25 ft: each renders as its own closed annulus ribbon (two opposite-
    // wound subpaths), filled with the band paint and clipped to the area.
    let mut s = square(0.0);
    s.borders = vec![
        slp_core::Border::new("cobble".to_string(), 0.5),
        slp_core::Border::new("edging".to_string(), 0.25),
    ];
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![s.clone()] /> });
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-border""#),
        2,
        "one ribbon per ring: {html}"
    );
    assert!(html.contains("clipPath"), "the ribbons clip to the area");
    assert_eq!(
        dokime::count(&html, "stroke-width"),
        1,
        "only the body outline strokes; ribbons are filled"
    );
}

#[test]
fn a_borderless_shape_renders_no_ring_strokes() {
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-border""#), 0);
    assert!(!html.contains("clipPath"), "no clip needed without rings");
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
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![shape.clone()] /> });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-border""#), 1);
    let start = html.find(r#"data-testid="shape-border""#).unwrap();
    let frag = &html[start..];
    let d_start = frag.find(" d=\"").unwrap() + 4;
    let path_d = &frag[d_start..d_start + frag[d_start..].find('\"').unwrap()];
    let coords: Vec<f64> = path_d
        .split_whitespace()
        .filter_map(|tok| tok.parse().ok())
        .collect();
    assert!(coords.len() >= 8, "the ribbon has real geometry: {path_d}");
    for pair in coords.chunks(2) {
        let (px, py) = (pair[0], pair[1]);
        assert!(
            px <= 40.0 + 1e-6 && py <= 200.0 + 1e-6,
            "ribbon stays on the interior side: ({px}, {py}) in {path_d}"
        );
    }
}

#[test]
fn a_degenerate_span_border_draws_nothing() {
    let mut s = square(0.0);
    let mut b = slp_core::Border::new("edging".to_string(), 0.5);
    b.start_node = Some(0.0);
    b.end_node = Some(9.0); // out of range
    s.borders = vec![b];
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![s.clone()] /> });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-border""#), 0);
}

#[test]
fn a_selected_shape_labels_its_node_indices() {
    // The border editor's From/To fields refer to node numbers, so a selected
    // shape shows each handle's index; unselected shapes stay unlabeled.
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> }
    });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-node-index""#), 4);
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-node-index""#), 0);
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
    let mut shape = square(0.0);
    let mut south = slp_core::Border::new("edging".to_string(), 0.5);
    south.start_node = Some(0.0);
    south.end_node = Some(1.0);
    let mut east = slp_core::Border::new("edging".to_string(), 0.5);
    east.start_node = Some(1.0);
    east.end_node = Some(2.0);
    shape.borders = vec![south, east];
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![shape.clone()] /> });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-border""#), 2);
    assert!(!html.contains("stroke-linecap"), "plain butt caps");
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-border-joint""#),
        0
    );
    // Ribbons are filled regions, not strokes.
    assert_eq!(
        dokime::count(&html, "stroke-width"),
        1,
        "only the body outline strokes"
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

#[test]
fn spans_meeting_at_a_reflex_corner_get_a_miter_joint_patch() {
    // Bands on edges 2→3 and 3→4 meet at the L's reflex corner: butt caps
    // leave a pie gap there, filled by exactly one junction quad.
    let mut s = ell(0.0);
    let mut a = slp_core::Border::new("edging".to_string(), 0.5);
    a.start_node = Some(2.0);
    a.end_node = Some(3.0);
    let mut b = slp_core::Border::new("edging".to_string(), 0.5);
    b.start_node = Some(3.0);
    b.end_node = Some(4.0);
    s.borders = vec![a, b];
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![s.clone()] /> });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-border""#), 2);
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-border-joint""#),
        1,
        "one miter patch at the reflex node: {html}"
    );
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
    // tangent. With the sagging curve the corner becomes effectively convex
    // at the tangent level — the patch disappears instead of rendering as a
    // rotated stub.
    let mut s = ell(0.0);
    let mut a = slp_core::Border::new("edging".to_string(), 0.25);
    a.start_node = Some(2.0);
    a.end_node = Some(3.0);
    let mut b = slp_core::Border::new("edging".to_string(), 0.25);
    b.start_node = Some(3.0);
    b.end_node = Some(4.0);
    s.borders = vec![a, b];
    // Edge 2 runs (4,1) → (2,1); controls pulled up to y=2 sag it into the
    // interior of the notch strip above.
    s.curves = vec![CurveEdge::new(
        Box::new(Coord::new(3.5, 2.0)),
        Box::new(Coord::new(2.5, 2.0)),
        2,
    )];
    let html = dokime::render({
        let s = s.clone();
        move || view! { <Shapes t=t() shapes=vec![s.clone()] /> }
    });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-border""#), 2);
    // The corner stays reflex, so the patch remains — but its geometry must
    // follow the curve's END tangent (world (2,1)−(2.5,2), i.e. the band's
    // inner corner at v + inward(tangent)·depth ≈ (22.236, 191.118)), not the
    // horizontal chord (which would put it at (20, 192.5) — the misplaced
    // stub seen in manual testing).
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-border-joint""#),
        1
    );
    assert!(
        html.contains("22.23606797749979,191.1180339887499"),
        "tangent-based inner corner: {html}"
    );
    assert!(
        !html.contains("points=\"20,190 20,192.5"),
        "no chord-based stub"
    );
}

#[test]
fn an_arc_edge_band_hugs_the_real_arc_not_its_mirror() {
    // Edge 0 of the square bulges into the interior as a semicircle (bulge 1,
    // apex world (2,2) → screen (20,180)). The band's ribbon must trace that
    // arc — every coordinate stays at or above the square's base (screen
    // y ≤ 200); the mirrored sweep seen in manual testing put the arc at
    // world (2,−2) → screen y ≈ 220, sashing outside/below.
    let mut shape = square(0.0);
    shape.bulges = vec![1.0];
    let mut band = slp_core::Border::new("edging".to_string(), 0.5);
    band.start_node = Some(0.0);
    band.end_node = Some(1.0);
    shape.borders = vec![band];
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![shape.clone()] /> });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-border""#), 1);
    let start = html.find(r#"data-testid="shape-border""#).unwrap();
    let frag = &html[start..];
    let d_start = frag.find(" d=\"").unwrap() + 4;
    let path_d = &frag[d_start..d_start + frag[d_start..].find('\"').unwrap()];
    let coords: Vec<f64> = path_d
        .split_whitespace()
        .filter_map(|tok| tok.parse().ok())
        .collect();
    let mut min_y = f64::MAX;
    for pair in coords.chunks(2) {
        assert!(
            pair[1] <= 200.5,
            "the ribbon never dips below the base: ({}, {}) in {path_d}",
            pair[0],
            pair[1]
        );
        min_y = min_y.min(pair[1]);
    }
    assert!(
        min_y < 182.0,
        "the ribbon reaches the true apex (screen y ≈ 180): min y {min_y}"
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
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![shape.clone()] /> });
    assert_eq!(dokime::count(&html, r#"data-testid="shape-border""#), 1);
    // The ribbon's outer polyline begins at the mid-edge point (20, 200).
    let start = html.find(r#"data-testid="shape-border""#).unwrap();
    let frag = &html[start..];
    let d0 = frag.find(" d=\"M ").unwrap() + 6;
    let m = &frag[d0..d0 + 20];
    assert!(m.starts_with("20 200"), "starts at edge-0 midpoint: {m}");
}

#[test]
fn a_selected_shape_with_a_span_shows_draggable_seam_handles() {
    // A span from mid-edge 0 (0.5) to node 2 on a selected square shows two seam
    // handles (its start and end); an unselected shape shows none, and a
    // full-ring border (no span) shows none.
    let mut shape = square(0.0);
    let mut band = slp_core::Border::new("edging".to_string(), 0.5);
    band.start_node = Some(0.5);
    band.end_node = Some(2.0);
    shape.borders = vec![band];
    let html = dokime::render({
        let shape = shape.clone();
        move || view! { <Shapes t=t() shapes=vec![shape.clone()] selected=Some(0) /> }
    });
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-seam-handle""#),
        2,
        "one handle per span endpoint: {html}"
    );
    // Parse every seam dot's (cx, cy) center.
    let centers = |html: &str| -> Vec<(f64, f64)> {
        let attr = |s: &str, k: &str| -> f64 {
            let i = s.find(k).unwrap() + k.len();
            s[i..]
                .trim_start_matches(['"', '\''])
                .split(['"', '\''])
                .next()
                .unwrap()
                .parse()
                .unwrap()
        };
        html.match_indices(r#"data-testid="shape-seam-handle""#)
            .map(|(i, _)| {
                let s = &html[i..];
                (attr(s, " cx="), attr(s, " cy="))
            })
            .collect()
    };
    let dots = centers(&html);
    // A **mid-edge** start (0.5 along edge 0, boundary screen (20, 200)) insets
    // straight inward (up) by the 0.5 ft band × 10 px/ft = 5 px → (20, 195).
    assert!(
        (dots[0].0 - 20.0).abs() < 1e-6 && (dots[0].1 - 195.0).abs() < 1e-6,
        "mid-edge start dot on the band's inner edge: {:?}",
        dots[0]
    );
    // A **corner** end (node 2, screen (40, 170)) insets along the corner's
    // interior bisector — diagonally into the shape (−x, +y here), 5/√2 each —
    // not along one edge's normal (which would land it on the adjacent edge).
    let d = 5.0 / 2.0_f64.sqrt();
    assert!(
        (dots[1].0 - (40.0 - d)).abs() < 1e-6 && (dots[1].1 - (170.0 + d)).abs() < 1e-6,
        "corner end dot on the bisector, diagonally inside: {:?}",
        dots[1]
    );
    // Unselected → no seam handles.
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![shape.clone()] /> });
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-seam-handle""#),
        0
    );
    // A full-ring border has no span endpoints to drag.
    let mut ring = square(0.0);
    ring.borders = vec![slp_core::Border::new("edging".to_string(), 0.5)];
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![ring.clone()] selected=Some(0) /> }
    });
    assert_eq!(
        dokime::count(&html, r#"data-testid="shape-seam-handle""#),
        0
    );
}

//! Drawn areas (paver patios, mulch beds, …): closed outlines at a given
//! elevation whose edges are straight, circular **arcs** (per-edge bulge), or
//! cubic-Bézier **curves** (per-edge control points), rendered filled with
//! corner markers and an area + elevation label — the area equivalent of
//! `Furnishings`. This draws the *committed* shapes from the `Plan`; the
//! in-progress outline being drawn is the `Placement` overlay. Category-
//! specific look (paver vs. mulch) lands with whichever story first needs it.
//!
//! An all-straight boundary renders as a plain `<polygon>`; once any edge bows
//! or curves it renders as a `<path>` (an `A` arc command per bowed edge, a
//! `C` cubic command per curved edge, `L` lines for the rest). The reported
//! area accounts for every arc/curve via `slp_core::boundary_area`.
//!
//! A selected shape's corners render as larger, interactive **node handles**
//! (the same "press to start a drag" gesture a tree's canopy/trunk handles
//! use) instead of the plain markers: press one to select it and start a
//! move drag, press an adjacent second one to arm the **insert-between**
//! popup (Insert/Cancel, floating near their midpoint). Each edge also gets a
//! per-edge **apex handle** (drag to bow it into an arc) and two **Bézier
//! control handles** (drag to curve it) — a curved edge drops the apex handle.
//! All selection/drag state is owned by the caller — this component only
//! renders it and reports presses.

use leptos::prelude::*;
use slp_core::{Border, CatalogItem, Coord, CurveEdge, Point, Shape, arc_svg, boundary_area};

use super::Transform;
use crate::style::{
    SELECTED_FILL, SELECTED_STROKE, SHAPE_FILL, SHAPE_FILL_OPACITY, SHAPE_STROKE, area_style,
};

/// The catalog item a drawn area's `material_ref` names, if any — the one
/// id-match lookup every material-derived attribute (category, texture)
/// resolves through. Shared with `circles`.
pub(crate) fn find_material<'a>(
    catalog: &'a [CatalogItem],
    material_ref: Option<&str>,
) -> Option<&'a CatalogItem> {
    let id = material_ref?;
    catalog.iter().find(|c| c.id == id)
}

/// Whether a catalog material carries a photo to tile across drawn surfaces.
pub(crate) fn has_texture(item: &CatalogItem) -> bool {
    item.image.as_deref().is_some_and(|s| !s.is_empty())
}

/// One SVG `<defs>` block holding one `<pattern>` per distinct textured
/// material among `refs` — each `id="{prefix}-{material id}"`, tiling the
/// material's photo at real-world scale (tile-size-ft × `px_ft`). Emitting one
/// pattern per *material* (not per drawn area) embeds a photo's data-URI once
/// no matter how many areas share it, and every area filling
/// `url(#{prefix}-{id})` gets the identical, aligned tile grid.
///
/// The pattern is anchored at the yard's world origin (`x`/`y` below), not the
/// SVG viewport origin — so the tile grid stays glued to world coordinates
/// (tiles don't "slide" under the drawn areas when the yard depth changes).
/// `None` when nothing is textured.
pub(crate) fn texture_patterns(
    prefix: &'static str,
    catalog: &[CatalogItem],
    refs: &[Option<String>],
    t: Transform,
) -> Option<impl IntoView + use<>> {
    let mut items: Vec<&CatalogItem> = Vec::new();
    for r in refs {
        if let Some(item) = find_material(catalog, r.as_deref())
            && has_texture(item)
            && !items.iter().any(|i| i.id == item.id)
        {
            items.push(item);
        }
    }
    let patterns = items
        .into_iter()
        .map(|item| {
            let (tile_w, tile_d) = slp_core::tile_size_ft(item);
            let (pw, ph) = (tile_w * t.px_ft, tile_d * t.px_ft);
            view! {
                <pattern
                    id=format!("{prefix}-{}", item.id)
                    patternUnits="userSpaceOnUse"
                    x=t.sx(0.0)
                    y=t.sy(0.0)
                    width=pw
                    height=ph
                >
                    <image
                        href=item.image.clone().unwrap_or_default()
                        x="0"
                        y="0"
                        width=pw
                        height=ph
                        preserveAspectRatio="none"
                    />
                </pattern>
            }
        })
        .collect::<Vec<_>>();
    (!patterns.is_empty()).then(|| view! { <defs>{patterns}</defs> })
}

/// One border band's resolved render inputs: its stroke paint (the material's
/// texture pattern when it has a photo, else the flat category color), the
/// matching stroke opacity, its laid width, and — when both span nodes are
/// set — the open node span it covers. Shared by `Shapes` and `Circles`
/// (a circle has no nodes and ignores `span`).
pub(crate) struct BorderPaint {
    pub paint: String,
    pub opacity: &'static str,
    pub width_ft: f64,
    pub span: Option<(usize, usize)>,
}

/// Resolve each border band's paint through the catalog.
pub(crate) fn border_paints(
    prefix: &'static str,
    catalog: &[CatalogItem],
    borders: &[Border],
) -> Vec<BorderPaint> {
    borders
        .iter()
        .map(|b| {
            let item = find_material(catalog, Some(&b.material_ref));
            let (paint, opacity) = match item {
                Some(c) if has_texture(c) => (format!("url(#{prefix}-{})", c.id), "1"),
                _ => {
                    let category = item.and_then(|c| c.category.as_deref());
                    (area_style(category).0.to_string(), SHAPE_FILL_OPACITY)
                }
            };
            let span = match (b.start_node, b.end_node) {
                (Some(from), Some(to)) => {
                    match (usize::try_from(from), usize::try_from(to)) {
                        (Ok(from), Ok(to)) => Some((from, to)),
                        // Unrepresentable indices: a dead span (drawn as nothing),
                        // matching the take-off's under-count-loudly choice.
                        _ => Some((usize::MAX, usize::MAX)),
                    }
                }
                _ => None,
            };
            BorderPaint {
                paint,
                opacity,
                width_ft: b.width_ft,
                span,
            }
        })
        .collect()
}

/// One planned border stroke: which border (by list index) it paints, the
/// run of consecutive edges it covers (`None` = the whole closed boundary),
/// and its inner depth from the boundary edge.
pub(crate) struct BorderStroke {
    pub border: usize,
    /// `Some((start, end))` = an open run from node `start` forward to node
    /// `end`; `None` = the full closed ring.
    pub run: Option<(usize, usize)>,
    /// The stroke's inner depth (ft): this band's outer offset — the summed
    /// widths of earlier bands that cover the same edges — plus its own width.
    pub depth_ft: f64,
}

/// Plan a boundary's border strokes with **per-edge offset stacking**: a band
/// nests inward only under the earlier bands that actually cover each of its
/// edges (bands on other edges don't push it inward), and where that offset
/// changes along the band it splits into runs. Painting the result
/// deepest-first lets each outer band overpaint the inner excess exactly
/// where it covers — which a single cumulative width cannot do once spans
/// cover different edges. Input: each border's `(width_ft, span)`, with
/// `span = None` for a full ring; a dead span (equal or out-of-range nodes)
/// covers nothing and yields no stroke.
pub(crate) fn border_strokes(
    borders: &[(f64, Option<(usize, usize)>)],
    n: usize,
) -> Vec<BorderStroke> {
    if n == 0 {
        return Vec::new();
    }
    // Which edges each band covers.
    let coverage: Vec<Vec<bool>> = borders
        .iter()
        .map(|(_, span)| {
            let mut cov = vec![false; n];
            match span {
                None => cov.iter_mut().for_each(|c| *c = true),
                Some((from, to)) => {
                    if *from < n && *to < n && from != to {
                        let mut i = *from;
                        while i != *to {
                            cov[i] = true;
                            i = (i + 1) % n;
                        }
                    }
                }
            }
            cov
        })
        .collect();
    let mut strokes = Vec::new();
    for (j, (w, span)) in borders.iter().enumerate() {
        if !coverage[j].iter().any(|&c| c) {
            continue;
        }
        // This band's outer offset at edge `e`.
        let offset = |e: usize| -> f64 {
            borders[..j]
                .iter()
                .zip(&coverage[..j])
                .filter(|(_, cov)| cov[e])
                .map(|((wi, _), _)| *wi)
                .sum()
        };
        // The band's edges in walk order, with each edge's offset.
        let mut edges: Vec<usize> = match span {
            None => (0..n).collect(),
            Some((from, to)) => {
                let mut v = Vec::new();
                let mut i = *from;
                while i != *to {
                    v.push(i);
                    i = (i + 1) % n;
                }
                v
            }
        };
        let mut offs: Vec<f64> = edges.iter().map(|&e| offset(e)).collect();
        let uniform = offs.windows(2).all(|p| (p[0] - p[1]).abs() < 1e-9);
        if span.is_none() && uniform {
            strokes.push(BorderStroke {
                border: j,
                run: None,
                depth_ft: offs[0] + w,
            });
            continue;
        }
        // A closed band with varying offsets: rotate the walk so it starts at
        // an offset change, putting the wrap seam on a real split.
        if span.is_none()
            && let Some(k) = (0..edges.len()).find(|&k| {
                let prev = (k + edges.len() - 1) % edges.len();
                (offs[k] - offs[prev]).abs() > 1e-9
            })
        {
            edges.rotate_left(k);
            offs.rotate_left(k);
        }
        // Contiguous runs of equal offset.
        let mut run_start = 0usize;
        for k in 1..=edges.len() {
            if k == edges.len() || (offs[k] - offs[run_start]).abs() > 1e-9 {
                strokes.push(BorderStroke {
                    border: j,
                    run: Some((edges[run_start], (edges[k - 1] + 1) % n)),
                    depth_ft: offs[run_start] + w,
                });
                run_start = k;
            }
        }
    }
    // Deepest first, so outer bands overpaint the inner excess they cover.
    strokes.sort_by(|a, b| b.depth_ft.total_cmp(&a.depth_ft));
    strokes
}

/// Per-edge band summary for junction filling: each edge's **total** band
/// depth (the stacked widths of every band covering it — the depth the
/// painted bands reach) and its innermost covering band (the last in list
/// order, whose paint faces the field). `(0.0, None)` for an uncovered edge.
pub(crate) fn edge_band_depths(
    borders: &[(f64, Option<(usize, usize)>)],
    n: usize,
) -> Vec<(f64, Option<usize>)> {
    let mut out = vec![(0.0, None); n];
    for (j, (w, span)) in borders.iter().enumerate() {
        let mut cover = |e: usize| {
            out[e].0 += w;
            out[e].1 = Some(j);
        };
        match span {
            None => (0..n).for_each(&mut cover),
            Some((from, to)) => {
                if *from < n && *to < n && from != to {
                    let mut i = *from;
                    while i != *to {
                        cover(i);
                        i = (i + 1) % n;
                    }
                }
            }
        }
    }
    out
}

/// Boundary edge `i`'s unit tangent directions **at its two end nodes**, in
/// screen px: the chord for a straight edge, the chord rotated by ∓θ/2 for an
/// arc (a circle's chord meets its tangent at half the subtended angle), and
/// the control-point directions for a Bézier. Junction patches need the true
/// tangents — a curved edge's chord can point somewhere else entirely.
/// `None` for a degenerate (zero-length) edge.
pub(crate) fn edge_end_tangents(
    corners: &[Coord],
    bulges: &[f64],
    curves: &[CurveEdge],
    i: usize,
) -> Option<((f64, f64), (f64, f64))> {
    let n = corners.len();
    let (a, b) = (&corners[i], &corners[(i + 1) % n]);
    let norm = |dx: f64, dy: f64| -> Option<(f64, f64)> {
        let len = dx.hypot(dy);
        (len > 1e-9).then(|| (dx / len, dy / len))
    };
    // World-space (y up) directions; the caller converts to screen.
    let world = if let Some(c) = curves.iter().find(|c| usize::try_from(c.edge) == Ok(i)) {
        let start = norm(c.control1.x - a.x, c.control1.y - a.y)
            .or_else(|| norm(c.control2.x - a.x, c.control2.y - a.y))
            .or_else(|| norm(b.x - a.x, b.y - a.y))?;
        let end = norm(b.x - c.control2.x, b.y - c.control2.y)
            .or_else(|| norm(b.x - c.control1.x, b.y - c.control1.y))
            .or_else(|| norm(b.x - a.x, b.y - a.y))?;
        (start, end)
    } else {
        let chord = norm(b.x - a.x, b.y - a.y)?;
        let bulge = bulges.get(i).copied().unwrap_or(0.0);
        if bulge.abs() < 1e-9 {
            (chord, chord)
        } else {
            // θ/2 = 2·atan(b), signed: positive bows left of travel (world
            // CCW), so the start tangent turns +θ/2 and the end −θ/2.
            let half = 2.0 * bulge.atan();
            let rot = |d: (f64, f64), phi: f64| {
                (
                    d.0 * phi.cos() - d.1 * phi.sin(),
                    d.0 * phi.sin() + d.1 * phi.cos(),
                )
            };
            (rot(chord, half), rot(chord, -half))
        }
    };
    // World → screen: the y axis flips.
    Some(((world.0.0, -world.0.1), (world.1.0, -world.1.1)))
}

/// The miter-fill quad for a band junction at boundary point `v` (screen px):
/// the pie wedge that butt-ended bands leave unpainted at a **reflex** corner
/// (the boundary turns away from the bands), bounded by the two bands' butt
/// faces and their inner edges extended to the miter point. `None` at a
/// convex or collinear corner, where butt-ended bands already meet.
///
/// `dir_*` are the unit edge directions walking through `v` (prev ends at it,
/// cur leaves it); `inward_*` are the matching unit normals into the polygon;
/// `depth_*` are the bands' painted depths (px).
pub(crate) fn junction_quad(
    v: (f64, f64),
    dir_prev: (f64, f64),
    dir_cur: (f64, f64),
    inward_prev: (f64, f64),
    inward_cur: (f64, f64),
    depth_prev: f64,
    depth_cur: f64,
) -> Option<[(f64, f64); 4]> {
    // Convex/collinear: the previous band's inner corner already sits on or
    // past the next band's butt face — no gap to fill.
    if inward_prev.0 * dir_cur.0 + inward_prev.1 * dir_cur.1 >= -1e-9 {
        return None;
    }
    // The bands' inner corners at the node.
    let p1 = (
        v.0 + inward_prev.0 * depth_prev,
        v.1 + inward_prev.1 * depth_prev,
    );
    let p2 = (
        v.0 + inward_cur.0 * depth_cur,
        v.1 + inward_cur.1 * depth_cur,
    );
    // Miter point: intersect the two inner edges (p1 along dir_prev, p2 along
    // dir_cur). The reflex check above guarantees they are not parallel, but
    // guard the division anyway.
    let det = dir_prev.0 * (-dir_cur.1) - (-dir_cur.0) * dir_prev.1;
    if det.abs() < 1e-9 {
        return None;
    }
    let (bx, by) = (p2.0 - p1.0, p2.1 - p1.1);
    let s = (bx * (-dir_cur.1) - (-dir_cur.0) * by) / det;
    let miter = (p1.0 + dir_prev.0 * s, p1.1 + dir_prev.1 * s);
    Some([v, p1, miter, p2])
}

/// Sample boundary edge `i` as world-space points from its start node up to
/// (not including) its end node: 1 point for a straight edge, `CURVE_SAMPLES`
/// for an arc or Bézier. Consecutive edges chain into a polyline; the caller
/// appends the final node.
const CURVE_SAMPLES: usize = 24;
#[allow(clippy::many_single_char_names)]
fn sample_edge(corners: &[Coord], bulges: &[f64], curves: &[CurveEdge], edge: usize) -> Vec<Point> {
    let n = corners.len();
    let (from, to) = (&corners[edge], &corners[(edge + 1) % n]);
    if let Some(c) = curves.iter().find(|c| usize::try_from(c.edge) == Ok(edge)) {
        // Cubic Bézier, sampled uniformly in parameter.
        return (0..CURVE_SAMPLES)
            .map(|k| {
                let t = f64::from(u32::try_from(k).unwrap_or(0)) / CURVE_SAMPLES as f64;
                let u = 1.0 - t;
                let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                Point::new(
                    b0 * from.x + b1 * c.control1.x + b2 * c.control2.x + b3 * to.x,
                    b0 * from.y + b1 * c.control1.y + b2 * c.control2.y + b3 * to.y,
                )
            })
            .collect();
    }
    let bulge = bulges.get(edge).copied().unwrap_or(0.0);
    if bulge.abs() < 1e-9 {
        return vec![Point::new(from.x, from.y)];
    }
    // Circular arc: rotate around its center from the start angle by the
    // signed subtended angle θ = 4·atan(b). The center sits on the chord's
    // left normal at `b·c/2 − sign(b)·r` from the midpoint (apex minus radius).
    let mid = Point::new(f64::midpoint(from.x, to.x), f64::midpoint(from.y, to.y));
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let chord = dx.hypot(dy);
    let radius = chord * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
    let normal = (-dy / chord, dx / chord); // left of travel, world
    let off = bulge * chord / 2.0 - bulge.signum() * radius;
    let center = Point::new(mid.x + normal.0 * off, mid.y + normal.1 * off);
    let theta = 4.0 * bulge.atan();
    let start = (from.y - center.y).atan2(from.x - center.x);
    (0..CURVE_SAMPLES)
        .map(|k| {
            let t = f64::from(u32::try_from(k).unwrap_or(0)) / CURVE_SAMPLES as f64;
            // A positive (bows-left) bulge sweeps **clockwise** around its
            // center in world coords — the angle decreases by θ from start
            // to end (sweeping +θ traces the arc's mirror image).
            let ang = start - theta * t;
            Point::new(center.x + radius * ang.cos(), center.y + radius * ang.sin())
        })
        .collect()
}

/// Sample a run of boundary edges (`start` forward to `end`, wrapping; the
/// whole boundary when `closed`) as screen-space points with unit **inward**
/// normals. Tangents are taken numerically from neighboring samples;
/// `inward` maps a travel tangent to the polygon's interior side.
fn sample_run(
    t: Transform,
    corners: &[Coord],
    bulges: &[f64],
    curves: &[CurveEdge],
    run: Option<(usize, usize)>,
    inward: impl Fn((f64, f64)) -> (f64, f64),
) -> Vec<BoundarySample> {
    let n = corners.len();
    let mut world: Vec<Point> = Vec::new();
    match run {
        None => {
            for i in 0..n {
                world.extend(sample_edge(corners, bulges, curves, i));
            }
            // Close the loop for tangent continuity by repeating the start.
            if let Some(first) = world.first().copied() {
                world.push(first);
            }
        }
        Some((start, end)) => {
            let mut i = start;
            while i != end {
                world.extend(sample_edge(corners, bulges, curves, i));
                i = (i + 1) % n;
            }
            let e = &corners[end];
            world.push(Point::new(e.x, e.y));
        }
    }
    let pts: Vec<(f64, f64)> = world.iter().map(|p| (t.sx(p.x), t.sy(p.y))).collect();
    let m = pts.len();
    (0..m)
        .map(|k| {
            // Central-difference tangent (one-sided at the ends).
            let prev = pts[k.saturating_sub(1)];
            let next = pts[(k + 1).min(m - 1)];
            let (dx, dy) = (next.0 - prev.0, next.1 - prev.1);
            let len = dx.hypot(dy).max(1e-9);
            (pts[k], inward((dx / len, dy / len)))
        })
        .collect()
}

/// One boundary sample: its screen point and the unit inward normal there.
type BoundarySample = ((f64, f64), (f64, f64));

/// The SVG path for a band **ribbon**: the filled region between the sampled
/// boundary run offset inward by `o_out` px (its outer edge) and by `o_in` px
/// (its inner edge). Built as real geometry — one-sided by construction — so
/// a band never bleeds to the far side of its edge, which a centered stroke
/// does wherever "just outside this edge" is still inside the polygon (a
/// concave pocket's lip). A closed run becomes two opposite-wound subpaths
/// (an annulus under the nonzero fill rule).
fn ribbon_path(samples: &[BoundarySample], o_out: f64, o_in: f64, closed: bool) -> Option<String> {
    use std::fmt::Write as _;
    if samples.len() < 2 {
        return None;
    }
    let at = |k: usize, o: f64| {
        let ((x, y), (nx, ny)) = samples[k];
        (x + nx * o, y + ny * o)
    };
    let mut d = String::new();
    let start = at(0, o_out);
    let _ = write!(d, "M {} {}", start.0, start.1);
    for k in 1..samples.len() {
        let p = at(k, o_out);
        let _ = write!(d, " L {} {}", p.0, p.1);
    }
    if closed {
        let _ = write!(d, " Z");
        let inner = at(samples.len() - 1, o_in);
        let _ = write!(d, " M {} {}", inner.0, inner.1);
    }
    for k in (0..samples.len()).rev() {
        let p = at(k, o_in);
        let _ = write!(d, " L {} {}", p.0, p.1);
    }
    let _ = write!(d, " Z");
    Some(d)
}

/// A drawn area's fill + fill-opacity: the selection tint while selected,
/// else the material's photo pattern (`url(#{prefix}-{id})`, opaque — a real
/// surface material occludes the grid/deck beneath it by design; selecting
/// the area reverts to the translucent tint so what's underneath shows
/// through while editing), else the flat category color (translucent over
/// the grid). One place encodes the precedence for `Shapes` and `Circles`.
pub(crate) fn surface_fill(
    prefix: &'static str,
    is_selected: bool,
    texture_id: Option<&str>,
    cat_fill: &'static str,
) -> (String, &'static str) {
    if is_selected {
        (SELECTED_FILL.to_string(), SHAPE_FILL_OPACITY)
    } else if let Some(id) = texture_id {
        (format!("url(#{prefix}-{id})"), "1")
    } else {
        (cat_fill.to_string(), SHAPE_FILL_OPACITY)
    }
}

/// The pattern-id prefix for `Shapes` areas (distinct from `Circles`' so the
/// two components' pattern definitions never share a document-global SVG id).
pub(crate) const SHAPE_PATTERN_PREFIX: &str = "area-mat";

/// A selected shape's node-handle radius (px) — bigger than the plain corner
/// marker so it reads as a drag target.
const NODE_HANDLE_R: f64 = 5.0;
/// A selected shape's edge (bulge) handle radius (px) — slightly smaller than
/// a node handle so the two read as different affordances.
const EDGE_HANDLE_R: f64 = 4.0;
/// A selected shape's Bézier control-point handle radius (px).
const CONTROL_HANDLE_R: f64 = 4.0;

/// Whether edge `ei` is a Bézier (has an entry in `curves`).
fn is_bezier(curves: &[CurveEdge], ei: usize) -> bool {
    curves.iter().any(|c| usize::try_from(c.edge) == Ok(ei))
}

/// The two control points (world coords) to show handles for on edge `ei`:
/// the curve's own controls when it's a Bézier, or the chord's 1/3 and 2/3
/// points when it's a plain straight edge (so dragging one promotes it to a
/// Bézier). `None` for an arc edge (which uses its apex handle instead).
fn edge_controls(
    corners: &[Coord],
    bulges: &[f64],
    curves: &[CurveEdge],
    ei: usize,
) -> Option<(Coord, Coord)> {
    let n = corners.len();
    if let Some(c) = curves.iter().find(|c| usize::try_from(c.edge) == Ok(ei)) {
        return Some(((*c.control1).clone(), (*c.control2).clone()));
    }
    // Arc edges show no control handles.
    if bulges.get(ei).copied().unwrap_or(0.0).abs() > 1e-9 {
        return None;
    }
    let from = &corners[ei];
    let to = &corners[(ei + 1) % n];
    let third = |t: f64| Coord::new(from.x + t * (to.x - from.x), from.y + t * (to.y - from.y));
    Some((third(1.0 / 3.0), third(2.0 / 3.0)))
}

// `selected_nodes` is only ever cloned (once, for the selected shape), never
// moved-from — but it's a plain owned prop like every other `Vec` prop here,
// not worth a `&[usize]` + lifetime just to satisfy the lint.
#[allow(clippy::needless_pass_by_value)]
#[component]
pub fn Shapes(
    t: Transform,
    shapes: Vec<Shape>,
    /// The plan catalog, used to resolve each area's `material_ref` to its
    /// material category (mulch, paver, …) for the fill color.
    #[prop(optional)]
    catalog: Vec<CatalogItem>,
    /// The index (into `shapes`) of the currently selected shape, if any — its
    /// corners render as interactive node handles instead of plain markers.
    #[prop(default = None)]
    selected: Option<usize>,
    /// Indices (into the selected shape's corners) of the currently selected
    /// nodes: empty (none picked), one (a move/insert-pair start), or an
    /// adjacent pair (showing the insert-between popup).
    #[prop(optional)]
    selected_nodes: Vec<usize>,
    /// A shape's filled body was pressed (by `shapes` index) — select it.
    #[prop(default = None)]
    on_shape_press: Option<Callback<usize>>,
    /// A selected shape's node handle was pressed (by corner index) — select
    /// it and start a move drag.
    #[prop(default = None)]
    on_node_press: Option<Callback<usize>>,
    /// The insert-between popup's "Insert" button was pressed.
    #[prop(default = None)]
    on_insert_node: Option<Callback<()>>,
    /// The insert-between popup's "Cancel" button was pressed.
    #[prop(default = None)]
    on_cancel_nodes: Option<Callback<()>>,
    /// A selected shape's edge (bulge) handle was pressed (by edge index) —
    /// start a drag that bows that edge into an arc.
    #[prop(default = None)]
    on_edge_press: Option<Callback<usize>>,
    /// A selected shape's Bézier control handle was pressed, as
    /// `(edge_index, which)` where `which` is 0 (control1) or 1 (control2) —
    /// start a drag that curves that edge.
    #[prop(default = None)]
    on_control_press: Option<Callback<(usize, usize)>>,
) -> impl IntoView {
    // One pattern per textured material, shared by every area that uses it —
    // border-ring materials included, so a textured border tiles too.
    let refs: Vec<Option<String>> = shapes
        .iter()
        .flat_map(|s| {
            std::iter::once(s.material_ref.clone())
                .chain(s.borders.iter().map(|b| Some(b.material_ref.clone())))
        })
        .collect();
    let defs = texture_patterns(SHAPE_PATTERN_PREFIX, &catalog, &refs, t);
    let areas = shapes
        .into_iter()
        .enumerate()
        .filter(|(_, s)| s.corners.len() >= 3)
        .map(|(i, s)| {
            let is_selected = selected == Some(i);
            let nodes = if is_selected {
                selected_nodes.clone()
            } else {
                Vec::new()
            };
            // One catalog lookup per area: category + whether its material
            // carries a photo (whose pattern the fill references by id).
            let item = find_material(&catalog, s.material_ref.as_deref());
            let category = item.and_then(|c| c.category.clone());
            let texture_id = item.filter(|c| has_texture(c)).map(|c| c.id.clone());
            let border_paints = border_paints(SHAPE_PATTERN_PREFIX, &catalog, &s.borders);
            shape_view(
                t,
                s,
                i,
                is_selected,
                category,
                texture_id,
                border_paints,
                nodes,
                on_shape_press,
                on_node_press,
                on_insert_node,
                on_cancel_nodes,
                on_edge_press,
                on_control_press,
            )
        })
        .collect::<Vec<_>>();
    (!areas.is_empty()).then(|| {
        view! { <g class="shapes">{defs}{areas}</g> }
    })
}

/// One drawn area: its filled polygon, corner markers (or, when selected,
/// interactive node handles plus an insert-between popup), and an area (ft²)
/// + elevation label at its centroid.
///
/// `shape`/`selected_nodes` are by-value prop-like passthroughs (matching
/// `Furnishings`'s `object_view`): Edition 2024's RPIT lifetime-capture rules
/// mean a borrow here would tie the returned `impl IntoView` to that borrow,
/// which the caller (a short-lived local in the iterator closure above) can't
/// satisfy.
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::many_single_char_names,
    clippy::needless_pass_by_value
)]
fn shape_view(
    t: Transform,
    shape: Shape,
    i: usize,
    is_selected: bool,
    category: Option<String>,
    texture_id: Option<String>,
    border_paints: Vec<BorderPaint>,
    selected_nodes: Vec<usize>,
    on_shape_press: Option<Callback<usize>>,
    on_node_press: Option<Callback<usize>>,
    on_insert_node: Option<Callback<()>>,
    on_cancel_nodes: Option<Callback<()>>,
    on_edge_press: Option<Callback<usize>>,
    on_control_press: Option<Callback<(usize, usize)>>,
) -> impl IntoView {
    let Shape {
        corners,
        elevation,
        bulges,
        curves,
        ..
    } = shape;
    // Straight edges render as line segments; any arc or curve makes the whole
    // boundary a path (arc `A` / bezier `C` commands for the curved edges).
    let has_curve = bulges.iter().any(|b| b.abs() > 1e-9) || !curves.is_empty();
    let points = corners
        .iter()
        .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
        .collect::<Vec<_>>()
        .join(" ");
    let markers = corners
        .iter()
        .enumerate()
        .map(|(ni, c)| {
            let (cx, cy) = (t.sx(c.x), t.sy(c.y));
            if is_selected {
                let node_selected = selected_nodes.contains(&ni);
                let fill = if node_selected {
                    SELECTED_STROKE
                } else {
                    SHAPE_STROKE
                };
                view! {
                    <circle
                        class="shape-node"
                        data-testid="shape-node"
                        cx=cx
                        cy=cy
                        r=NODE_HANDLE_R
                        fill=fill
                        on:mousedown=move |ev: leptos::ev::MouseEvent| {
                            ev.stop_propagation();
                            if let Some(cb) = on_node_press {
                                cb.run(ni);
                            }
                        }
                    />
                    // The node's index, so the border editor's From/To node
                    // fields have something visible to refer to.
                    <text
                        class="shape-node-index"
                        data-testid="shape-node-index"
                        x=cx + 7.0
                        y=cy - 6.0
                        font-size="9"
                        fill=SHAPE_STROKE
                    >
                        {ni}
                    </text>
                }
                .into_any()
            } else {
                view! { <circle class="shape-corner" cx=cx cy=cy r="3" fill=SHAPE_STROKE /> }
                    .into_any()
            }
        })
        .collect::<Vec<_>>();
    // A selected shape gets a bulge handle at each *non-Bézier* edge's apex
    // (its current arc peak, or the chord midpoint when straight): dragging it
    // bows the edge into an arc. A Bézier edge uses its control handles below
    // instead, so it gets no apex handle.
    let edge_count = corners.len();
    let edge_handles = if is_selected {
        (0..edge_count)
            .filter(|&ei| !is_bezier(&curves, ei))
            .filter_map(|ei| {
                let (ax, ay) = edge_apex(&corners, &bulges, ei)?;
                let (hx, hy) = (t.sx(ax), t.sy(ay));
                Some(view! {
                    <circle
                        class="shape-edge-handle"
                        data-testid="shape-edge-handle"
                        cx=hx
                        cy=hy
                        r=EDGE_HANDLE_R
                        fill=SHAPE_FILL
                        stroke=SELECTED_STROKE
                        on:mousedown=move |ev: leptos::ev::MouseEvent| {
                            ev.stop_propagation();
                            if let Some(cb) = on_edge_press {
                                cb.run(ei);
                            }
                        }
                    />
                })
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    // Each straight or Bézier edge also gets two control-point handles (with a
    // guide line from each anchor node to its control). Dragging one on a
    // straight edge promotes it to a Bézier; on a Bézier edge it reshapes the
    // curve. Arc edges show none (they use the apex handle above).
    let control_handles = if is_selected {
        (0..edge_count)
            .filter_map(|ei| {
                let (c1, c2) = edge_controls(&corners, &bulges, &curves, ei)?;
                let bezier = is_bezier(&curves, ei);
                let anchors = [(&corners[ei], c1), (&corners[(ei + 1) % edge_count], c2)];
                let handles = anchors
                    .into_iter()
                    .enumerate()
                    .map(|(which, (anchor, ctrl))| {
                        let (hx, hy) = (t.sx(ctrl.x), t.sy(ctrl.y));
                        // A guide line joins a Bézier control to its anchor
                        // node; on a still-straight edge the handle sits on the
                        // chord, so a guide would just overdraw the edge.
                        let guide = bezier.then(|| {
                            view! {
                                <line
                                    class="shape-control-guide"
                                    x1=t.sx(anchor.x)
                                    y1=t.sy(anchor.y)
                                    x2=hx
                                    y2=hy
                                    stroke=SELECTED_STROKE
                                    stroke-width="1"
                                    stroke-dasharray="3,2"
                                />
                            }
                        });
                        view! {
                            {guide}
                            <circle
                                class="shape-control-handle"
                                data-testid="shape-control-handle"
                                cx=hx
                                cy=hy
                                r=CONTROL_HANDLE_R
                                fill=SELECTED_FILL
                                stroke=SELECTED_STROKE
                                on:mousedown=move |ev: leptos::ev::MouseEvent| {
                                    ev.stop_propagation();
                                    if let Some(cb) = on_control_press {
                                        cb.run((ei, which));
                                    }
                                }
                            />
                        }
                    })
                    .collect::<Vec<_>>();
                Some(handles)
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let n = f64::from(u32::try_from(corners.len()).unwrap_or(1).max(1));
    let cx = corners.iter().map(|c| t.sx(c.x)).sum::<f64>() / n;
    let cy = corners.iter().map(|c| t.sy(c.y)).sum::<f64>() / n;
    let curve_tuples: Vec<(usize, Point, Point)> = curves
        .iter()
        .filter_map(|c| {
            usize::try_from(c.edge).ok().map(|e| {
                (
                    e,
                    Point::new(c.control1.x, c.control1.y),
                    Point::new(c.control2.x, c.control2.y),
                )
            })
        })
        .collect();
    let ft2 = boundary_area(&corners, &bulges, &curve_tuples);
    let label = if elevation == 0.0 {
        format!("{ft2:.0} ft²")
    } else {
        format!("{ft2:.0} ft² · {elevation:+.1} ft")
    };
    // The unselected look comes from the area's material category (mulch is
    // bark brown, uncategorized is the neutral default); a selection tint
    // overrides it while selected.
    let (cat_fill, cat_stroke) = area_style(category.as_deref());
    let stroke = if is_selected {
        SELECTED_STROKE
    } else {
        cat_stroke
    };
    let (fill, fill_opacity) = surface_fill(
        SHAPE_PATTERN_PREFIX,
        is_selected,
        texture_id.as_deref(),
        cat_fill,
    );
    let mut class = String::from("shape");
    if is_selected {
        class.push_str(" shape--selected");
    }
    // The insert-between popup appears once an adjacent pair of nodes is
    // selected, floating near their midpoint.
    let popup = (selected_nodes.len() == 2).then(|| {
        let (mx, my) = match (corners.get(selected_nodes[0]), corners.get(selected_nodes[1])) {
            (Some(a), Some(b)) => (
                f64::midpoint(t.sx(a.x), t.sx(b.x)),
                f64::midpoint(t.sy(a.y), t.sy(b.y)),
            ),
            _ => (cx, cy),
        };
        view! {
            <g class="node-insert-popup" transform=format!("translate({mx},{my})")>
                <g
                    data-testid="insert-node"
                    on:mousedown=move |ev: leptos::ev::MouseEvent| {
                        ev.stop_propagation();
                        if let Some(cb) = on_insert_node {
                            cb.run(());
                        }
                    }
                >
                    <rect x="-34" y="-11" width="34" height="22" rx="3" fill="#fff" stroke=SELECTED_STROKE />
                    <text x="-17" y="4" text-anchor="middle" font-size="10" fill=SELECTED_STROKE>
                        "Insert"
                    </text>
                </g>
                <g
                    data-testid="cancel-node-select"
                    on:mousedown=move |ev: leptos::ev::MouseEvent| {
                        ev.stop_propagation();
                        if let Some(cb) = on_cancel_nodes {
                            cb.run(());
                        }
                    }
                >
                    <rect x="0" y="-11" width="34" height="22" rx="3" fill="#fff" stroke=SHAPE_STROKE />
                    <text x="17" y="4" text-anchor="middle" font-size="10" fill=SHAPE_STROKE>
                        "Cancel"
                    </text>
                </g>
            </g>
        }
    });
    // An all-straight boundary is a plain polygon; once any edge bows or
    // curves, the whole boundary is a path (arc `A` / bezier `C` commands for
    // the curved edges, lines for the rest) so it renders true-to-scale.
    let d = has_curve.then(|| boundary_path(t, &corners, &bulges, &curves));
    let body = if let Some(d) = d.clone() {
        view! {
            <path d=d fill=fill fill-opacity=fill_opacity stroke=stroke stroke-width="2" />
        }
        .into_any()
    } else {
        view! {
            <polygon
                points=points.clone()
                fill=fill
                fill-opacity=fill_opacity
                stroke=stroke
                stroke-width="2"
            />
        }
        .into_any()
    };
    // Border rings (B5): the boundary stroked with each ring's material,
    // clipped to the area — painter's algorithm with cumulative widths
    // (innermost drawn first, outermost last), so ring *j* shows as a band
    // `width_ft` wide just inside ring *j−1*. Sits over the field fill,
    // under the markers/handles.
    let borders_view = (!border_paints.is_empty()).then(|| {
        let clip_id = format!("{SHAPE_PATTERN_PREFIX}-border-clip-{i}");
        let clip_shape = if let Some(d) = d.clone() {
            view! { <path d=d /> }.into_any()
        } else {
            view! { <polygon points=points.clone() /> }.into_any()
        };
        // Per-edge offset stacking (see `border_strokes`) resolves each band
        // to runs at an outer offset; each run then renders as its true
        // **offset ribbon** (`ribbon_path`) — exact one-sided geometry, so a
        // band never bleeds across its edge into another part of the interior
        // (a centered stroke does, at a concave pocket's lip). The area clip
        // stays as a guard for tight-curvature self-intersections, where an
        // inner offset can poke through the opposite boundary.
        let plan: Vec<(f64, Option<(usize, usize)>)> = border_paints
            .iter()
            .map(|bp| (bp.width_ft, bp.span))
            .collect();
        let signed2: f64 = {
            let spt: Vec<(f64, f64)> = corners.iter().map(|c| (t.sx(c.x), t.sy(c.y))).collect();
            (0..corners.len())
                .map(|k| {
                    let (a, b) = (spt[k], spt[(k + 1) % corners.len()]);
                    a.0 * b.1 - b.0 * a.1
                })
                .sum()
        };
        let inward = move |dir: (f64, f64)| -> (f64, f64) {
            if signed2 > 0.0 {
                (-dir.1, dir.0)
            } else {
                (dir.1, -dir.0)
            }
        };
        let ring_views = border_strokes(&plan, corners.len())
            .into_iter()
            .filter_map(|st| {
                let bp = &border_paints[st.border];
                let samples = sample_run(t, &corners, &bulges, &curves, st.run, inward);
                let d = ribbon_path(
                    &samples,
                    (st.depth_ft - bp.width_ft) * t.px_ft,
                    st.depth_ft * t.px_ft,
                    st.run.is_none(),
                )?;
                Some(
                    view! {
                        <path
                            class="shape-border"
                            data-testid="shape-border"
                            d=d
                            fill=bp.paint.clone()
                            fill-opacity=bp.opacity
                            clip-path=format!("url(#{clip_id})")
                        />
                    }
                    .into_any(),
                )
            })
            .collect::<Vec<_>>();
        // Junction patches: where two covered edges meet at a reflex corner,
        // fill the butt-cap pie gap with the exact miter quad, painted like
        // the deeper side's innermost band (clipped to the area like the
        // strokes). Convex corners need none — adjacent bands overlap there.
        let depths = edge_band_depths(&plan, corners.len());
        let n = corners.len();
        let spt: Vec<(f64, f64)> = corners.iter().map(|c| (t.sx(c.x), t.sy(c.y))).collect();
        // The polygon's winding fixes which perpendicular of a travel tangent
        // points inward (interior is left of travel for a positive screen
        // shoelace) — exact at every node, curved edges included, where a
        // point-probe against the straight-chord outline can misclassify.
        let signed2: f64 = (0..n)
            .map(|k| {
                let (a, b) = (spt[k], spt[(k + 1) % n]);
                a.0 * b.1 - b.0 * a.1
            })
            .sum();
        let inward = |dir: (f64, f64)| -> (f64, f64) {
            if signed2 > 0.0 {
                (-dir.1, dir.0)
            } else {
                (dir.1, -dir.0)
            }
        };
        let joints = (0..n)
            .filter_map(|v| {
                let (e_prev, e_cur) = ((v + n - 1) % n, v);
                let (d_prev, _) = depths[e_prev];
                let (d_cur, _) = depths[e_cur];
                if d_prev <= 0.0 || d_cur <= 0.0 {
                    return None;
                }
                // The true tangents at the node — a curved edge's chord can
                // point somewhere else entirely.
                let (_, dir_prev) = edge_end_tangents(&corners, &bulges, &curves, e_prev)?;
                let (dir_cur, _) = edge_end_tangents(&corners, &bulges, &curves, e_cur)?;
                let quad = junction_quad(
                    spt[v],
                    dir_prev,
                    dir_cur,
                    inward(dir_prev),
                    inward(dir_cur),
                    d_prev * t.px_ft,
                    d_cur * t.px_ft,
                )?;
                // Painted like the deeper side's innermost band.
                let top = if d_prev >= d_cur {
                    depths[e_prev].1
                } else {
                    depths[e_cur].1
                }?;
                let bp = &border_paints[top];
                let pts = quad
                    .iter()
                    .map(|(x, y)| format!("{x},{y}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                Some(view! {
                    <polygon
                        class="shape-border-joint"
                        data-testid="shape-border-joint"
                        points=pts
                        fill=bp.paint.clone()
                        fill-opacity=bp.opacity
                        clip-path=format!("url(#{clip_id})")
                    />
                })
            })
            .collect::<Vec<_>>();
        view! {
            <clipPath id=clip_id.clone()>{clip_shape}</clipPath>
            {ring_views}
            {joints}
        }
    });
    view! {
        <g
            class=class
            on:mousedown=move |_ev: leptos::ev::MouseEvent| {
                if let Some(cb) = on_shape_press {
                    cb.run(i);
                }
            }
        >
            {body}
            {borders_view}
            {markers}
            {edge_handles}
            {control_handles}
            <text class="shape-label" x=cx y=cy text-anchor="middle" font-size="11" fill="#5a5540">
                {label}
            </text>
            {popup}
        </g>
    }
}

/// The SVG `<path>` `d` for a closed boundary in screen space: `M` to the
/// first node, then per edge a `C` (cubic-Bézier) command when the edge has
/// controls, else an `A` (arc) command when its bulge is non-zero, else an `L`
/// (line), closing with `Z`. Edge `i` runs node `i`→`i+1`; a curve takes
/// precedence over a bulge. The arc's screen radius is its world chord scaled
/// by `t.px_ft` (the transform is isotropic).
fn boundary_path(t: Transform, corners: &[Coord], bulges: &[f64], curves: &[CurveEdge]) -> String {
    let len = corners.len();
    let first = &corners[0];
    let mut path = format!("M {} {}", t.sx(first.x), t.sy(first.y));
    for i in 0..len {
        edge_command(&mut path, t, corners, bulges, curves, i);
    }
    path.push_str(" Z");
    path
}

/// Append boundary edge `i`'s SVG command to `path`: a `C` when the edge has
/// Bézier controls, an `A` when its bulge bows it, else an `L` — shared by the
/// closed boundary path and an open border span.
fn edge_command(
    path: &mut String,
    t: Transform,
    corners: &[Coord],
    bulges: &[f64],
    curves: &[CurveEdge],
    i: usize,
) {
    use std::fmt::Write as _;
    let len = corners.len();
    let sx = |c: &Coord| t.sx(c.x);
    let sy = |c: &Coord| t.sy(c.y);
    let from = &corners[i];
    let to = &corners[(i + 1) % len];
    if let Some(c) = curves.iter().find(|c| usize::try_from(c.edge) == Ok(i)) {
        let _ = write!(
            path,
            " C {} {} {} {} {} {}",
            sx(&c.control1),
            sy(&c.control1),
            sx(&c.control2),
            sy(&c.control2),
            sx(to),
            sy(to),
        );
        return;
    }
    let bulge = bulges.get(i).copied().unwrap_or(0.0);
    let world_chord = (from.x - to.x).hypot(from.y - to.y);
    match arc_svg(world_chord * t.px_ft, bulge) {
        Some(arc) => {
            let (large, sweep) = (u8::from(arc.large_arc), u8::from(arc.sweep));
            let _ = write!(
                path,
                " A {r} {r} 0 {large} {sweep} {x} {y}",
                r = arc.radius,
                x = sx(to),
                y = sy(to),
            );
        }
        None => {
            let _ = write!(path, " L {} {}", sx(to), sy(to));
        }
    }
}

/// The world-space apex of edge `i` (node `i`→next): the point on its arc
/// farthest from the chord — where the bulge handle sits. The sagitta is
/// `bulge · (chord/2)` along the chord's left normal (matching the renderer's
/// "positive bulge bows left" convention), so a straight edge's apex is just
/// the chord midpoint. `None` for a degenerate (zero-length) edge.
fn edge_apex(corners: &[Coord], bulges: &[f64], i: usize) -> Option<(f64, f64)> {
    let n = corners.len();
    let from = &corners[i];
    let to = &corners[(i + 1) % n];
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let chord = dx.hypot(dy);
    if chord < 1e-9 {
        return None;
    }
    let (mx, my) = (from.x.midpoint(to.x), from.y.midpoint(to.y));
    // Left normal of `from`→`to` (a +90° / CCW turn).
    let (nx, ny) = (-dy / chord, dx / chord);
    let sagitta = bulges.get(i).copied().unwrap_or(0.0) * (chord / 2.0);
    Some((mx + sagitta * nx, my + sagitta * ny))
}

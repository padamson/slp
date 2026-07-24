//! Cost take-off: turn a plan into a priced bill of materials.
//!
//! Two kinds of catalog entry are costed, chosen by each item's `price_unit`:
//! **objects** (priced per item) and drawn-area **materials** (priced per ft²
//! of surface, or per yd³ of volume at a bed's depth). An object line counts
//! only placements that are both **planned** (`status`) and **real**
//! (`!is_virtual`): `existing` objects are already owned, and a `virtual`
//! object is a what-if ghost, never a second real item. A material line sums
//! the area (or volume) of every drawn `Shape`/`Circle` whose `material_ref`
//! names it.

use crate::generated::slp::{Border, CatalogItem, Course, ItemStatus, Plan, PriceUnit};
use crate::{Point, Shape, boundary_area, boundary_perimeter, boundary_span_length, circle_area};

/// One line of the bill of materials: a catalog item/material, the measured
/// quantity referencing it (a count of objects, ft² of surface, or yd³ of
/// volume — `unit` says which), and the dollars it adds up to.
#[derive(Debug, Clone, PartialEq)]
pub struct LineItem {
    /// The catalog item's `id` — objects reference it via `catalog_ref`, drawn
    /// areas via `material_ref`.
    pub catalog_ref: String,
    /// The catalog item's display name, if it has one.
    pub name: Option<String>,
    /// The measured quantity: a whole number of objects for a per-item line,
    /// or a ft²/yd³ measure for a material line (`unit` disambiguates).
    pub quantity: f64,
    /// What `quantity` measures (and `unit_price` is charged per).
    pub unit: PriceUnit,
    /// Price per unit, in dollars; `0.0` when the catalog item has no price.
    pub unit_price: f64,
    /// `quantity × unit_price`, in dollars.
    pub line_total: f64,
    /// The chosen **laying patterns** of the drawn areas this material line
    /// sums (deduped, in drawn order) — the layout note the shopping trip
    /// needs. Empty for object lines and pattern-less areas.
    pub patterns: Vec<String>,
}

/// A priced bill of materials for a plan: one line per placed catalog item plus
/// the grand total, in dollars.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BillOfMaterials {
    /// Line items, in catalog order. Catalog items with no planned placements
    /// are omitted.
    pub lines: Vec<LineItem>,
    /// Sum of every line total, in dollars.
    pub grand_total: f64,
}

/// Cost take-off for a plan.
///
/// Each catalog item is costed per its `price_unit`: a per-item line counts the
/// **planned** and **real** (`!is_virtual`) objects referencing it (`existing`
/// objects are already owned and `virtual` ones are ghosts — both excluded); a
/// per-ft²/per-yd³ material line sums the surface area / volume of every drawn
/// `Shape`/`Circle` whose `material_ref` names it — **minus** any border rings
/// (B5), which cost separately: a per-ft² border material adds its ring areas,
/// and a per-linear-ft material (edging stones) sums its rings' centerline
/// perimeters. Lines come out in catalog order; an item with no counted
/// quantity yields no line. A catalog item with no `unit_price` is priced at
/// `0.0`.
#[must_use]
pub fn take_off(plan: &Plan) -> BillOfMaterials {
    let mut lines = Vec::new();
    let mut grand_total = 0.0;
    for item in &plan.catalog {
        let quantity = match item.price_unit {
            PriceUnit::per_item => object_count(plan, &item.id),
            PriceUnit::per_square_foot => material_area(plan, &item.id),
            PriceUnit::per_cubic_yard => material_volume(plan, &item.id),
            PriceUnit::per_linear_foot => material_linear(plan, &item.id),
        };
        if quantity <= 0.0 {
            continue;
        }
        let unit_price = item.unit_price.unwrap_or(0.0);
        let line_total = quantity * unit_price;
        grand_total += line_total;
        lines.push(LineItem {
            catalog_ref: item.id.clone(),
            name: item.name.clone(),
            quantity,
            unit: item.price_unit.clone(),
            unit_price,
            line_total,
            patterns: chosen_patterns(plan, &item.id),
        });
    }
    BillOfMaterials { lines, grand_total }
}

/// The chosen laying patterns of the drawn areas whose `material_ref` names
/// catalog item `id`, deduped in drawn order (shapes then circles) — the
/// layout note a material's estimate line carries.
fn chosen_patterns(plan: &Plan, id: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let shape_pats = plan
        .shapes
        .iter()
        .filter(|s| s.material_ref.as_deref() == Some(id))
        .filter_map(|s| s.pattern.clone());
    let circle_pats = plan
        .circles
        .iter()
        .filter(|c| c.material_ref.as_deref() == Some(id))
        .filter_map(|c| c.pattern.clone());
    for p in shape_pats.chain(circle_pats) {
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// The number of **planned** and **real** objects placing catalog item `id`.
fn object_count(plan: &Plan, id: &str) -> f64 {
    let n = plan
        .objects
        .iter()
        .filter(|o| o.status == ItemStatus::planned && !o.is_virtual && o.catalog_ref == id)
        .count();
    // A plan never holds anywhere near `u32::MAX` objects, so the saturating
    // conversion is exact; `f64::from(u32)` is then lossless.
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

/// The total surface area (ft²) of material `id` across the plan: the
/// **field** of every drawn `Shape`/`Circle` made of it (its area minus its
/// border rings' area — the border is bought separately), plus every border
/// **ring** made of it on any area (a contrasting per-ft² border paver).
fn material_area(plan: &Plan, id: &str) -> f64 {
    area_measures(plan)
        .iter()
        .map(|m| {
            let rings = ring_measures(&m.rings);
            let field = if m.material_ref == Some(id) {
                let border_area: f64 = rings.iter().map(|r| r.area_ft2).sum();
                (m.area_ft2 - border_area).max(0.0)
            } else {
                0.0
            };
            let ring_share: f64 = m
                .rings
                .iter()
                .zip(&rings)
                .filter(|(spec, _)| spec.material_ref == id)
                .map(|(_, r)| r.area_ft2)
                .sum();
            field + ring_share
        })
        .sum()
}

/// The total linear feet of material `id` across the plan: the centerline
/// perimeter of every border ring made of it (edging stones and other
/// per-linear-ft border products).
fn material_linear(plan: &Plan, id: &str) -> f64 {
    area_measures(plan)
        .iter()
        .map(|m| {
            m.rings
                .iter()
                .zip(ring_measures(&m.rings))
                .filter(|(spec, _)| spec.material_ref == id)
                .map(|(_, r)| r.centerline_ft)
                .sum::<f64>()
        })
        .sum()
}

/// One border ring's measured geometry under the rounded-offset model.
struct RingMeasure {
    /// The ring centerline's length (ft) — what a per-linear-ft product costs by.
    centerline_ft: f64,
    /// The ring band's area (ft²) — what a per-ft² border material costs by,
    /// and what the field loses.
    area_ft2: f64,
}

/// One border band's resolved geometry inputs: its material, laid width, the
/// centerline's base length, and whether the band wraps the whole boundary
/// (a closed ring) or runs open along a node span.
struct RingSpec<'a> {
    material_ref: &'a str,
    width_ft: f64,
    /// The band centerline's base length: the closed boundary perimeter for a
    /// ring, the span's edge length for an open run (0 for a degenerate span).
    base_len_ft: f64,
    /// Whether the band wraps the boundary — only a closed ring shrinks by
    /// `2π × offset` as it nests inward (an open band has no corners to
    /// round off; its inner-offset length change is ignored).
    closed: bool,
}

/// Resolve a border into its [`RingSpec`]: a full-perimeter ring unless
/// **both** span positions are set, in which case the open span from
/// `start_node` forward to `end_node` (a stale or empty span measures 0 — it
/// under-counts loudly rather than silently billing the whole perimeter).
/// `span_len` resolves the span for the concrete boundary (shapes have nodes;
/// a circle has none and always rings).
fn ring_spec(b: &Border, perimeter_ft: f64, span_len: impl Fn(f64, f64) -> f64) -> RingSpec<'_> {
    let span = match (b.start_node, b.end_node) {
        (Some(s), Some(e)) => Some(span_len(s, e)),
        _ => None,
    };
    RingSpec {
        material_ref: &b.material_ref,
        width_ft: b.width_ft,
        base_len_ft: span.unwrap_or(perimeter_ft),
        closed: span.is_none(),
    }
}

/// Measure each border band, outermost first. Insetting a **closed** boundary
/// by `d` shortens its perimeter by `2π·d` (corners round off — exact for a
/// circle, a good buy-estimate approximation elsewhere), so ring *i* at
/// cumulative outer offset `o` has centerline length `P − 2π·(o + wᵢ/2)`,
/// clamped at 0 once the rings have consumed the boundary; its band area is
/// exactly `centerline × wᵢ` under that model. An **open** span band keeps
/// its base length at any offset. A non-positive band width measures nothing.
fn ring_measures(rings: &[RingSpec]) -> Vec<RingMeasure> {
    let mut offset = 0.0;
    rings
        .iter()
        .map(|r| {
            let w = r.width_ft.max(0.0);
            let centerline_ft = if r.closed {
                (r.base_len_ft - std::f64::consts::TAU * (offset + w / 2.0)).max(0.0)
            } else {
                r.base_len_ft
            };
            offset += w;
            RingMeasure {
                centerline_ft,
                area_ft2: centerline_ft * w,
            }
        })
        .collect()
}

/// The total volume (yd³) of material `id` across the plan: every drawn area
/// made *of* it (at that area's own depth, e.g. a mulch or gravel bed), **plus**
/// its volume as a sub-layer beneath a surface. The sub-layers come from each
/// area's **own** `courses` (per-area composition); an area with no courses
/// falls back to its surface material's catalog default base/bedding (B2.2).
/// All by `yd³ = ft²·depth_in / 324`.
fn material_volume(plan: &Plan, id: &str) -> f64 {
    let mut volume = 0.0;
    for m in area_measures(plan) {
        // A bed literally made of `id` (mulch, gravel), at its own depth.
        if m.material_ref == Some(id) {
            volume += volume_yd3(m.area_ft2, m.depth_in);
        }
        // The sub-base beneath a surface: this area's own courses when it has
        // them, else its surface material's catalog default courses. Courses
        // use the **full** area — the base and bedding run beneath the border
        // rings too (a border sits on the same prepared bed as the field).
        let fallback;
        let effective: &[Course] = if m.courses.is_empty() {
            fallback = m
                .material_ref
                .and_then(|mat| catalog_item(plan, mat))
                .map(default_courses)
                .unwrap_or_default();
            &fallback
        } else {
            m.courses
        };
        for course in effective {
            if course.material_ref == id {
                volume += volume_yd3(m.area_ft2, course.depth_in);
            }
        }
    }
    volume
}

/// The default sub-base courses a drawn area inherits from its surface material:
/// the catalog item's base course then bedding course, when it declares them (a
/// paver's ~4 in gravel then ~1 in sand). Empty for a material with no sub-base
/// (a mulch bed, a bare surface). A freshly-drawn paver area is seeded with
/// these, and an area left with no `courses` of its own is costed by them.
#[must_use]
pub fn default_courses(item: &CatalogItem) -> Vec<Course> {
    let mut courses = Vec::new();
    if let (Some(base), Some(depth)) = (&item.base_material_ref, item.base_depth_in) {
        courses.push(Course::new(depth, base.clone()));
    }
    if let (Some(bed), Some(depth)) = (&item.bedding_material_ref, item.bedding_depth_in) {
        courses.push(Course::new(depth, bed.clone()));
    }
    courses
}

/// Tile size (ft) assumed for a material photo when the catalog item declares
/// no `tile_width_ft`/`tile_depth_in` — the schema's promised "sensible
/// default". Lives here (not in a render layer) so 2D tiling, the future 3D
/// albedo, and thumbnails all resolve the same effective size.
pub const DEFAULT_TILE_FT: f64 = 2.0;

/// The effective real-world span (E–W ft, N–S ft) of a material's photo tile:
/// the item's declared `tile_width_ft`/`tile_depth_ft`, each falling back to
/// [`DEFAULT_TILE_FT`] when absent or non-positive (0 = "use the default",
/// per the schema).
#[must_use]
pub fn tile_size_ft(item: &CatalogItem) -> (f64, f64) {
    let effective = |v: Option<f64>| v.filter(|v| *v > 0.0).unwrap_or(DEFAULT_TILE_FT);
    (effective(item.tile_width_ft), effective(item.tile_depth_ft))
}

/// One drawn area's raw take-off inputs (a `Shape` or a `Circle`).
struct AreaMeasure<'a> {
    /// The surface material the area is made of, if any.
    material_ref: Option<&'a str>,
    /// The enclosed area, ft².
    area_ft2: f64,
    /// The area's own material depth (a bed's inches), 0 when unset.
    depth_in: f64,
    /// The area's own sub-base composition (empty = catalog default).
    courses: &'a [Course],
    /// The area's border bands, outermost first, resolved to ring/span
    /// geometry (empty = none).
    rings: Vec<RingSpec<'a>>,
}

/// Every drawn area's measures — shapes then circles — the raw inputs for
/// area/volume/border take-off.
fn area_measures(plan: &Plan) -> Vec<AreaMeasure<'_>> {
    let shapes = plan.shapes.iter().map(|s| {
        let curves = shape_curves(s);
        let perimeter = boundary_perimeter(&s.corners, &s.bulges, &curves);
        AreaMeasure {
            material_ref: s.material_ref.as_deref(),
            area_ft2: shape_area(s),
            depth_in: s.depth_in.unwrap_or(0.0),
            courses: s.courses.as_slice(),
            rings: s
                .borders
                .iter()
                .map(|b| {
                    ring_spec(b, perimeter, |from, to| {
                        boundary_span_length(&s.corners, &s.bulges, &curves, from, to)
                    })
                })
                .collect(),
        }
    });
    let circles = plan.circles.iter().map(|c| {
        // A circle has no nodes: every border is a full ring (span fields are
        // ignored by resolving each one against the circumference).
        let perimeter = std::f64::consts::TAU * c.radius_ft;
        AreaMeasure {
            material_ref: c.material_ref.as_deref(),
            area_ft2: circle_area(c.radius_ft),
            depth_in: c.depth_in.unwrap_or(0.0),
            courses: c.courses.as_slice(),
            rings: c
                .borders
                .iter()
                .map(|b| RingSpec {
                    material_ref: &b.material_ref,
                    width_ft: b.width_ft,
                    base_len_ft: perimeter,
                    closed: true,
                })
                .collect(),
        }
    });
    shapes.chain(circles).collect()
}

/// The catalog item with `id`, if the plan's catalog holds one.
fn catalog_item<'a>(plan: &'a Plan, id: &str) -> Option<&'a CatalogItem> {
    plan.catalog.iter().find(|c| c.id == id)
}

/// A shape's Bézier edges as `(edge index, control1, control2)`, the form the
/// boundary area/perimeter functions take.
fn shape_curves(s: &Shape) -> Vec<(usize, Point, Point)> {
    s.curves
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
        .collect()
}

/// A shape's enclosed area (ft²), accounting for any arc/curve edges.
fn shape_area(s: &Shape) -> f64 {
    boundary_area(&s.corners, &s.bulges, &shape_curves(s))
}

/// Volume in cubic yards of a `ft²` surface at `depth_in` inches:
/// `yd³ = ft²·depth_in / 324` (324 = 27 ft³/yd³ × 12 in/ft).
fn volume_yd3(area_ft2: f64, depth_in: f64) -> f64 {
    area_ft2 * depth_in / 324.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::slp::{CatalogItem, Object};

    fn item(id: &str, name: &str, price: Option<f64>) -> CatalogItem {
        let mut c = CatalogItem::new(id.to_string());
        c.name = Some(name.to_string());
        c.unit_price = price;
        c
    }

    fn placed(catalog_ref: &str, status: ItemStatus) -> Object {
        let mut o = Object::new(catalog_ref.to_string(), 0.0, 0.0);
        o.status = status;
        o
    }

    fn virtual_placed(catalog_ref: &str, status: ItemStatus) -> Object {
        let mut o = placed(catalog_ref, status);
        o.is_virtual = true;
        o
    }

    fn plan(catalog: Vec<CatalogItem>, objects: Vec<Object>) -> Plan {
        let mut p = Plan::new(20.0, 30.0);
        p.catalog = catalog;
        p.objects = objects;
        p
    }

    #[test]
    fn empty_plan_has_no_lines_and_zero_total() {
        let bom = take_off(&plan(vec![], vec![]));
        assert!(bom.lines.is_empty());
        assert!(bom.grand_total.abs() < 1e-9);
    }

    #[test]
    fn prices_quantities_and_grand_total() {
        // 2 chairs @ $49.50 + 1 table @ $200 = $99 + $200 = $299.
        let bom = take_off(&plan(
            vec![
                item("chair", "Patio chair", Some(49.50)),
                item("table", "Dining table", Some(200.0)),
            ],
            vec![
                placed("chair", ItemStatus::planned),
                placed("chair", ItemStatus::planned),
                placed("table", ItemStatus::planned),
            ],
        ));
        assert_eq!(bom.lines.len(), 2);
        // Lines come out in catalog order: chair, then table.
        assert_eq!(bom.lines[0].catalog_ref, "chair");
        assert!((bom.lines[0].quantity - 2.0).abs() < 1e-9);
        assert!((bom.lines[0].line_total - 99.0).abs() < 1e-9);
        assert_eq!(bom.lines[1].catalog_ref, "table");
        assert!((bom.lines[1].quantity - 1.0).abs() < 1e-9);
        assert!((bom.lines[1].line_total - 200.0).abs() < 1e-9);
        assert!((bom.grand_total - 299.0).abs() < 1e-9);
    }

    #[test]
    fn existing_placements_are_not_counted() {
        // Same item placed once each as planned / existing (both real): only
        // the planned one is bought.
        let bom = take_off(&plan(
            vec![item("chair", "Patio chair", Some(50.0))],
            vec![
                placed("chair", ItemStatus::planned),
                placed("chair", ItemStatus::existing),
            ],
        ));
        assert_eq!(bom.lines.len(), 1);
        assert!((bom.lines[0].quantity - 1.0).abs() < 1e-9);
        assert!((bom.grand_total - 50.0).abs() < 1e-9);
    }

    #[test]
    fn virtual_placements_are_never_counted_regardless_of_status() {
        // A what-if ghost of a planned item, and one of an existing item:
        // neither is a second real item, so neither is bought. Only the one
        // real planned placement counts.
        let bom = take_off(&plan(
            vec![item("chair", "Patio chair", Some(50.0))],
            vec![
                placed("chair", ItemStatus::planned),
                virtual_placed("chair", ItemStatus::planned),
                virtual_placed("chair", ItemStatus::existing),
            ],
        ));
        assert_eq!(bom.lines.len(), 1);
        assert!((bom.lines[0].quantity - 1.0).abs() < 1e-9);
        assert!((bom.grand_total - 50.0).abs() < 1e-9);
    }

    #[test]
    fn the_default_status_is_counted() {
        // `Object::new` leaves status at its schema default (planned), so a
        // placement created without an explicit status still costs.
        let bom = take_off(&plan(
            vec![item("chair", "Patio chair", Some(50.0))],
            vec![Object::new("chair".to_string(), 1.0, 2.0)],
        ));
        assert!((bom.lines[0].quantity - 1.0).abs() < 1e-9);
        assert!((bom.grand_total - 50.0).abs() < 1e-9);
    }

    #[test]
    fn unresolved_catalog_refs_are_excluded() {
        // An object referencing an id that isn't in the catalog can't be priced,
        // so it contributes no line and nothing to the total.
        let bom = take_off(&plan(
            vec![item("chair", "Patio chair", Some(50.0))],
            vec![
                placed("chair", ItemStatus::planned),
                placed("ghost-id", ItemStatus::planned),
            ],
        ));
        assert_eq!(bom.lines.len(), 1);
        assert!((bom.lines[0].quantity - 1.0).abs() < 1e-9);
        assert!((bom.grand_total - 50.0).abs() < 1e-9);
    }

    #[test]
    fn catalog_items_with_no_placements_yield_no_line() {
        // "bench" is in the catalog but never placed, so it must not appear.
        let bom = take_off(&plan(
            vec![
                item("chair", "Patio chair", Some(50.0)),
                item("bench", "Garden bench", Some(300.0)),
            ],
            vec![placed("chair", ItemStatus::planned)],
        ));
        assert_eq!(bom.lines.len(), 1);
        assert_eq!(bom.lines[0].catalog_ref, "chair");
        assert!((bom.grand_total - 50.0).abs() < 1e-9);
    }

    #[test]
    fn missing_unit_price_is_treated_as_zero() {
        // A placed item with no price still shows its quantity, priced at $0.
        let bom = take_off(&plan(
            vec![item("chair", "Patio chair", None)],
            vec![
                placed("chair", ItemStatus::planned),
                placed("chair", ItemStatus::planned),
            ],
        ));
        assert!((bom.lines[0].quantity - 2.0).abs() < 1e-9);
        assert!(bom.lines[0].unit_price.abs() < 1e-9);
        assert!(bom.lines[0].line_total.abs() < 1e-9);
        assert!(bom.grand_total.abs() < 1e-9);
    }

    // --- Area/volume materials (mulch, pavers) ---

    use crate::Coord;
    use crate::generated::slp::{Circle, Shape};

    fn material(id: &str, name: &str, price: f64, unit: PriceUnit) -> CatalogItem {
        let mut c = item(id, name, Some(price));
        c.price_unit = unit;
        c
    }

    /// A `w`×`d` rectangular shape made of material `mat`, `depth` inches deep.
    fn bed(mat: &str, w: f64, d: f64, depth: f64) -> Shape {
        Shape {
            corners: vec![
                Coord::new(0.0, 0.0),
                Coord::new(w, 0.0),
                Coord::new(w, d),
                Coord::new(0.0, d),
            ],
            material_ref: Some(mat.to_string()),
            depth_in: Some(depth),
            ..Shape::new(0.0)
        }
    }

    fn plan_with_areas(
        catalog: Vec<CatalogItem>,
        shapes: Vec<Shape>,
        circles: Vec<Circle>,
    ) -> Plan {
        let mut p = Plan::new(40.0, 40.0);
        p.catalog = catalog;
        p.shapes = shapes;
        p.circles = circles;
        p
    }

    #[test]
    fn a_per_square_foot_material_sums_its_areas() {
        // Two paver areas (10×8 = 80 ft² and 5×4 = 20 ft²) = 100 ft² × $6 = $600.
        let bom = take_off(&plan_with_areas(
            vec![material("paver", "Pavers", 6.0, PriceUnit::per_square_foot)],
            vec![bed("paver", 10.0, 8.0, 0.0), bed("paver", 5.0, 4.0, 0.0)],
            vec![],
        ));
        assert_eq!(bom.lines.len(), 1);
        assert_eq!(bom.lines[0].unit, PriceUnit::per_square_foot);
        assert!((bom.lines[0].quantity - 100.0).abs() < 1e-9, "100 ft²");
        assert!((bom.lines[0].line_total - 600.0).abs() < 1e-9);
        assert!((bom.grand_total - 600.0).abs() < 1e-9);
    }

    #[test]
    fn a_material_line_notes_its_areas_chosen_laying_patterns() {
        // Two paver areas with patterns (one duplicated), one without, and a
        // circle with its own — the line lists each chosen pattern once, in
        // drawn order (shapes then circles); the mulch line (no patterns on
        // its areas) carries none. An area whose material is *different*
        // doesn't leak its pattern onto this line.
        let mut a = bed("paver", 10.0, 8.0, 0.0);
        a.pattern = Some("Herringbone".to_string());
        let mut b = bed("paver", 5.0, 4.0, 0.0);
        b.pattern = Some("Linear".to_string());
        let mut dup = bed("paver", 2.0, 2.0, 0.0);
        dup.pattern = Some("Herringbone".to_string());
        let plain = bed("paver", 1.0, 1.0, 0.0);
        let mut mulch = bed("mulch", 3.0, 3.0, 3.0);
        mulch.pattern = Some("Not A Paver Pattern".to_string());
        let mut circle = Circle {
            material_ref: Some("paver".to_string()),
            depth_in: Some(0.0),
            ..Circle::new(Box::new(Coord::new(20.0, 20.0)), 0.0, 4.0)
        };
        circle.pattern = Some("Basket Weave".to_string());
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Pavers", 6.0, PriceUnit::per_square_foot),
                material("mulch", "Mulch", 40.0, PriceUnit::per_cubic_yard),
            ],
            vec![a, b, dup, plain, mulch],
            vec![circle],
        ));
        let paver = bom.lines.iter().find(|l| l.catalog_ref == "paver").unwrap();
        assert_eq!(
            paver.patterns,
            vec!["Herringbone", "Linear", "Basket Weave"],
            "deduped, drawn order, shapes before circles"
        );
        let mulch = bom.lines.iter().find(|l| l.catalog_ref == "mulch").unwrap();
        assert_eq!(
            mulch.patterns,
            vec!["Not A Paver Pattern"],
            "each line notes only its own areas' patterns"
        );
    }

    #[test]
    fn a_per_square_foot_material_sums_shapes_and_circles_together() {
        // A paver shape (10×8 = 80 ft²) AND a paver circle (radius 4 →
        // π·16 ≈ 50.27 ft²) both count toward the one paver line, summed — so
        // the circle's area must be added (not dropped or subtracted).
        let circle = Circle {
            material_ref: Some("paver".to_string()),
            depth_in: Some(0.0),
            ..Circle::new(Box::new(Coord::new(20.0, 20.0)), 0.0, 4.0)
        };
        let bom = take_off(&plan_with_areas(
            vec![material("paver", "Pavers", 6.0, PriceUnit::per_square_foot)],
            vec![bed("paver", 10.0, 8.0, 0.0)],
            vec![circle],
        ));
        let expected = 80.0 + circle_area(4.0);
        assert!(
            (bom.lines[0].quantity - expected).abs() < 1e-9,
            "shape 80 ft² + circle {:.2} ft² = {expected:.2}, got {}",
            circle_area(4.0),
            bom.lines[0].quantity
        );
    }

    #[test]
    fn a_per_cubic_yard_material_costs_by_volume_at_its_depth() {
        // A 10×8 = 80 ft² mulch bed, 3 in deep: yd³ = 80·3/324 = 0.7407…;
        // × $40/yd³ ≈ $29.63.
        let bom = take_off(&plan_with_areas(
            vec![material("mulch", "Mulch", 40.0, PriceUnit::per_cubic_yard)],
            vec![bed("mulch", 10.0, 8.0, 3.0)],
            vec![],
        ));
        assert_eq!(bom.lines.len(), 1);
        assert_eq!(bom.lines[0].unit, PriceUnit::per_cubic_yard);
        let yd3 = 80.0 * 3.0 / 324.0;
        assert!((bom.lines[0].quantity - yd3).abs() < 1e-9);
        assert!((bom.lines[0].line_total - yd3 * 40.0).abs() < 1e-9);
    }

    #[test]
    fn a_circle_bed_contributes_its_disk_volume() {
        // A round mulch bed, radius 4 ft (area π·16 ≈ 50.27 ft²), 3 in deep.
        let circle = Circle {
            material_ref: Some("mulch".to_string()),
            depth_in: Some(3.0),
            ..Circle::new(Box::new(Coord::new(10.0, 10.0)), 0.0, 4.0)
        };
        let bom = take_off(&plan_with_areas(
            vec![material("mulch", "Mulch", 40.0, PriceUnit::per_cubic_yard)],
            vec![],
            vec![circle],
        ));
        let yd3 = circle_area(4.0) * 3.0 / 324.0;
        assert!((bom.lines[0].quantity - yd3).abs() < 1e-9);
    }

    #[test]
    fn a_material_with_no_beds_yields_no_line() {
        let bom = take_off(&plan_with_areas(
            vec![material("mulch", "Mulch", 40.0, PriceUnit::per_cubic_yard)],
            vec![],
            vec![],
        ));
        assert!(bom.lines.is_empty());
    }

    #[test]
    fn a_bed_of_a_different_material_is_not_counted() {
        // A gravel bed doesn't add to the mulch line.
        let bom = take_off(&plan_with_areas(
            vec![material("mulch", "Mulch", 40.0, PriceUnit::per_cubic_yard)],
            vec![bed("gravel", 10.0, 8.0, 3.0)],
            vec![],
        ));
        assert!(bom.lines.is_empty(), "no mulch bed, so no mulch line");
    }

    // --- Paver sub-base (gravel + bedding) driven off the paver assembly ---

    /// A paver material sitting on `base_depth` in of `base` gravel and
    /// `bed_depth` in of `bed` sand.
    fn paver_on(base: &str, base_depth: f64, bed: &str, bed_depth: f64) -> CatalogItem {
        let mut c = material("paver", "Pavers", 6.0, PriceUnit::per_square_foot);
        c.base_material_ref = Some(base.to_string());
        c.base_depth_in = Some(base_depth);
        c.bedding_material_ref = Some(bed.to_string());
        c.bedding_depth_in = Some(bed_depth);
        c
    }

    #[test]
    fn a_paver_area_drives_gravel_base_and_bedding_sand_volumes() {
        // A 10×10 = 100 ft² paver patio on 4 in gravel + 1 in sand:
        //   gravel = 100·4/324 ≈ 1.235 yd³ × $50 ≈ $61.73
        //   sand   = 100·1/324 ≈ 0.309 yd³ × $30 ≈ $9.26
        //   pavers = 100 ft² × $6 = $600
        let bom = take_off(&plan_with_areas(
            vec![
                paver_on("gravel", 4.0, "sand", 1.0),
                material("gravel", "Gravel base", 50.0, PriceUnit::per_cubic_yard),
                material("sand", "Bedding sand", 30.0, PriceUnit::per_cubic_yard),
            ],
            vec![bed("paver", 10.0, 10.0, 0.0)],
            vec![],
        ));
        // Three itemized lines, in catalog order: pavers, gravel, sand.
        assert_eq!(bom.lines.len(), 3);
        assert_eq!(bom.lines[0].catalog_ref, "paver");
        assert!((bom.lines[0].quantity - 100.0).abs() < 1e-9);

        assert_eq!(bom.lines[1].catalog_ref, "gravel");
        let gravel_yd3 = 100.0 * 4.0 / 324.0;
        assert!(
            (bom.lines[1].quantity - gravel_yd3).abs() < 1e-9,
            "gravel yd³"
        );
        assert!((bom.lines[1].line_total - gravel_yd3 * 50.0).abs() < 1e-9);

        assert_eq!(bom.lines[2].catalog_ref, "sand");
        let sand_yd3 = 100.0 * 1.0 / 324.0;
        assert!((bom.lines[2].quantity - sand_yd3).abs() < 1e-9, "sand yd³");
        assert!((bom.lines[2].line_total - sand_yd3 * 30.0).abs() < 1e-9);
    }

    #[test]
    fn a_base_course_sums_direct_beds_and_paver_driven_volume() {
        // "gravel" is used both as a paver's base (100 ft² × 4 in) AND as its
        // own 6 in deep drainage bed (10×8 = 80 ft²). The gravel line is the
        // sum of both, so neither contribution may be dropped.
        let bom = take_off(&plan_with_areas(
            vec![
                paver_on("gravel", 4.0, "sand", 1.0),
                material("gravel", "Gravel", 50.0, PriceUnit::per_cubic_yard),
                material("sand", "Sand", 30.0, PriceUnit::per_cubic_yard),
            ],
            vec![bed("paver", 10.0, 10.0, 0.0), bed("gravel", 10.0, 8.0, 6.0)],
            vec![],
        ));
        let gravel = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "gravel")
            .unwrap();
        let expected = 100.0 * 4.0 / 324.0 + 80.0 * 6.0 / 324.0;
        assert!(
            (gravel.quantity - expected).abs() < 1e-9,
            "paver base {} + direct bed {} = {expected}, got {}",
            100.0 * 4.0 / 324.0,
            80.0 * 6.0 / 324.0,
            gravel.quantity
        );
    }

    #[test]
    fn a_plain_per_ft2_material_drives_no_sub_base() {
        // A paver with no base/bedding refs adds only its own ft² line — no
        // phantom gravel/sand volume.
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Pavers", 6.0, PriceUnit::per_square_foot),
                material("gravel", "Gravel", 50.0, PriceUnit::per_cubic_yard),
            ],
            vec![bed("paver", 10.0, 10.0, 0.0)],
            vec![],
        ));
        assert_eq!(bom.lines.len(), 1, "only the paver line");
        assert_eq!(bom.lines[0].catalog_ref, "paver");
    }

    #[test]
    fn a_round_paver_area_also_drives_its_sub_base() {
        // The base/bedding volume follows a circular paver area too.
        let circle = Circle {
            material_ref: Some("paver".to_string()),
            depth_in: Some(0.0),
            ..Circle::new(Box::new(Coord::new(20.0, 20.0)), 0.0, 5.0)
        };
        let bom = take_off(&plan_with_areas(
            vec![
                paver_on("gravel", 4.0, "sand", 1.0),
                material("gravel", "Gravel", 50.0, PriceUnit::per_cubic_yard),
                material("sand", "Sand", 30.0, PriceUnit::per_cubic_yard),
            ],
            vec![],
            vec![circle],
        ));
        let gravel = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "gravel")
            .unwrap();
        let expected = circle_area(5.0) * 4.0 / 324.0;
        assert!((gravel.quantity - expected).abs() < 1e-9);
    }

    // --- Per-area composition: an area's own `courses` (B3.1) ---

    /// A `w`×`d` paver patio whose sub-base is the given explicit `courses`.
    fn paver_with_courses(w: f64, d: f64, courses: Vec<Course>) -> Shape {
        let mut s = bed("paver", w, d, 0.0);
        s.courses = courses;
        s
    }

    #[test]
    fn default_courses_are_a_surface_materials_base_then_bedding() {
        let paver = paver_on("gravel", 4.0, "sand", 1.0);
        let courses = default_courses(&paver);
        assert_eq!(courses.len(), 2, "base + bedding");
        assert_eq!(courses[0].material_ref, "gravel");
        assert!((courses[0].depth_in - 4.0).abs() < 1e-9);
        assert_eq!(courses[1].material_ref, "sand");
        assert!((courses[1].depth_in - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_material_with_no_sub_base_has_no_default_courses() {
        // Mulch (no base/bedding refs) yields an empty course list.
        let mulch = material("mulch", "Mulch", 40.0, PriceUnit::per_cubic_yard);
        assert!(default_courses(&mulch).is_empty());
    }

    // --- Effective photo-tile size (material images) ---

    #[test]
    fn declared_tile_dimensions_are_used_as_is() {
        let mut paver = material("paver", "Pavers", 8.0, PriceUnit::per_square_foot);
        paver.tile_width_ft = Some(3.5);
        paver.tile_depth_ft = Some(1.25);
        assert_eq!(tile_size_ft(&paver), (3.5, 1.25));
    }

    #[test]
    fn absent_tile_dimensions_fall_back_to_the_default() {
        let paver = material("paver", "Pavers", 8.0, PriceUnit::per_square_foot);
        assert_eq!(tile_size_ft(&paver), (DEFAULT_TILE_FT, DEFAULT_TILE_FT));
    }

    #[test]
    fn a_zero_or_negative_tile_dimension_means_use_the_default() {
        // The schema reads 0 as "use the default"; negative is nonsense input
        // and gets the same treatment. Each axis falls back independently.
        let mut paver = material("paver", "Pavers", 8.0, PriceUnit::per_square_foot);
        paver.tile_width_ft = Some(0.0);
        paver.tile_depth_ft = Some(-1.0);
        assert_eq!(tile_size_ft(&paver), (DEFAULT_TILE_FT, DEFAULT_TILE_FT));

        // One declared, one zeroed — only the zeroed axis defaults.
        paver.tile_width_ft = Some(4.0);
        paver.tile_depth_ft = Some(0.0);
        assert_eq!(tile_size_ft(&paver), (4.0, DEFAULT_TILE_FT));
    }

    #[test]
    fn each_area_uses_its_own_courses_for_its_sub_base() {
        // Two patios on different gravels at different depths — the whole point
        // of per-area composition. A (100 ft²) on 6 in gravel-a; B (50 ft²) on
        // 4 in gravel-b: each gravel line is only its own patio's volume.
        let a = paver_with_courses(10.0, 10.0, vec![Course::new(6.0, "gravel-a".into())]);
        let b = paver_with_courses(5.0, 10.0, vec![Course::new(4.0, "gravel-b".into())]);
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Pavers", 6.0, PriceUnit::per_square_foot),
                material("gravel-a", "Gravel A", 50.0, PriceUnit::per_cubic_yard),
                material("gravel-b", "Gravel B", 60.0, PriceUnit::per_cubic_yard),
            ],
            vec![a, b],
            vec![],
        ));
        let ga = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "gravel-a")
            .unwrap();
        let gb = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "gravel-b")
            .unwrap();
        assert!(
            (ga.quantity - 100.0 * 6.0 / 324.0).abs() < 1e-9,
            "patio A's gravel"
        );
        assert!(
            (gb.quantity - 50.0 * 4.0 / 324.0).abs() < 1e-9,
            "patio B's gravel"
        );
    }

    #[test]
    fn an_areas_courses_override_the_catalog_template() {
        // The paver's catalog default is 4 in gravel + 1 in sand, but this area
        // declares its own single 6 in gravel course — the area's courses win,
        // and the template's sand (absent from the courses) is not costed.
        let area = paver_with_courses(10.0, 10.0, vec![Course::new(6.0, "gravel".into())]);
        let bom = take_off(&plan_with_areas(
            vec![
                paver_on("gravel", 4.0, "sand", 1.0),
                material("gravel", "Gravel", 50.0, PriceUnit::per_cubic_yard),
                material("sand", "Sand", 30.0, PriceUnit::per_cubic_yard),
            ],
            vec![area],
            vec![],
        ));
        let gravel = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "gravel")
            .unwrap();
        assert!(
            (gravel.quantity - 100.0 * 6.0 / 324.0).abs() < 1e-9,
            "the area's 6 in course, not the catalog's 4 in template"
        );
        assert!(
            bom.lines.iter().all(|l| l.catalog_ref != "sand"),
            "the template's bedding sand is ignored once the area sets its own courses"
        );
    }

    #[test]
    fn a_circle_area_with_courses_costs_them_too() {
        let mut circle = Circle {
            material_ref: Some("paver".to_string()),
            depth_in: Some(0.0),
            ..Circle::new(Box::new(Coord::new(20.0, 20.0)), 0.0, 5.0)
        };
        circle.courses = vec![Course::new(5.0, "gravel".into())];
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Pavers", 6.0, PriceUnit::per_square_foot),
                material("gravel", "Gravel", 50.0, PriceUnit::per_cubic_yard),
            ],
            vec![],
            vec![circle],
        ));
        let gravel = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "gravel")
            .unwrap();
        assert!((gravel.quantity - circle_area(5.0) * 5.0 / 324.0).abs() < 1e-9);
    }

    #[test]
    fn objects_and_area_materials_coexist_in_catalog_order() {
        let bom = take_off(&{
            let mut p = plan_with_areas(
                vec![
                    item("chair", "Chair", Some(50.0)),
                    material("mulch", "Mulch", 40.0, PriceUnit::per_cubic_yard),
                ],
                vec![bed("mulch", 10.0, 8.0, 3.0)],
                vec![],
            );
            p.objects = vec![placed("chair", ItemStatus::planned)];
            p
        });
        assert_eq!(bom.lines.len(), 2);
        assert_eq!(bom.lines[0].catalog_ref, "chair");
        assert_eq!(bom.lines[0].unit, PriceUnit::per_item);
        assert_eq!(bom.lines[1].catalog_ref, "mulch");
        assert_eq!(bom.lines[1].unit, PriceUnit::per_cubic_yard);
    }

    // --- Border rings (B5): border courses + edging stones ---

    use crate::generated::slp::Border;
    use std::f64::consts::TAU;

    /// A 10×10 paver patio with the given border rings.
    fn paver_with_borders(borders: Vec<Border>) -> Shape {
        let mut s = bed("paver", 10.0, 10.0, 0.0);
        s.borders = borders;
        s
    }

    #[test]
    fn a_border_ring_costs_its_band_and_shrinks_the_field() {
        // 10×10 patio (P = 40): one 0.5 ft border of a contrasting per-ft²
        // paver. Centerline = 40 − 2π·0.25; band = centerline × 0.5; the field
        // loses exactly that band.
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("cobble", "Border cobble", 9.0, PriceUnit::per_square_foot),
            ],
            vec![paver_with_borders(vec![Border::new("cobble".into(), 0.5)])],
            vec![],
        ));
        let centerline = 40.0 - TAU * 0.25;
        let band = centerline * 0.5;
        let field = bom.lines.iter().find(|l| l.catalog_ref == "paver").unwrap();
        let border = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "cobble")
            .unwrap();
        assert!(
            (field.quantity - (100.0 - band)).abs() < 1e-9,
            "field shrinks by the band: want {}, got {}",
            100.0 - band,
            field.quantity
        );
        assert_eq!(border.unit, PriceUnit::per_square_foot);
        assert!(
            (border.quantity - band).abs() < 1e-9,
            "band ft²: want {band}, got {}",
            border.quantity
        );
    }

    #[test]
    fn an_edging_stone_costs_by_centerline_linear_feet() {
        // The same patio edged with a per-linear-ft edging stone, 1/3 ft wide.
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("edging", "Edging stones", 4.0, PriceUnit::per_linear_foot),
            ],
            vec![paver_with_borders(vec![Border::new(
                "edging".into(),
                1.0 / 3.0,
            )])],
            vec![],
        ));
        let centerline = 40.0 - TAU * (1.0 / 6.0);
        let edging = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "edging")
            .unwrap();
        assert_eq!(edging.unit, PriceUnit::per_linear_foot);
        assert!(
            (edging.quantity - centerline).abs() < 1e-9,
            "want {centerline}, got {}",
            edging.quantity
        );
        assert!((edging.line_total - centerline * 4.0).abs() < 1e-9);
        // The per-linear-ft ring still shrinks the field by its band area.
        let field = bom.lines.iter().find(|l| l.catalog_ref == "paver").unwrap();
        let band = centerline * (1.0 / 3.0);
        assert!((field.quantity - (100.0 - band)).abs() < 1e-9);
    }

    #[test]
    fn inner_rings_measure_against_the_shrunken_boundary_in_order() {
        // Outer 0.5 ft cobble, inner 0.25 ft edging: the inner ring's
        // centerline offset is 0.5 + 0.125.
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("cobble", "Border cobble", 9.0, PriceUnit::per_square_foot),
                material("edging", "Edging stones", 4.0, PriceUnit::per_linear_foot),
            ],
            vec![paver_with_borders(vec![
                Border::new("cobble".into(), 0.5),
                Border::new("edging".into(), 0.25),
            ])],
            vec![],
        ));
        let outer_center = 40.0 - TAU * 0.25;
        let inner_center = 40.0 - TAU * (0.5 + 0.125);
        let cobble = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "cobble")
            .unwrap();
        let edging = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "edging")
            .unwrap();
        assert!(
            (cobble.quantity - outer_center * 0.5).abs() < 1e-9,
            "outer band"
        );
        assert!(
            (edging.quantity - inner_center).abs() < 1e-9,
            "inner centerline"
        );
        let field = bom.lines.iter().find(|l| l.catalog_ref == "paver").unwrap();
        let want = 100.0 - outer_center * 0.5 - inner_center * 0.25;
        assert!(
            (field.quantity - want).abs() < 1e-9,
            "field minus both bands"
        );
    }

    #[test]
    fn a_circle_border_is_exact() {
        // A radius-5 circular patio (P = 2π·5) with a 0.5 ft border: the
        // rounded-offset model is exact for a circle — centerline = 2π·4.75.
        let mut circle = Circle {
            material_ref: Some("paver".to_string()),
            depth_in: Some(0.0),
            ..Circle::new(Box::new(Coord::new(20.0, 20.0)), 0.0, 5.0)
        };
        circle.borders = vec![Border::new("cobble".into(), 0.5)];
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("cobble", "Border cobble", 9.0, PriceUnit::per_square_foot),
            ],
            vec![],
            vec![circle],
        ));
        let centerline = TAU * 4.75;
        let cobble = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "cobble")
            .unwrap();
        assert!((cobble.quantity - centerline * 0.5).abs() < 1e-9);
        // Exactness check: band area = π(5² − 4.5²) = centerline × 0.5.
        let exact = std::f64::consts::PI * (25.0 - 4.5 * 4.5);
        assert!((cobble.quantity - exact).abs() < 1e-9, "exact annulus area");
    }

    #[test]
    fn oversized_borders_clamp_and_never_go_negative() {
        // Absurd 40 ft-wide border on a 10×10 patio: centerline clamps to 0,
        // the ring measures nothing, and the field never goes below 0.
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("cobble", "Border cobble", 9.0, PriceUnit::per_square_foot),
            ],
            vec![paver_with_borders(vec![Border::new("cobble".into(), 40.0)])],
            vec![],
        ));
        assert!(
            bom.lines.iter().all(|l| l.catalog_ref != "cobble"),
            "a fully-consumed ring measures nothing"
        );
        let field = bom.lines.iter().find(|l| l.catalog_ref == "paver").unwrap();
        assert!(field.quantity > 0.0 && field.quantity <= 100.0);
    }

    #[test]
    fn borders_leave_the_sub_base_at_full_area() {
        // The gravel base runs beneath the border too: a bordered 10×10 paver
        // patio still drives 100 ft² × 4 in of gravel.
        let mut area = paver_with_borders(vec![Border::new("cobble".into(), 0.5)]);
        area.courses = vec![Course::new(4.0, "gravel".into())];
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("cobble", "Border cobble", 9.0, PriceUnit::per_square_foot),
                material("gravel", "Gravel", 50.0, PriceUnit::per_cubic_yard),
            ],
            vec![area],
            vec![],
        ));
        let gravel = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "gravel")
            .unwrap();
        assert!(
            (gravel.quantity - 100.0 * 4.0 / 324.0).abs() < 1e-9,
            "full area, not field-only"
        );
    }

    #[test]
    fn a_curved_areas_take_off_follows_its_curve() {
        // A 10×10 paver patio whose south edge bows outward as a Bézier: its
        // costed ft² must match the curve-aware boundary area (more than the
        // straight 100), not the straight-chord polygon.
        use crate::generated::slp::CurveEdge;
        let mut s = bed("paver", 10.0, 10.0, 0.0);
        s.curves = vec![CurveEdge::new(
            Box::new(Coord::new(2.5, -4.0)),
            Box::new(Coord::new(7.5, -4.0)),
            0,
        )];
        let expected = {
            let curves = vec![(
                0usize,
                crate::Point::new(2.5, -4.0),
                crate::Point::new(7.5, -4.0),
            )];
            crate::boundary_area(&s.corners, &s.bulges, &curves)
        };
        assert!(expected > 100.5, "the bow adds area: {expected}");
        let bom = take_off(&plan_with_areas(
            vec![material("paver", "Pavers", 6.0, PriceUnit::per_square_foot)],
            vec![s],
            vec![],
        ));
        assert!(
            (bom.lines[0].quantity - expected).abs() < 1e-9,
            "want {expected}, got {}",
            bom.lines[0].quantity
        );
    }

    #[test]
    fn a_span_border_costs_only_its_edges_with_no_corner_shrink() {
        // 10×10 patio (P = 40); a 0.5 ft edging span from node 0 to node 2
        // covers edges 0 and 1 (10 + 10 = 20 lf) — an open band, so no 2π
        // corner shrink — and the field loses exactly 20 × 0.5 ft².
        let mut area = paver_with_borders(vec![]);
        let mut b = Border::new("edging".into(), 0.5);
        b.start_node = Some(0.0);
        b.end_node = Some(2.0);
        area.borders = vec![b];
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("edging", "Edging stones", 4.0, PriceUnit::per_linear_foot),
            ],
            vec![area],
            vec![],
        ));
        let edging = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "edging")
            .unwrap();
        assert!(
            (edging.quantity - 20.0).abs() < 1e-9,
            "span edges only: got {}",
            edging.quantity
        );
        let field = bom.lines.iter().find(|l| l.catalog_ref == "paver").unwrap();
        assert!(
            (field.quantity - (100.0 - 10.0)).abs() < 1e-9,
            "field minus the band"
        );
    }

    #[test]
    fn a_half_specified_or_bad_span_falls_back_predictably() {
        // Only one node set → per the schema, still a full ring.
        let mut ring = Border::new("cobble".into(), 0.5);
        ring.start_node = Some(1.0);
        // Both set but out of range → a dead span: measures nothing rather
        // than silently billing the whole perimeter.
        let mut dead = Border::new("edging".into(), 0.5);
        dead.start_node = Some(0.0);
        dead.end_node = Some(9.0);
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("cobble", "Border cobble", 9.0, PriceUnit::per_square_foot),
                material("edging", "Edging stones", 4.0, PriceUnit::per_linear_foot),
            ],
            vec![paver_with_borders(vec![ring, dead])],
            vec![],
        ));
        let cobble = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "cobble")
            .unwrap();
        let want_ring = (40.0 - TAU * 0.25) * 0.5;
        assert!((cobble.quantity - want_ring).abs() < 1e-9, "a full ring");
        assert!(
            bom.lines.iter().all(|l| l.catalog_ref != "edging"),
            "a dead span measures nothing"
        );
    }

    #[test]
    fn a_circles_border_ignores_span_nodes() {
        // A circle has no nodes: even a span-tagged border rings the whole
        // circumference.
        let mut circle = Circle {
            material_ref: Some("paver".to_string()),
            depth_in: Some(0.0),
            ..Circle::new(Box::new(Coord::new(20.0, 20.0)), 0.0, 5.0)
        };
        let mut b = Border::new("edging".into(), 0.5);
        b.start_node = Some(0.0);
        b.end_node = Some(2.0);
        circle.borders = vec![b];
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("edging", "Edging stones", 4.0, PriceUnit::per_linear_foot),
            ],
            vec![],
            vec![circle],
        ));
        let edging = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "edging")
            .unwrap();
        assert!(
            (edging.quantity - TAU * 4.75).abs() < 1e-9,
            "the full ring centerline"
        );
    }

    #[test]
    fn a_mid_edge_span_costs_its_partial_edges() {
        // 10×10 patio: an edging span from position 0.5 (midpoint of edge 0)
        // to 1.5 (midpoint of edge 1) = half edge0 (5) + half edge1 (5) = 10
        // linear ft.
        let mut area = paver_with_borders(vec![]);
        let mut b = Border::new("edging".into(), 0.5);
        b.start_node = Some(0.5);
        b.end_node = Some(1.5);
        area.borders = vec![b];
        let bom = take_off(&plan_with_areas(
            vec![
                material("paver", "Field pavers", 6.0, PriceUnit::per_square_foot),
                material("edging", "Edging stones", 4.0, PriceUnit::per_linear_foot),
            ],
            vec![area],
            vec![],
        ));
        let edging = bom
            .lines
            .iter()
            .find(|l| l.catalog_ref == "edging")
            .unwrap();
        assert!(
            (edging.quantity - 10.0).abs() < 1e-9,
            "partial edges: got {}",
            edging.quantity
        );
    }
}

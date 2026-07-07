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

use crate::generated::slp::{ItemStatus, Plan, PriceUnit};
use crate::{Point, Shape, boundary_area, circle_area};

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
/// `Shape`/`Circle` whose `material_ref` names it. Lines come out in catalog
/// order; an item with no counted quantity yields no line. A catalog item with
/// no `unit_price` is priced at `0.0`. (Per-linear-ft materials aren't costed
/// yet — they yield no line.)
#[must_use]
pub fn take_off(plan: &Plan) -> BillOfMaterials {
    let mut lines = Vec::new();
    let mut grand_total = 0.0;
    for item in &plan.catalog {
        let quantity = match item.price_unit {
            PriceUnit::per_item => object_count(plan, &item.id),
            PriceUnit::per_square_foot => material_area(plan, &item.id),
            PriceUnit::per_cubic_yard => material_volume(plan, &item.id),
            PriceUnit::per_linear_foot => 0.0,
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
        });
    }
    BillOfMaterials { lines, grand_total }
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

/// The total surface area (ft²) of every drawn `Shape`/`Circle` made of
/// material `id`.
fn material_area(plan: &Plan, id: &str) -> f64 {
    let shapes: f64 = plan
        .shapes
        .iter()
        .filter(|s| s.material_ref.as_deref() == Some(id))
        .map(shape_area)
        .sum();
    let circles: f64 = plan
        .circles
        .iter()
        .filter(|c| c.material_ref.as_deref() == Some(id))
        .map(|c| circle_area(c.radius_ft))
        .sum();
    shapes + circles
}

/// The total volume (yd³) of every drawn `Shape`/`Circle` made of material
/// `id`, each at its own depth — `yd³ = ft²·depth_in / 324`.
fn material_volume(plan: &Plan, id: &str) -> f64 {
    let shapes: f64 = plan
        .shapes
        .iter()
        .filter(|s| s.material_ref.as_deref() == Some(id))
        .map(|s| volume_yd3(shape_area(s), s.depth_in.unwrap_or(0.0)))
        .sum();
    let circles: f64 = plan
        .circles
        .iter()
        .filter(|c| c.material_ref.as_deref() == Some(id))
        .map(|c| volume_yd3(circle_area(c.radius_ft), c.depth_in.unwrap_or(0.0)))
        .sum();
    shapes + circles
}

/// A shape's enclosed area (ft²), accounting for any arc/curve edges.
fn shape_area(s: &Shape) -> f64 {
    let curves: Vec<(usize, Point, Point)> = s
        .curves
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
    boundary_area(&s.corners, &s.bulges, &curves)
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
}

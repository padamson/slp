//! Cost take-off: turn a plan's placed objects into a priced bill of materials.
//!
//! The take-off answers "what do I buy, and what does it cost?" — it counts only
//! **planned** placements (`existing` and `virtual` objects appear on the plan
//! but are never purchased) and prices each by its catalog item.

use crate::generated::slp::{ItemStatus, Plan};

/// One line of the bill of materials: a catalog item, how many *planned*
/// placements reference it, and the dollars those placements add up to.
#[derive(Debug, Clone, PartialEq)]
pub struct LineItem {
    /// The catalog item's `id` — the key objects reference via `catalog_ref`.
    pub catalog_ref: String,
    /// The catalog item's display name, if it has one.
    pub name: Option<String>,
    /// Number of planned objects placing this item.
    pub qty: u32,
    /// Price per item, in dollars; `0.0` when the catalog item has no price.
    pub unit_price: f64,
    /// `qty × unit_price`, in dollars.
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
/// Counts only **planned** placements: `existing` and `virtual` objects are
/// shown on the plan but never bought, so they are excluded. Objects whose
/// `catalog_ref` matches no catalog item are excluded too (you can't price what
/// isn't in the catalog). Lines come out in catalog order; a catalog item with
/// no planned placements yields no line. A catalog item with no `unit_price` is
/// priced at `0.0`.
#[must_use]
pub fn take_off(plan: &Plan) -> BillOfMaterials {
    let mut lines = Vec::new();
    let mut grand_total = 0.0;
    for item in &plan.catalog {
        let mut qty: u32 = 0;
        for object in &plan.objects {
            if object.status == ItemStatus::planned && object.catalog_ref == item.id {
                qty += 1;
            }
        }
        if qty == 0 {
            continue;
        }
        let unit_price = item.unit_price.unwrap_or(0.0);
        let line_total = f64::from(qty) * unit_price;
        grand_total += line_total;
        lines.push(LineItem {
            catalog_ref: item.id.clone(),
            name: item.name.clone(),
            qty,
            unit_price,
            line_total,
        });
    }
    BillOfMaterials { lines, grand_total }
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
        assert_eq!(bom.lines[0].qty, 2);
        assert!((bom.lines[0].line_total - 99.0).abs() < 1e-9);
        assert_eq!(bom.lines[1].catalog_ref, "table");
        assert_eq!(bom.lines[1].qty, 1);
        assert!((bom.lines[1].line_total - 200.0).abs() < 1e-9);
        assert!((bom.grand_total - 299.0).abs() < 1e-9);
    }

    #[test]
    fn existing_and_virtual_placements_are_not_counted() {
        // Same item placed once each as planned / existing / virtual: only the
        // planned one is bought.
        let bom = take_off(&plan(
            vec![item("chair", "Patio chair", Some(50.0))],
            vec![
                placed("chair", ItemStatus::planned),
                placed("chair", ItemStatus::existing),
                placed("chair", ItemStatus::r#virtual),
            ],
        ));
        assert_eq!(bom.lines.len(), 1);
        assert_eq!(bom.lines[0].qty, 1);
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
        assert_eq!(bom.lines[0].qty, 1);
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
        assert_eq!(bom.lines[0].qty, 1);
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
        assert_eq!(bom.lines[0].qty, 2);
        assert!(bom.lines[0].unit_price.abs() < 1e-9);
        assert!(bom.lines[0].line_total.abs() < 1e-9);
        assert!(bom.grand_total.abs() < 1e-9);
    }
}

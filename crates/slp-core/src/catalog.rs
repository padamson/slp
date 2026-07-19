//! Catalog reference counting: how many places in a plan point at a catalog
//! item. Deleting a still-referenced item would leave dangling refs — placed
//! objects that silently stop costing, areas that lose their texture and
//! course composition, pavers whose default base/bedding layer vanishes — so
//! the catalog editor blocks deletion while this count is non-zero.

use crate::generated::slp::Plan;

/// How many references the plan holds to catalog item `id`: placed objects
/// (`catalog_ref`), drawn areas (`material_ref` on shapes and circles), every
/// course in an area's composition, every border ring around an area, and
/// other catalog items whose default base/bedding layer points at it.
#[must_use]
pub fn reference_count(plan: &Plan, id: &str) -> usize {
    let objects = plan.objects.iter().filter(|o| o.catalog_ref == id).count();
    let areas = plan
        .shapes
        .iter()
        .map(|s| (&s.material_ref, &s.courses, &s.borders))
        .chain(
            plan.circles
                .iter()
                .map(|c| (&c.material_ref, &c.courses, &c.borders)),
        )
        .map(|(material_ref, courses, borders)| {
            usize::from(material_ref.as_deref() == Some(id))
                + courses.iter().filter(|k| k.material_ref == id).count()
                + borders.iter().filter(|b| b.material_ref == id).count()
        })
        .sum::<usize>();
    let compositions = plan
        .catalog
        .iter()
        .filter(|c| c.id != id)
        .map(|c| {
            usize::from(c.base_material_ref.as_deref() == Some(id))
                + usize::from(c.bedding_material_ref.as_deref() == Some(id))
        })
        .sum::<usize>();
    objects + areas + compositions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::slp::{CatalogItem, Circle, Coord, Course, Object, Shape};

    fn plan() -> Plan {
        Plan {
            catalog: ["paver", "gravel", "chair"]
                .map(|id| CatalogItem::new(id.to_string()))
                .to_vec(),
            ..Plan::new(40.0, 60.0)
        }
    }

    #[test]
    fn an_unreferenced_item_counts_zero() {
        assert_eq!(reference_count(&plan(), "chair"), 0);
    }

    #[test]
    fn placed_objects_count_by_catalog_ref() {
        let mut p = plan();
        p.objects.push(Object::new("chair".to_string(), 1.0, 1.0));
        p.objects.push(Object::new("chair".to_string(), 5.0, 5.0));
        assert_eq!(reference_count(&p, "chair"), 2);
        assert_eq!(reference_count(&p, "paver"), 0);
    }

    #[test]
    fn shape_and_circle_areas_count_by_material_ref() {
        let mut p = plan();
        let mut s = Shape::new(0.0);
        s.material_ref = Some("paver".to_string());
        p.shapes.push(s);
        let mut c = Circle::new(Box::new(Coord::new(10.0, 10.0)), 0.0, 3.0);
        c.material_ref = Some("paver".to_string());
        p.circles.push(c);
        assert_eq!(reference_count(&p, "paver"), 2);
    }

    #[test]
    fn area_courses_count_each_layer() {
        let mut p = plan();
        let mut s = Shape::new(0.0);
        s.material_ref = Some("paver".to_string());
        s.courses.push(Course::new(4.0, "gravel".to_string()));
        s.courses.push(Course::new(2.0, "gravel".to_string()));
        p.shapes.push(s);
        assert_eq!(reference_count(&p, "gravel"), 2);
        assert_eq!(reference_count(&p, "paver"), 1);
    }

    #[test]
    fn another_items_base_or_bedding_layer_counts() {
        let mut p = plan();
        p.catalog[0].base_material_ref = Some("gravel".to_string());
        p.catalog[0].bedding_material_ref = Some("gravel".to_string());
        assert_eq!(reference_count(&p, "gravel"), 2);
        // An item's own refs never count against itself.
        p.catalog[1].base_material_ref = Some("gravel".to_string());
        assert_eq!(reference_count(&p, "gravel"), 2);
    }

    #[test]
    fn area_border_rings_count_each_ring() {
        let mut p = plan();
        let mut s = Shape::new(0.0);
        s.material_ref = Some("paver".to_string());
        s.borders
            .push(crate::Border::new("gravel".to_string(), 0.5));
        s.borders
            .push(crate::Border::new("gravel".to_string(), 0.25));
        p.shapes.push(s);
        let mut c = Circle::new(Box::new(Coord::new(10.0, 10.0)), 0.0, 3.0);
        c.borders
            .push(crate::Border::new("gravel".to_string(), 0.5));
        p.circles.push(c);
        assert_eq!(reference_count(&p, "gravel"), 3);
    }
}

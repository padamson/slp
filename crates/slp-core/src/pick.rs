//! Picking a placed object under a point — hit-testing footprints so the UI can
//! select an object to toggle its status or rotate it.

use crate::{CatalogItem, Object, Point, footprint_corners, point_in_polygon};

/// Fallback footprint side (ft) when a catalog item carries no dimensions — must
/// match the render so what you click is what you see.
const DEFAULT_FT: f64 = 1.0;

/// The index of the topmost object whose footprint contains `p`, if any. Later
/// objects paint on top, so they are tested first (reverse order). An object
/// whose `catalog_ref` resolves to no catalog item has no footprint and is
/// skipped.
#[must_use]
pub fn object_at(p: Point, objects: &[Object], catalog: &[CatalogItem]) -> Option<usize> {
    objects.iter().enumerate().rev().find_map(|(i, obj)| {
        let item = catalog.iter().find(|c| c.id == obj.catalog_ref)?;
        let w = item.width_ft.unwrap_or(DEFAULT_FT);
        let d = item.depth_ft.unwrap_or(DEFAULT_FT);
        let corners = footprint_corners(obj.x, obj.y, w, d, obj.rot.unwrap_or(0.0));
        point_in_polygon(p, &corners).then_some(i)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, w: f64, d: f64) -> CatalogItem {
        let mut c = CatalogItem::new(id.to_string());
        c.width_ft = Some(w);
        c.depth_ft = Some(d);
        c
    }

    fn placed(catalog_ref: &str, x: f64, y: f64) -> Object {
        Object::new(catalog_ref.to_string(), x, y)
    }

    #[test]
    fn picks_the_object_under_the_point() {
        let catalog = vec![item("chair", 2.0, 2.0)];
        let objects = vec![placed("chair", 5.0, 5.0)];
        // Inside the 2×2 footprint centered at (5,5).
        assert_eq!(object_at(Point::new(5.5, 4.5), &objects, &catalog), Some(0));
        // Outside it.
        assert_eq!(object_at(Point::new(9.0, 9.0), &objects, &catalog), None);
    }

    #[test]
    fn picks_the_topmost_of_overlapping_objects() {
        let catalog = vec![item("chair", 3.0, 3.0)];
        // Two chairs overlapping at (5,5); the later one is drawn on top.
        let objects = vec![placed("chair", 5.0, 5.0), placed("chair", 5.5, 5.5)];
        assert_eq!(object_at(Point::new(5.2, 5.2), &objects, &catalog), Some(1));
    }

    #[test]
    fn respects_rotation() {
        // A 4×1 ft bar at the origin rotated 90° runs north-south, so a point 1.5 ft
        // north is inside it — but would be outside the un-rotated (east-west) bar.
        let catalog = vec![item("bar", 4.0, 1.0)];
        let mut obj = placed("bar", 0.0, 0.0);
        obj.rot = Some(90.0);
        let objects = vec![obj];
        assert_eq!(object_at(Point::new(0.0, 1.5), &objects, &catalog), Some(0));
        assert_eq!(object_at(Point::new(1.5, 0.0), &objects, &catalog), None);
    }

    #[test]
    fn an_unresolved_catalog_ref_is_not_pickable() {
        let catalog = vec![item("chair", 2.0, 2.0)];
        let objects = vec![placed("ghost", 5.0, 5.0)];
        assert_eq!(object_at(Point::new(5.0, 5.0), &objects, &catalog), None);
    }

    #[test]
    fn nothing_placed_picks_nothing() {
        assert_eq!(object_at(Point::new(5.0, 5.0), &[], &[]), None);
    }
}

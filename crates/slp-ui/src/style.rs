//! Shared visual styling for plan entities — the single source of truth for
//! the colors, fill treatment, and outline each entity draws with, so the
//! canvas (`Furnishings`, `House`, `Deck`) and the `Legend` render identically.
//! Changing a look — e.g. the virtual-status dash pattern — means editing one
//! constant here, not hunting through every place that draws it.
//!
//! Two corner conventions read at a glance: a placed footprint (furniture) has
//! plain **square** rect corners; a user-drawn outline (house, deck) carries a
//! small **node** (circle) marker at each vertex, since those corners are
//! individually draggable points, not just a shape's edges.

use slp_core::ItemStatus;

/// House walls + corner markers: solid outline, node corners.
pub const HOUSE_FILL: &str = "#d8d2c4";
pub const HOUSE_FILL_OPACITY: &str = "0.6";
pub const HOUSE_STROKE: &str = "#8a7f6a";

/// Deck levels + corner markers: solid outline, node corners.
pub const DECK_FILL: &str = "#c8a97e";
pub const DECK_FILL_OPACITY: &str = "0.55";
pub const DECK_STROKE: &str = "#8a6f4f";

/// Furniture footprints' base palette — square corners, no corner markers.
/// Status (below), selection, and overflow are independent modifiers layered
/// on top of this.
pub const FURNITURE_FILL: &str = "#a8927a";
pub const FURNITURE_STROKE: &str = "#5a4a3a";

/// A selected object's tint.
pub const SELECTED_FILL: &str = "#7ea9d4";
pub const SELECTED_STROKE: &str = "#2b6cb0";

/// An object that doesn't fit its surface — the loudest signal; wins over
/// both selection and status.
pub const OVERFLOW_STROKE: &str = "#d4351c";

/// A placed object's look, driven by its cost `status`: an extra CSS class
/// (a query hook), the outline's dash pattern, and its fill opacity.
/// `Furnishings` and `Legend` both call this, so they can't drift apart.
pub struct FurnitureStyle {
    /// Extra class for query hooks (empty for the default, `planned`).
    pub class: &'static str,
    /// SVG `stroke-dasharray` (`"none"` = solid).
    pub dash: &'static str,
    /// SVG `fill-opacity`.
    pub fill_opacity: &'static str,
}

#[must_use]
pub fn furniture_style(status: &ItemStatus) -> FurnitureStyle {
    match status {
        ItemStatus::existing => FurnitureStyle {
            class: " furniture-item--existing",
            dash: "6,3",
            fill_opacity: "0.55",
        },
        ItemStatus::r#virtual => FurnitureStyle {
            class: " furniture-item--virtual",
            dash: "3,3",
            fill_opacity: "0.3",
        },
        _ => FurnitureStyle {
            class: "",
            dash: "none",
            fill_opacity: "0.7",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planned_is_the_solid_default() {
        let s = furniture_style(&ItemStatus::planned);
        assert_eq!(s.class, "");
        assert_eq!(s.dash, "none");
        assert_eq!(s.fill_opacity, "0.7");
    }

    #[test]
    fn existing_is_dashed_and_dimmed() {
        let s = furniture_style(&ItemStatus::existing);
        assert_eq!(s.class, " furniture-item--existing");
        assert_eq!(s.dash, "6,3");
        assert_eq!(s.fill_opacity, "0.55");
    }

    #[test]
    fn virtual_is_a_lighter_ghost_than_existing() {
        let s = furniture_style(&ItemStatus::r#virtual);
        assert_eq!(s.class, " furniture-item--virtual");
        assert_eq!(s.dash, "3,3");
        assert_eq!(s.fill_opacity, "0.3");
    }
}

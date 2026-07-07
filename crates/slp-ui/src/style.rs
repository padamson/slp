//! Shared visual styling for plan entities — the single source of truth for
//! the colors, fill treatment, and outline each entity draws with, so the
//! canvas (`Furnishings`, `House`, `Deck`) and the `Legend` render identically.
//! Changing a look — e.g. the virtual dash pattern — means editing one
//! constant here, not hunting through every place that draws it.
//!
//! Two independent visual channels read at a glance for a placed object:
//! - **Line count** (`status`): a single outline is **planned** (to buy), a
//!   double outline (two nested strokes) is **existing** (already owned).
//! - **Line style** (`is_virtual`): solid is **real**, dashed is **virtual** —
//!   a what-if ghost duplicate, never a second real item.
//!
//! A third, separate convention distinguishes *what kind of thing* something
//! is: a placed footprint (furniture) has plain **square** rect corners; a
//! user-drawn outline (house, deck) carries a small **node** (circle) marker
//! at each vertex, since those corners are individually draggable points.
//! Structures are always real (no virtual variant) and, for now, always
//! render as a single outline — `structure_status` is carried in the schema
//! but double/single structure rendering is a follow-up slice (it needs a
//! polygon inset, not just a nested rect).

use slp_core::ItemStatus;

/// House walls + corner markers: solid outline, node corners.
pub const HOUSE_FILL: &str = "#d8d2c4";
pub const HOUSE_FILL_OPACITY: &str = "0.6";
pub const HOUSE_STROKE: &str = "#8a7f6a";

/// Deck levels + corner markers: solid outline, node corners.
pub const DECK_FILL: &str = "#c8a97e";
pub const DECK_FILL_OPACITY: &str = "0.55";
pub const DECK_STROKE: &str = "#8a6f4f";

/// A drawn area's (paver, mulch bed, …) generic look, node corners — the
/// category-specific fill (F3 is category-agnostic) lands with whichever
/// story first needs it (e.g. B1/B4).
pub const SHAPE_FILL: &str = "#c4c0a8";
pub const SHAPE_FILL_OPACITY: &str = "0.5";
pub const SHAPE_STROKE: &str = "#8a8568";

/// A mulch bed's fill/stroke — a dark bark brown, distinct from the neutral
/// default area look, so a mulch bed reads at a glance.
pub const MULCH_FILL: &str = "#6b4a2f";
pub const MULCH_STROKE: &str = "#4a3220";

/// Furniture footprints' base palette — square corners, no corner markers.
/// Status/virtual (below), selection, and overflow are independent modifiers
/// layered on top of this.
pub const FURNITURE_FILL: &str = "#a8927a";
pub const FURNITURE_STROKE: &str = "#5a4a3a";

/// A selected object's tint.
pub const SELECTED_FILL: &str = "#7ea9d4";
pub const SELECTED_STROKE: &str = "#2b6cb0";

/// An object that doesn't fit its surface, or sits somewhere its category
/// can't (a tree's trunk on hardscape, a fire pit on the house) — the loudest
/// signal; wins over both selection and status.
pub const OVERFLOW_STROKE: &str = "#d4351c";

/// A safety clearance ring (e.g. a fire pit's keep-clear zone) when nothing
/// intrudes on it — a quiet, dashed reminder.
pub const CLEARANCE_STROKE: &str = "#8a8275";
/// The ring's stroke width — thinner than an object's own outline, since it's
/// a secondary hint riding around the footprint, not the footprint itself.
pub const CLEARANCE_STROKE_W: &str = "1";
/// When something *does* intrude (another object's footprint or a structure
/// edge), the ring switches to this — a darker red than [`OVERFLOW_STROKE`],
/// so "something's inside the keep-clear zone" reads as its own signal rather
/// than looking identical to "this object doesn't fit."
pub const CLEARANCE_INTRUDE_STROKE: &str = "#7a1216";

/// A tree's canopy: a light, translucent green disk (a trunk renders inside it
/// in [`TRUNK_FILL`]).
pub const CANOPY_FILL: &str = "#a8d5a0";
pub const CANOPY_FILL_OPACITY: &str = "0.35";
pub const CANOPY_STROKE: &str = "#6f9c64";
/// A tree's trunk: a small, dark-brown disk at the canopy's center.
pub const TRUNK_FILL: &str = "#5a3a22";
pub const TRUNK_STROKE: &str = "#3a2415";

/// A fire pit's footprint — a metal fill instead of the shared furniture
/// brown.
pub const FIRE_PIT_FILL: &str = "#b8b8bc";

/// Group opacity for the placement preview ghost — faint, so it reads as "not
/// committed yet" without needing its own status/virtual styling (the armed
/// item's eventual look isn't known until the modifiers held at the click).
pub const PREVIEW_OPACITY: &str = "0.5";

/// Gap (px) between a double outline's two nested strokes — small, so the pair
/// reads as one closely-spaced "double rule".
pub const DOUBLE_LINE_GAP_PX: f64 = 1.6;
/// Stroke width (px) for a double outline's two lines — thinner than a single
/// outline, so two closely-spaced lines don't add up to a heavy border.
pub const DOUBLE_LINE_STROKE_W: &str = "0.9";

/// A placed object's look, driven by `status` (single vs. double outline) and
/// `is_virtual` (solid vs. dashed). `Furnishings` and `Legend` both call this,
/// so they can't drift apart.
pub struct FurnitureStyle {
    /// Extra classes for query hooks.
    pub class: &'static str,
    /// SVG `stroke-dasharray` (`"none"` = solid).
    pub dash: &'static str,
    /// SVG `fill-opacity`.
    pub fill_opacity: &'static str,
    /// A double (not single) outline — rendered as two nested strokes, inset
    /// by [`DOUBLE_LINE_GAP_PX`], since it's already owned (`existing`).
    pub double: bool,
}

/// The `(fill, stroke)` a drawn area renders with, by its material category: a
/// mulch bed reads as bark brown, every other/uncategorized area as the
/// neutral default. More categories (pavers, gravel) join as their stories land.
#[must_use]
pub fn area_style(category: Option<&str>) -> (&'static str, &'static str) {
    match category {
        Some("mulch-bed") => (MULCH_FILL, MULCH_STROKE),
        _ => (SHAPE_FILL, SHAPE_STROKE),
    }
}

#[must_use]
pub fn furniture_style(status: &ItemStatus, is_virtual: bool) -> FurnitureStyle {
    let double = *status == ItemStatus::existing;
    let (dash, fill_opacity) = if is_virtual {
        ("4,3", "0.35")
    } else {
        ("none", "0.7")
    };
    let class = match (double, is_virtual) {
        (false, false) => " furniture-item--planned",
        (false, true) => " furniture-item--planned furniture-item--virtual",
        (true, false) => " furniture-item--existing",
        (true, true) => " furniture-item--existing furniture-item--virtual",
    };
    FurnitureStyle {
        class,
        dash,
        fill_opacity,
        double,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planned_real_is_a_single_solid_full_color_outline() {
        let s = furniture_style(&ItemStatus::planned, false);
        assert_eq!(s.class, " furniture-item--planned");
        assert_eq!(s.dash, "none");
        assert_eq!(s.fill_opacity, "0.7");
        assert!(!s.double);
    }

    #[test]
    fn existing_real_is_a_double_solid_full_color_outline() {
        let s = furniture_style(&ItemStatus::existing, false);
        assert_eq!(s.class, " furniture-item--existing");
        assert_eq!(s.dash, "none");
        assert_eq!(s.fill_opacity, "0.7");
        assert!(s.double);
    }

    #[test]
    fn planned_virtual_is_a_single_dashed_ghost() {
        let s = furniture_style(&ItemStatus::planned, true);
        assert_eq!(s.class, " furniture-item--planned furniture-item--virtual");
        assert_eq!(s.dash, "4,3");
        assert_eq!(s.fill_opacity, "0.35");
        assert!(!s.double);
    }

    #[test]
    fn existing_virtual_is_a_double_dashed_ghost() {
        let s = furniture_style(&ItemStatus::existing, true);
        assert_eq!(s.class, " furniture-item--existing furniture-item--virtual");
        assert_eq!(s.dash, "4,3");
        assert_eq!(s.fill_opacity, "0.35");
        assert!(s.double);
    }
}

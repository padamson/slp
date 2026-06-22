//! slp-ui components, colocated Storybook-style (same convention as theoria):
//! `<name>.rs` + `<name>.stories.rs` (`stories` feature) + `<name>.tests.rs`
//! (`cfg(test)`). Dotted file names use `#[path]` (not valid module idents).

mod grid;
mod planner;
mod scale_bar;
mod yard;
mod yard_controls;

pub use grid::Grid;
pub use planner::Planner;
pub use scale_bar::ScaleBar;
pub use yard::Yard;
pub use yard_controls::YardControls;

/// World→screen transform: feet to SVG pixels. The yard's south-west corner is
/// the world origin; `+x` runs east, `+y` runs north (drawn upward, so screen-y
/// is flipped).
#[derive(Clone, Copy)]
pub struct Transform {
    /// Pixels per foot.
    pub px_ft: f64,
    /// Padding around the yard, in pixels.
    pub pad: f64,
    /// Yard depth in feet, used to flip the y axis.
    pub yard_d: f64,
}

impl Transform {
    /// World-x (feet) → screen-x (px).
    #[must_use]
    pub fn sx(self, x: f64) -> f64 {
        self.pad + x * self.px_ft
    }

    /// World-y (feet) → screen-y (px), north drawn upward.
    #[must_use]
    pub fn sy(self, y: f64) -> f64 {
        self.pad + (self.yard_d - y) * self.px_ft
    }
}

#[cfg(feature = "stories")]
#[path = "grid.stories.rs"]
mod grid_stories;
#[cfg(feature = "stories")]
#[path = "planner.stories.rs"]
mod planner_stories;
#[cfg(feature = "stories")]
#[path = "scale_bar.stories.rs"]
mod scale_bar_stories;
#[cfg(feature = "stories")]
#[path = "yard_controls.stories.rs"]
mod yard_controls_stories;
#[cfg(feature = "stories")]
#[path = "yard.stories.rs"]
mod yard_stories;

/// All slp-ui stories, in display order.
#[cfg(feature = "stories")]
pub fn stories() -> Vec<theoria::Story> {
    let mut s = Vec::new();
    s.extend(planner_stories::stories());
    s.extend(yard_stories::stories());
    s.extend(grid_stories::stories());
    s.extend(scale_bar_stories::stories());
    s.extend(yard_controls_stories::stories());
    s
}

#[cfg(test)]
#[path = "grid.tests.rs"]
mod grid_tests;
#[cfg(test)]
#[path = "planner.tests.rs"]
mod planner_tests;
#[cfg(test)]
#[path = "scale_bar.tests.rs"]
mod scale_bar_tests;
#[cfg(test)]
#[path = "yard_controls.tests.rs"]
mod yard_controls_tests;
#[cfg(test)]
#[path = "yard.tests.rs"]
mod yard_tests;

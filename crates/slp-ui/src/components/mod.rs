//! slp-ui components, colocated Storybook-style (same convention as theoria):
//! `<name>.rs` + `<name>.stories.rs` (`stories` feature) + `<name>.tests.rs`
//! (`cfg(test)`). Dotted file names use `#[path]` (not valid module idents).

mod catalog_picker;
mod deck;
mod door;
mod estimate_panel;
mod furnishings;
mod grid;
mod house;
mod number_field;
mod placement;
mod planner;
mod scale_bar;
mod steps;
mod toggle;
mod tool_button;
mod tool_group;
mod wall;
mod window;
mod yard;
mod yard_controls;

pub use catalog_picker::CatalogPicker;
pub use deck::Deck;
pub use door::Door;
pub use estimate_panel::EstimatePanel;
pub use furnishings::Furnishings;
pub use grid::Grid;
pub use house::House;
pub use number_field::NumberField;
pub use placement::Placement;
pub use planner::Planner;
pub use scale_bar::ScaleBar;
pub use steps::Steps;
pub use toggle::Toggle;
pub use tool_button::ToolButton;
pub use tool_group::ToolGroup;
pub use wall::Wall;
pub use window::Window;
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

    /// Screen-x (px, in SVG user space) → world-x (feet). Inverse of [`sx`].
    ///
    /// [`sx`]: Transform::sx
    #[must_use]
    pub fn wx(self, sx: f64) -> f64 {
        (sx - self.pad) / self.px_ft
    }

    /// Screen-y (px, in SVG user space) → world-y (feet). Inverse of [`sy`].
    ///
    /// [`sy`]: Transform::sy
    #[must_use]
    pub fn wy(self, sy: f64) -> f64 {
        self.yard_d - (sy - self.pad) / self.px_ft
    }
}

#[cfg(feature = "stories")]
#[path = "catalog_picker.stories.rs"]
mod catalog_picker_stories;
#[cfg(feature = "stories")]
#[path = "deck.stories.rs"]
mod deck_stories;
#[cfg(feature = "stories")]
#[path = "door.stories.rs"]
mod door_stories;
#[cfg(feature = "stories")]
#[path = "estimate_panel.stories.rs"]
mod estimate_panel_stories;
#[cfg(feature = "stories")]
#[path = "furnishings.stories.rs"]
mod furnishings_stories;
#[cfg(feature = "stories")]
#[path = "grid.stories.rs"]
mod grid_stories;
#[cfg(feature = "stories")]
#[path = "house.stories.rs"]
mod house_stories;
#[cfg(feature = "stories")]
#[path = "number_field.stories.rs"]
mod number_field_stories;
#[cfg(feature = "stories")]
#[path = "placement.stories.rs"]
mod placement_stories;
#[cfg(feature = "stories")]
#[path = "planner.stories.rs"]
mod planner_stories;
#[cfg(feature = "stories")]
#[path = "scale_bar.stories.rs"]
mod scale_bar_stories;
#[cfg(feature = "stories")]
#[path = "steps.stories.rs"]
mod steps_stories;
#[cfg(feature = "stories")]
#[path = "toggle.stories.rs"]
mod toggle_stories;
#[cfg(feature = "stories")]
#[path = "tool_button.stories.rs"]
mod tool_button_stories;
#[cfg(feature = "stories")]
#[path = "tool_group.stories.rs"]
mod tool_group_stories;
#[cfg(feature = "stories")]
#[path = "wall.stories.rs"]
mod wall_stories;
#[cfg(feature = "stories")]
#[path = "window.stories.rs"]
mod window_stories;
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
    s.extend(placement_stories::stories());
    s.extend(house_stories::stories());
    s.extend(deck_stories::stories());
    s.extend(furnishings_stories::stories());
    s.extend(estimate_panel_stories::stories());
    s.extend(steps_stories::stories());
    // The composition ladder, smallest first: Door/Window → Wall → House.
    s.extend(door_stories::stories());
    s.extend(window_stories::stories());
    s.extend(wall_stories::stories());
    s.extend(grid_stories::stories());
    s.extend(scale_bar_stories::stories());
    // Reusable controls.
    s.extend(tool_button_stories::stories());
    s.extend(toggle_stories::stories());
    s.extend(number_field_stories::stories());
    s.extend(catalog_picker_stories::stories());
    s.extend(tool_group_stories::stories());
    s.extend(yard_controls_stories::stories());
    s
}

#[cfg(test)]
#[path = "catalog_picker.tests.rs"]
mod catalog_picker_tests;
#[cfg(test)]
#[path = "deck.tests.rs"]
mod deck_tests;
#[cfg(test)]
#[path = "door.tests.rs"]
mod door_tests;
#[cfg(test)]
#[path = "estimate_panel.tests.rs"]
mod estimate_panel_tests;
#[cfg(test)]
#[path = "furnishings.tests.rs"]
mod furnishings_tests;
#[cfg(test)]
#[path = "grid.tests.rs"]
mod grid_tests;
#[cfg(test)]
#[path = "house.tests.rs"]
mod house_tests;
#[cfg(test)]
#[path = "number_field.tests.rs"]
mod number_field_tests;
#[cfg(test)]
#[path = "placement.tests.rs"]
mod placement_tests;
#[cfg(test)]
#[path = "planner.tests.rs"]
mod planner_tests;
#[cfg(test)]
#[path = "scale_bar.tests.rs"]
mod scale_bar_tests;
#[cfg(test)]
#[path = "steps.tests.rs"]
mod steps_tests;
#[cfg(test)]
#[path = "toggle.tests.rs"]
mod toggle_tests;
#[cfg(test)]
#[path = "tool_button.tests.rs"]
mod tool_button_tests;
#[cfg(test)]
#[path = "tool_group.tests.rs"]
mod tool_group_tests;
#[cfg(test)]
#[path = "transform.tests.rs"]
mod transform_tests;
#[cfg(test)]
#[path = "wall.tests.rs"]
mod wall_tests;
#[cfg(test)]
#[path = "window.tests.rs"]
mod window_tests;
#[cfg(test)]
#[path = "yard_controls.tests.rs"]
mod yard_controls_tests;
#[cfg(test)]
#[path = "yard.tests.rs"]
mod yard_tests;

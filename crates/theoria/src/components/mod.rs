//! theoria components, colocated Storybook-style: each `<name>.rs` sits beside
//! its `<name>.stories.rs` (theoria stories, `stories` feature) and
//! `<name>.tests.rs` (dokime component tests, `cfg(test)`). The dotted file
//! names need `#[path]` since they aren't valid Rust module identifiers.

mod gallery;
mod story_nav;

pub use gallery::Gallery;
pub use story_nav::StoryNav;

#[cfg(feature = "stories")]
#[path = "story_nav.stories.rs"]
mod story_nav_stories;

/// All theoria stories (its own leaf components used as fixtures).
#[cfg(feature = "stories")]
pub fn stories() -> Vec<crate::Story> {
    story_nav_stories::stories()
}

#[cfg(test)]
#[path = "story_nav.tests.rs"]
mod story_nav_tests;

#[cfg(test)]
#[path = "gallery.tests.rs"]
mod gallery_tests;

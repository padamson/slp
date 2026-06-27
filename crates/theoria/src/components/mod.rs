//! theoria components, colocated Storybook-style: each `<name>.rs` sits beside
//! its `<name>.stories.rs` (theoria stories, `stories` feature) and
//! `<name>.tests.rs` (dokime component tests, `cfg(test)`). The dotted file
//! names need `#[path]` since they aren't valid Rust module identifiers.

mod controls;
mod gallery;
mod markdown;
mod show_code;
mod story_nav;

pub use controls::Controls;
pub use gallery::Gallery;
pub use markdown::Markdown;
pub use show_code::ShowCode;
pub use story_nav::StoryNav;

#[cfg(feature = "stories")]
#[path = "controls.stories.rs"]
mod controls_stories;
#[cfg(feature = "stories")]
#[path = "show_code.stories.rs"]
mod show_code_stories;
#[cfg(feature = "stories")]
#[path = "story_nav.stories.rs"]
mod story_nav_stories;

/// All theoria stories (its own leaf components used as fixtures), so theoria can
/// be previewed in its own gallery: `theoria serve --config theoria-e2e.toml`.
#[cfg(feature = "stories")]
pub fn stories() -> Vec<crate::Story> {
    let mut stories = story_nav_stories::stories();
    stories.extend(controls_stories::stories());
    stories.extend(show_code_stories::stories());
    stories
}

#[cfg(test)]
#[path = "controls.tests.rs"]
mod controls_tests;
#[cfg(test)]
#[path = "markdown.tests.rs"]
mod markdown_tests;
#[cfg(test)]
#[path = "show_code.tests.rs"]
mod show_code_tests;
#[cfg(test)]
#[path = "story_nav.tests.rs"]
mod story_nav_tests;

#[cfg(test)]
#[path = "gallery.tests.rs"]
mod gallery_tests;

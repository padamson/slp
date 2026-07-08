//! theoria story for `CourseEditor`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::Course;

use super::CourseEditor;
use theoria::Story;

fn noop_us() -> Callback<usize> {
    Callback::new(|_| {})
}
fn noop_um() -> Callback<(usize, String)> {
    Callback::new(|_| {})
}
fn noop_ud() -> Callback<(usize, f64)> {
    Callback::new(|_| {})
}
fn noop() -> Callback<()> {
    Callback::new(|()| {})
}

pub fn stories() -> Vec<Story> {
    vec![Story::new("Panels/CourseEditor/A paver's build-up", || {
        view! {
            <CourseEditor
                courses=vec![
                    Course::new(4.0, "gravel".to_string()),
                    Course::new(1.0, "sand".to_string()),
                ]
                material_options=vec![
                    ("gravel".to_string(), "Gravel base".to_string()),
                    ("sand".to_string(), "Bedding sand".to_string()),
                    ("stone-dust".to_string(), "Stone dust".to_string()),
                ]
                on_material=noop_um()
                on_depth=noop_ud()
                on_add=noop()
                on_remove=noop_us()
            />
        }
    })]
}

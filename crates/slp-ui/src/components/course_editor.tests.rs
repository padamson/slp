//! dokime component tests for `CourseEditor`.

use leptos::prelude::*;
use slp_core::Course;

use super::CourseEditor;

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

fn options() -> Vec<(String, String)> {
    vec![
        ("gravel".to_string(), "Gravel base".to_string()),
        ("sand".to_string(), "Bedding sand".to_string()),
    ]
}

fn editor(courses: Vec<Course>) -> String {
    dokime::render(move || {
        let courses = courses.clone();
        view! {
            <CourseEditor
                courses=courses
                material_options=options()
                on_material=noop_um()
                on_depth=noop_ud()
                on_add=noop()
                on_remove=noop_us()
            />
        }
    })
}

#[test]
fn renders_a_row_per_course_with_its_material_selected() {
    let html = editor(vec![
        Course::new(4.0, "gravel".to_string()),
        Course::new(1.0, "sand".to_string()),
    ]);
    assert!(
        html.contains(r#"data-testid="course-row-0""#),
        "first course row"
    );
    assert!(
        html.contains(r#"data-testid="course-row-1""#),
        "second course row"
    );
    // Each row's material dropdown reflects the course's material.
    assert!(
        html.contains(r#"value="gravel" selected"#),
        "row 0 has gravel selected"
    );
    assert!(
        html.contains(r#"value="sand" selected"#),
        "row 1 has sand selected"
    );
    // Both option labels are offered.
    assert!(html.contains("Gravel base"));
    assert!(html.contains("Bedding sand"));
}

#[test]
fn has_an_add_layer_button_and_a_remove_per_row() {
    let html = editor(vec![Course::new(4.0, "gravel".to_string())]);
    assert!(
        html.contains(r#"data-testid="course-add""#),
        "add-layer button"
    );
    assert!(
        html.contains(r#"data-testid="course-remove-0""#),
        "remove for row 0"
    );
}

#[test]
fn an_empty_course_list_shows_just_the_add_button() {
    let html = editor(vec![]);
    assert_eq!(
        dokime::count(&html, r#"data-testid="course-row"#),
        0,
        "no rows"
    );
    assert!(
        html.contains(r#"data-testid="course-add""#),
        "still offers add"
    );
}

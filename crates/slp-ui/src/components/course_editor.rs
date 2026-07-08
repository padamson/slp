//! The composition editor: the ordered sub-base **courses** beneath a drawn
//! surface (a paver's gravel base, bedding sand, …), each a material + a
//! thickness, editable per area. Embedded in the [`AreaInspector`](super::AreaInspector)
//! for a paver patio so two patios can sit on different gravels at different
//! depths. Controlled — the parent owns the `courses` and applies each edit,
//! recomputing cost live.

use leptos::prelude::*;
use slp_core::Course;

use super::{NumberField, SelectField};

#[allow(clippy::needless_pass_by_value)]
#[component]
pub fn CourseEditor(
    /// The area's ordered courses, top (surface-adjacent) to bottom.
    courses: Vec<Course>,
    /// Materials a course may be made of, as `(id, label)` — typically the
    /// catalog's bulk (per-yd³) materials.
    material_options: Vec<(String, String)>,
    /// Set course `i`'s material to the given catalog id.
    on_material: Callback<(usize, String)>,
    /// Set course `i`'s thickness (inches).
    on_depth: Callback<(usize, f64)>,
    /// Append a new course.
    on_add: Callback<()>,
    /// Remove course `i`.
    on_remove: Callback<usize>,
) -> impl IntoView {
    let rows = courses
        .into_iter()
        .enumerate()
        .map(|(i, course)| {
            let options = material_options.clone();
            view! {
                <div class="course-row" data-testid=format!("course-row-{i}")>
                    <SelectField
                        label=""
                        testid="course-material"
                        value=course.material_ref
                        options=options
                        on_change=Callback::new(move |id: String| on_material.run((i, id)))
                    />
                    <NumberField
                        label="in"
                        testid="course-depth"
                        value=course.depth_in
                        step=1.0
                        min=0.0
                        on_input=Callback::new(move |d: f64| on_depth.run((i, d)))
                    />
                    <button
                        class="course-remove"
                        data-testid=format!("course-remove-{i}")
                        on:click=move |_| on_remove.run(i)
                    >
                        "×"
                    </button>
                </div>
            }
        })
        .collect::<Vec<_>>();
    view! {
        <div class="course-editor" data-testid="course-editor">
            <div class="course-editor-label">"Composition"</div>
            {rows}
            <button class="course-add" data-testid="course-add" on:click=move |_| on_add.run(())>
                "+ Layer"
            </button>
        </div>
    }
}

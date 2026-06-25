//! Number inputs for the yard's width and depth (feet), via the shared
//! `NumberField`. Editing either updates the signal the canvas renders from;
//! dimensions are clamped to a sane minimum.

use leptos::prelude::*;

use super::NumberField;

/// Smallest allowed yard dimension, in feet.
const MIN_FT: f64 = 1.0;

#[component]
pub fn YardControls(
    width: ReadSignal<f64>,
    set_width: WriteSignal<f64>,
    depth: ReadSignal<f64>,
    set_depth: WriteSignal<f64>,
) -> impl IntoView {
    view! {
        <div class="yard-controls">
            <NumberField
                label="Width (ft)"
                testid="yard-width"
                value=width
                on_input=Callback::new(move |v: f64| set_width.set(v.max(MIN_FT)))
                step=0.5
                min=MIN_FT
            />
            <NumberField
                label="Depth (ft)"
                testid="yard-depth"
                value=depth
                on_input=Callback::new(move |v: f64| set_depth.set(v.max(MIN_FT)))
                step=0.5
                min=MIN_FT
            />
        </div>
    }
}

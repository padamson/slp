//! Number inputs for the yard's width and depth (feet). Editing either updates
//! the signal the canvas renders from. Dimensions are clamped to a sane minimum.

use leptos::prelude::*;

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
            <label>
                "Width (ft) "
                <input
                    data-testid="yard-width"
                    type="number"
                    min="1"
                    step="0.5"
                    prop:value=move || width.get()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                            set_width.set(v.max(MIN_FT));
                        }
                    }
                />
            </label>
            <label>
                "Depth (ft) "
                <input
                    data-testid="yard-depth"
                    type="number"
                    min="1"
                    step="0.5"
                    prop:value=move || depth.get()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                            set_depth.set(v.max(MIN_FT));
                        }
                    }
                />
            </label>
        </div>
    }
}

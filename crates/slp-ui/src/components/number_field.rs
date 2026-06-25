//! A labeled number input. Controlled — the parent owns the value; unparsable
//! input is ignored (the value just doesn't change).

use leptos::prelude::*;

#[component]
pub fn NumberField(
    label: &'static str,
    testid: &'static str,
    #[prop(into)] value: Signal<f64>,
    on_input: Callback<f64>,
    #[prop(default = 0.5)] step: f64,
    #[prop(optional)] min: Option<f64>,
) -> impl IntoView {
    view! {
        <label class="number-field">
            {label}
            " "
            <input
                type="number"
                data-testid=testid
                step=step
                min=min
                prop:value=move || value.get()
                on:input=move |ev| {
                    if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                        on_input.run(v);
                    }
                }
            />
        </label>
    }
}

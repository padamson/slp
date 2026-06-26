//! The controls ("knobs") panel: one editable widget per story arg. Editing a
//! widget writes the arg's signal, which the story's view reads — so the stage
//! re-renders live. The args come from the `#[story]` macro.

use leptos::prelude::*;

use crate::ArgControl;

#[component]
pub fn Controls(args: Vec<(&'static str, ArgControl)>) -> impl IntoView {
    (!args.is_empty()).then(move || {
        let rows = args
            .into_iter()
            .map(|(name, ctl)| {
                let widget = match ctl {
                    ArgControl::Bool(s) => view! {
                        <input
                            type="checkbox"
                            prop:checked=move || s.get()
                            on:change=move |ev| s.set(event_target_checked(&ev))
                        />
                    }
                    .into_any(),
                    ArgControl::Num(s) => view! {
                        <input
                            type="number"
                            step="0.5"
                            prop:value=move || s.get()
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                    s.set(v);
                                }
                            }
                        />
                    }
                    .into_any(),
                    ArgControl::Text(s) => view! {
                        <input
                            type="text"
                            prop:value=move || s.get()
                            on:input=move |ev| s.set(event_target_value(&ev))
                        />
                    }
                    .into_any(),
                };
                view! {
                    <tr class="control-row">
                        <td class="control-name">{name}</td>
                        <td class="control-input">{widget}</td>
                    </tr>
                }
            })
            .collect::<Vec<_>>();
        view! {
            <table class="theoria-controls">
                <tbody>{rows}</tbody>
            </table>
        }
    })
}

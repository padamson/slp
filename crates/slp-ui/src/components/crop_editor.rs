//! Adjust a color swatch's crop (B5): the pasted screenshot with a box overlay
//! at the current bounding box, plus X/Y/W/H inputs (as % of the image) to
//! nudge/resize it. Vision bounding boxes are usually close but not exact, so
//! this lets the user tighten the box before adding; "Use crop" re-crops the
//! region (via `vision::crop`) into the swatch.

use leptos::prelude::*;

use super::NumberField;
use crate::vision::{self, BBox};

#[component]
pub fn CropEditor(
    /// The pasted screenshot (a `data:` URI) to crop from.
    screenshot: String,
    /// The starting crop box.
    bbox: BBox,
    /// Apply the adjusted crop: the re-cropped swatch (if it succeeded) and the
    /// new box.
    on_apply: Callback<(Option<String>, BBox)>,
    /// Close without applying.
    on_close: Callback<()>,
) -> impl IntoView {
    // Box position/size as percentages, for tidy inputs and CSS.
    let x = RwSignal::new(bbox.x * 100.0);
    let y = RwSignal::new(bbox.y * 100.0);
    let w = RwSignal::new(bbox.width * 100.0);
    let h = RwSignal::new(bbox.height * 100.0);
    let screenshot = StoredValue::new(screenshot);

    let current = move || BBox {
        x: (x.get() / 100.0).clamp(0.0, 1.0),
        y: (y.get() / 100.0).clamp(0.0, 1.0),
        width: (w.get() / 100.0).clamp(0.0, 1.0),
        height: (h.get() / 100.0).clamp(0.0, 1.0),
    };
    let apply = move |_| {
        let bbox = current();
        let shot = screenshot.get_value();
        leptos::task::spawn_local(async move {
            let swatch = vision::crop(&shot, bbox).await;
            on_apply.run((swatch, bbox));
        });
    };

    let pct = |v: RwSignal<f64>| Callback::new(move |val: f64| v.set(val.clamp(0.0, 100.0)));
    view! {
        <div class="crop-editor" data-testid="crop-editor">
            <div class="crop-stage">
                <img class="crop-image" src=screenshot.get_value() alt="screenshot" />
                <div
                    class="crop-box"
                    data-testid="crop-box"
                    style=move || {
                        format!(
                            "left:{}%;top:{}%;width:{}%;height:{}%",
                            x.get(),
                            y.get(),
                            w.get(),
                            h.get(),
                        )
                    }
                />
            </div>
            <div class="crop-fields">
                <NumberField
                    label="X %"
                    testid="crop-x"
                    value=Signal::derive(move || x.get())
                    step=1.0
                    min=0.0
                    on_input=pct(x)
                />
                <NumberField
                    label="Y %"
                    testid="crop-y"
                    value=Signal::derive(move || y.get())
                    step=1.0
                    min=0.0
                    on_input=pct(y)
                />
                <NumberField
                    label="W %"
                    testid="crop-w"
                    value=Signal::derive(move || w.get())
                    step=1.0
                    min=0.0
                    on_input=pct(w)
                />
                <NumberField
                    label="H %"
                    testid="crop-h"
                    value=Signal::derive(move || h.get())
                    step=1.0
                    min=0.0
                    on_input=pct(h)
                />
            </div>
            <div class="crop-actions">
                <button class="crop-apply" data-testid="crop-apply" on:click=apply>
                    "Use crop"
                </button>
                <button
                    class="crop-cancel"
                    data-testid="crop-cancel"
                    on:click=move |_| on_close.run(())
                >
                    "Cancel"
                </button>
            </div>
        </div>
    }
}

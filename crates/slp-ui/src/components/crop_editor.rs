//! Adjust a color swatch's crop (B5): the pasted screenshot with a **draggable,
//! resizable box** at the current bounding box (drag the box to move it, its
//! corner handle to resize), plus X/Y/W/H % inputs. Vision boxes are usually
//! close but not exact, so this lets the user tighten the box before adding;
//! "Use crop" re-crops the region (via `vision::crop`) into the swatch.

use leptos::prelude::*;

use super::NumberField;
use crate::vision::{self, BBox};

/// An in-progress drag: whether it's a resize (vs a move), the pointer's start
/// position (client px), the box's start position/size (%), and the stage's
/// rendered size (px) to convert pixel deltas to fractions.
#[derive(Clone, Copy)]
#[cfg_attr(not(feature = "csr"), allow(dead_code))]
pub(crate) struct Drag {
    pub(crate) resize: bool,
    pub(crate) sx: f64,
    pub(crate) sy: f64,
    pub(crate) bx: f64,
    pub(crate) by: f64,
    pub(crate) bw: f64,
    pub(crate) bh: f64,
    pub(crate) rw: f64,
    pub(crate) rh: f64,
}

/// Where a drag has taken the box: from the drag's start state and the
/// pointer's current position (client px), the new `(x, y, w, h)` in percent.
/// A move slides the box; a resize grows it from its anchored top-left corner;
/// both stay clamped inside the image (and a resize never collapses below 2%).
pub(crate) fn drag_box(d: &Drag, cx: f64, cy: f64) -> (f64, f64, f64, f64) {
    let dx = (cx - d.sx) / d.rw * 100.0;
    let dy = (cy - d.sy) / d.rh * 100.0;
    if d.resize {
        (
            d.bx,
            d.by,
            (d.bw + dx).clamp(2.0, 100.0 - d.bx),
            (d.bh + dy).clamp(2.0, 100.0 - d.by),
        )
    } else {
        (
            (d.bx + dx).clamp(0.0, 100.0 - d.bw),
            (d.by + dy).clamp(0.0, 100.0 - d.bh),
            d.bw,
            d.bh,
        )
    }
}

#[allow(clippy::many_single_char_names, clippy::too_many_lines)]
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
    let stage = NodeRef::<leptos::html::Div>::new();
    let cbox = NodeRef::<leptos::html::Div>::new();
    let drag = RwSignal::new(None::<Drag>);

    // Begin a move/resize drag: capture the pointer on the box (so its move/up
    // handlers keep firing when the pointer outruns it mid-drag) and record the
    // start state so `on_move` can turn the pointer delta into a box delta.
    let begin = move |resize: bool| {
        move |ev: leptos::ev::PointerEvent| {
            ev.prevent_default();
            ev.stop_propagation();
            #[cfg(not(feature = "csr"))]
            let _ = resize;
            #[cfg(feature = "csr")]
            {
                use wasm_bindgen::JsCast;
                let (rw, rh) = stage.get_untracked().map_or((1.0, 1.0), |el| {
                    let r = el.get_bounding_client_rect();
                    (r.width().max(1.0), r.height().max(1.0))
                });
                if let Some(el) = cbox.get_untracked() {
                    let _ = el
                        .unchecked_ref::<web_sys::Element>()
                        .set_pointer_capture(ev.pointer_id());
                }
                drag.set(Some(Drag {
                    resize,
                    sx: f64::from(ev.client_x()),
                    sy: f64::from(ev.client_y()),
                    bx: x.get_untracked(),
                    by: y.get_untracked(),
                    bw: w.get_untracked(),
                    bh: h.get_untracked(),
                    rw,
                    rh,
                }));
            }
        }
    };
    let on_move = move |ev: leptos::ev::PointerEvent| {
        if let Some(d) = drag.get_untracked() {
            let (nx, ny, nw, nh) = drag_box(&d, f64::from(ev.client_x()), f64::from(ev.client_y()));
            x.set(nx);
            y.set(ny);
            w.set(nw);
            h.set(nh);
        }
    };
    let on_up = move |_ev: leptos::ev::PointerEvent| drag.set(None);

    let current = move || BBox {
        // Preserve which screenshot this box is on — the editor only nudges the
        // rectangle, not the source image.
        image: bbox.image,
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
    // A modal: the dimmed backdrop closes on click, while clicks inside the
    // dialog stop there so adjusting the crop never dismisses it.
    //
    // In the browser it renders through a `<Portal>` into `<body>`: the catalog
    // panel is `position: fixed` + scrollable, and WebKit clips fixed-position
    // descendants to such an ancestor's box — an inline modal paints only
    // within the panel in Safari (dimmed strip, dialog clipped away) while
    // Chrome shows it fine. Portaling out escapes that clip. Off the browser
    // (dokime/SSR) the Portal would render nothing, so emit the markup inline.
    let dialog = move || {
        view! {
        <div
            class="crop-backdrop"
            data-testid="crop-backdrop"
            on:click=move |_| on_close.run(())
        >
            <div
                class="crop-editor"
                data-testid="crop-editor"
                on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
            >
                <div class="crop-stage" data-testid="crop-stage" node_ref=stage>
                    <img class="crop-image" src=screenshot.get_value() alt="screenshot" />
                    <div
                        class="crop-box"
                        data-testid="crop-box"
                        node_ref=cbox
                        on:pointerdown=begin(false)
                        on:pointermove=on_move
                        on:pointerup=on_up
                        style=move || {
                            format!(
                                "left:{}%;top:{}%;width:{}%;height:{}%",
                                x.get(),
                                y.get(),
                                w.get(),
                                h.get(),
                            )
                        }
                    >
                        <div
                            class="crop-handle"
                            data-testid="crop-handle"
                            on:pointerdown=begin(true)
                        />
                    </div>
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
        </div>
        }
    };
    #[cfg(feature = "csr")]
    let out = {
        use leptos::portal::Portal;
        view! { <Portal>{dialog}</Portal> }.into_any()
    };
    #[cfg(not(feature = "csr"))]
    let out = dialog().into_any();
    out
}

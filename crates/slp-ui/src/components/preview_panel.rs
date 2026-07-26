//! The photorealistic preview (R4): a **Preview** button with an
//! *Overhead / Eye-level* toggle, and a portaled modal showing the result.
//!
//! Overhead sends the plan's derived raster as an init image, so the render
//! keeps things where they were drawn; eye-level sends only the scene prompt,
//! for a "standing in it" view that's plausible rather than exact. Generation is
//! on demand — nothing is stored in the plan.
//!
//! This component is presentational: the parent owns the state and does the
//! work, so the whole flow is drivable from tests without a model.
//! (Portaled like `CropEditor` — `WebKit` clips a `position: fixed`
//! descendant of a scrollable fixed panel.)

use leptos::prelude::*;

/// Which view the preview renders.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PreviewMode {
    /// Bird's-eye, conditioned on the plan raster — keeps the drawn layout.
    #[default]
    Overhead,
    /// Eye-level, prompt-only — plausible but layout-approximate.
    EyeLevel,
}

impl PreviewMode {
    /// The wire value the `slpRender` bridge dispatches on.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overhead => "overhead",
            Self::EyeLevel => "eye-level",
        }
    }
}

/// The preview's current state — one enum so the modal can't show a spinner and
/// an image at once.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum PreviewState {
    /// Nothing requested yet; the modal is closed.
    #[default]
    Idle,
    /// A generation is in flight.
    Working,
    /// Finished: a `data:` URI to show.
    Done(String),
    /// Failed: a human-readable message (backend unreachable, no plan, …).
    Failed(String),
}

#[component]
pub fn PreviewPanel(
    /// The current state; drives the modal.
    #[prop(into)]
    state: Signal<PreviewState>,
    /// The selected mode.
    #[prop(into)]
    mode: Signal<PreviewMode>,
    /// Pick a mode.
    on_mode: Callback<PreviewMode>,
    /// Generate (or regenerate) with the current mode.
    on_generate: Callback<()>,
    /// Dismiss the modal.
    on_close: Callback<()>,
) -> impl IntoView {
    let mode_btn = move |value: PreviewMode, label: &'static str, testid: &'static str| {
        view! {
            <button
                class="preview-mode"
                class:active=move || mode.get() == value
                data-testid=testid
                on:click=move |_| on_mode.run(value)
            >
                {label}
            </button>
        }
    };

    // The modal only exists once something's been asked for, so the button sits
    // alone until then.
    let modal = move || {
        let st = state.get();
        if st == PreviewState::Idle {
            return None;
        }
        let body = match st {
            PreviewState::Working => view! {
                <div class="preview-working" data-testid="preview-working">
                    <div class="preview-spinner"></div>
                    <p>"Rendering your plan…"</p>
                </div>
            }
            .into_any(),
            PreviewState::Done(uri) => view! {
                <img class="preview-image" data-testid="preview-image" src=uri alt="Photorealistic preview of the plan" />
            }
            .into_any(),
            PreviewState::Failed(msg) => view! {
                <div class="preview-failed" data-testid="preview-failed">
                    <p>{msg}</p>
                    <p class="preview-hint">
                        "The preview needs a local image backend — start SwarmUI, or check the endpoint in your preview settings."
                    </p>
                </div>
            }
            .into_any(),
            PreviewState::Idle => return None,
        };
        Some(view! {
            <div class="preview-backdrop" data-testid="preview-modal">
                <div class="preview-dialog">
                    <div class="preview-body">{body}</div>
                    <div class="preview-actions">
                        <button
                            class="preview-regenerate"
                            data-testid="preview-regenerate"
                            disabled=move || state.get() == PreviewState::Working
                            on:click=move |_| on_generate.run(())
                        >
                            "Regenerate"
                        </button>
                        <button
                            class="preview-close"
                            data-testid="preview-close"
                            on:click=move |_| on_close.run(())
                        >
                            "Close"
                        </button>
                    </div>
                </div>
            </div>
        })
    };

    #[cfg(feature = "csr")]
    let modal_out = {
        use leptos::portal::Portal;
        view! { <Portal>{modal}</Portal> }.into_any()
    };
    #[cfg(not(feature = "csr"))]
    let modal_out = modal().into_any();

    view! {
        <div class="preview-panel" data-testid="preview-panel">
            <div class="preview-modes">
                {mode_btn(PreviewMode::Overhead, "Overhead", "preview-mode-overhead")}
                {mode_btn(PreviewMode::EyeLevel, "Eye-level", "preview-mode-eye-level")}
            </div>
            <button
                class="preview-generate"
                data-testid="preview-generate"
                disabled=move || state.get() == PreviewState::Working
                on:click=move |_| on_generate.run(())
            >
                "Preview"
            </button>
            {modal_out}
        </div>
    }
}

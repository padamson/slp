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

use super::FileInput;

/// Which view the preview renders.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PreviewMode {
    /// Bird's-eye, conditioned on the plan raster — keeps the drawn layout.
    #[default]
    Overhead,
    /// Eye-level, prompt-only — plausible but layout-approximate.
    EyeLevel,
    /// Conditioned on a **photo of the real yard**: re-render *that* yard with
    /// the planned materials. Needs a photo; the plan supplies the prompt.
    FromPhoto,
}

impl PreviewMode {
    /// The wire value the `slpRender` bridge dispatches on.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overhead => "overhead",
            Self::EyeLevel => "eye-level",
            Self::FromPhoto => "from-photo",
        }
    }
}

/// One finished render, kept for the session so Regenerate doesn't throw away
/// the one you liked.
#[derive(Clone, PartialEq, Debug)]
pub struct Shot {
    /// The image as a `data:` URI.
    pub image: String,
    /// Which mode produced it.
    pub mode: PreviewMode,
    /// Where the backend keeps its own full-resolution copy, if it does.
    pub source: Option<String>,
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
    /// Finished — the render to show.
    Done(Shot),
    /// Failed: a human-readable message (backend unreachable, no plan, …).
    Failed(String),
}

/// What the modal shows for a given state. Free-standing because it's a pure
/// state → view mapping, and it keeps `PreviewPanel` readable.
fn modal_body(state: PreviewState) -> AnyView {
    match state {
        PreviewState::Working => view! {
            <div class="preview-working" data-testid="preview-working">
                <div class="preview-spinner"></div>
                <p>"Rendering your plan…"</p>
            </div>
        }
        .into_any(),
        PreviewState::Done(shot) => {
            // The backend's own copy outlives this session, so say where it is.
            // A browser can't open Finder, so the honest affordance is a
            // selectable path, not a button that pretends to.
            let source = shot.source.clone().map(|s| {
                view! {
                    <p class="preview-source" data-testid="preview-source">
                        "Full resolution saved by the backend at " <code>{s}</code>
                    </p>
                }
            });
            view! {
                <img
                    class="preview-image"
                    data-testid="preview-image"
                    src=shot.image
                    alt="Photorealistic preview of the plan"
                />
                {source}
            }
            .into_any()
        }
        PreviewState::Failed(msg) => view! {
            <div class="preview-failed" data-testid="preview-failed">
                <p>{msg}</p>
                <p class="preview-hint">
                    "The preview needs a local image backend — start SwarmUI, or check the endpoint in your preview settings."
                </p>
            </div>
        }
        .into_any(),
        // The caller returns early on Idle; nothing to show.
        PreviewState::Idle => ().into_any(),
    }
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
    /// The yard photo backing `FromPhoto`, as a `data:` URI. Session-only — a
    /// photo is megabytes, and the plan shares a `localStorage` budget with it.
    #[prop(into, default = Signal::derive(|| None))]
    photo: Signal<Option<String>>,
    /// A photo was chosen (a `data:` URI), or cleared with `None`.
    #[prop(default = Callback::new(|_| {}))]
    on_photo: Callback<Option<String>>,
    /// This session's renders, newest last — so Regenerate doesn't discard the
    /// one you liked. In memory only; the backend keeps the full-res copies.
    #[prop(into, default = Signal::derive(Vec::new))]
    gallery: Signal<Vec<Shot>>,
    /// Reopen a gallery entry (by index).
    #[prop(default = Callback::new(|_| {}))]
    on_select: Callback<usize>,
    /// A filename stem for downloads, e.g. the plan's name.
    #[prop(into, default = Signal::derive(|| "landscape-plan".to_string()))]
    download_stem: Signal<String>,
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

    // Downloading a data: URI is just an anchor — no JS, no bridge. Named so a
    // folder of these stays legible: <plan>-<mode>.png.
    let download = move || {
        let PreviewState::Done(shot) = state.get() else {
            return None;
        };
        let name = format!("{}-{}.png", download_stem.get(), shot.mode.as_str());
        Some(view! {
            <a
                class="preview-download"
                data-testid="preview-download"
                href=shot.image
                download=name
            >
                "Download"
            </a>
        })
    };

    // Blocked while working, or in photo mode with no photo yet.
    let cant_generate = move || {
        state.get() == PreviewState::Working
            || (mode.get() == PreviewMode::FromPhoto && photo.get().is_none())
    };

    // The modal only exists once something's been asked for, so the button sits
    // alone until then.
    let modal = move || {
        let st = state.get();
        if st == PreviewState::Idle {
            return None;
        }
        let body = modal_body(st);
        Some(view! {
            <div class="preview-backdrop" data-testid="preview-modal">
                <div class="preview-dialog">
                    <div class="preview-body">{body}</div>
                    <div class="preview-actions">
                        {download}
                        <button
                            class="preview-regenerate"
                            data-testid="preview-regenerate"
                            disabled=cant_generate
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

    // The photo slot only appears for the mode that uses one. With a photo
    // chosen it shows a thumbnail and a clear button; without one it prompts,
    // and generating is blocked (there'd be nothing to condition on).
    let photo_slot = move || {
        (mode.get() == PreviewMode::FromPhoto).then(|| {
            photo.get().map_or_else(
                || {
                    view! {
                        <span class="preview-photo-pick">
                            <FileInput
                                label="Yard photo"
                                testid="preview-photo"
                                accept="image/*"
                                on_file=Callback::new(move |uri: String| on_photo.run(Some(uri)))
                            />
                        </span>
                    }
                    .into_any()
                },
                |uri| {
                    view! {
                        <span class="preview-photo-has">
                            <img class="preview-photo-thumb" data-testid="preview-photo-thumb" src=uri alt="The yard photo the render is based on" />
                            <button
                                class="preview-photo-clear"
                                data-testid="preview-photo-clear"
                                title="Remove the photo"
                                on:click=move |_| on_photo.run(None)
                            >
                                "×"
                            </button>
                        </span>
                    }
                    .into_any()
                },
            )
        })
    };

    view! {
        <div class="preview-panel" data-testid="preview-panel">
            <div class="preview-modes">
                {mode_btn(PreviewMode::Overhead, "Overhead", "preview-mode-overhead")}
                {mode_btn(PreviewMode::EyeLevel, "Eye-level", "preview-mode-eye-level")}
                {mode_btn(PreviewMode::FromPhoto, "From photo", "preview-mode-from-photo")}
            </div>
            {photo_slot}
            <button
                class="preview-generate"
                data-testid="preview-generate"
                disabled=cant_generate
                on:click=move |_| on_generate.run(())
            >
                "Preview"
            </button>
            {move || {
                let shots = gallery.get();
                (!shots.is_empty())
                    .then(|| {
                        let thumbs = shots
                            .into_iter()
                            .enumerate()
                            .map(|(i, shot)| {
                                view! {
                                    <img
                                        class="preview-thumb"
                                        data-testid="preview-thumb"
                                        src=shot.image
                                        title=shot.mode.as_str()
                                        alt="An earlier render"
                                        on:click=move |_| on_select.run(i)
                                    />
                                }
                            })
                            .collect::<Vec<_>>();
                        view! {
                            <span class="preview-gallery" data-testid="preview-gallery">
                                {thumbs}
                            </span>
                        }
                    })
            }}
            {modal_out}
        </div>
    }
}

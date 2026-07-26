//! Stories for `PreviewPanel` — every state the preview can be in.

use leptos::prelude::*;
use theoria::Story;

use super::preview_panel::{PreviewMode, PreviewPanel, PreviewState};

fn panel(state: PreviewState, mode: PreviewMode) -> impl IntoView {
    photo_panel(state, mode, None)
}

fn photo_panel(state: PreviewState, mode: PreviewMode, photo: Option<&str>) -> impl IntoView {
    let photo = photo.map(str::to_string);
    view! {
        <PreviewPanel
            state=Signal::derive(move || state.clone())
            mode=Signal::derive(move || mode)
            on_mode=Callback::new(|_| {})
            photo=Signal::derive(move || photo.clone())
            on_photo=Callback::new(|_| {})
            on_generate=Callback::new(|()| {})
            on_close=Callback::new(|()| {})
        />
    }
}

/// A tiny inline image so the "done" story shows something without shipping a
/// binary: a 1×1 olive PNG, scaled by the modal's own CSS.
const SAMPLE: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Panels/Preview/Idle (overhead selected)", || {
            panel(PreviewState::Idle, PreviewMode::Overhead)
        }),
        Story::new("Panels/Preview/Idle (eye-level selected)", || {
            panel(PreviewState::Idle, PreviewMode::EyeLevel)
        }),
        Story::new("Panels/Preview/Rendering", || {
            panel(PreviewState::Working, PreviewMode::Overhead)
        }),
        Story::new("Panels/Preview/Done", || {
            panel(
                PreviewState::Done(SAMPLE.to_string()),
                PreviewMode::Overhead,
            )
        }),
        Story::new("Panels/Preview/From photo (none chosen yet)", || {
            photo_panel(PreviewState::Idle, PreviewMode::FromPhoto, None)
        }),
        Story::new("Panels/Preview/From photo (chosen)", || {
            photo_panel(PreviewState::Idle, PreviewMode::FromPhoto, Some(SAMPLE))
        }),
        Story::new("Panels/Preview/Backend unreachable", || {
            panel(
                PreviewState::Failed(
                    "Can't reach the preview backend at http://localhost:7801 — is SwarmUI running?"
                        .to_string(),
                ),
                PreviewMode::Overhead,
            )
        }),
    ]
}

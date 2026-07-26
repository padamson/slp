//! Stories for `PreviewPanel` — every state the preview can be in.

use leptos::prelude::*;
use theoria::Story;

use super::preview_panel::{PreviewMode, PreviewPanel, PreviewState};

fn panel(state: PreviewState, mode: PreviewMode) -> impl IntoView {
    view! {
        <PreviewPanel
            state=Signal::derive(move || state.clone())
            mode=Signal::derive(move || mode)
            on_mode=Callback::new(|_| {})
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

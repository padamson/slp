//! dokime component tests for `PreviewPanel`. The parent owns the state, so
//! each state renders independently — no model, no network.

use leptos::prelude::*;

use super::preview_panel::{PreviewMode, PreviewPanel, PreviewState};

fn render(state: PreviewState, mode: PreviewMode) -> String {
    dokime::render(move || {
        view! {
            <PreviewPanel
                state=Signal::derive(move || state.clone())
                mode=Signal::derive(move || mode)
                on_mode=Callback::new(|_| {})
                on_generate=Callback::new(|()| {})
                on_close=Callback::new(|()| {})
            />
        }
    })
}

#[test]
fn idle_shows_the_button_and_modes_but_no_modal() {
    let html = render(PreviewState::Idle, PreviewMode::Overhead);
    assert!(
        html.contains(r#"data-testid="preview-generate""#),
        "the button"
    );
    assert!(html.contains(r#"data-testid="preview-mode-overhead""#));
    assert!(html.contains(r#"data-testid="preview-mode-eye-level""#));
    assert!(
        !html.contains(r#"data-testid="preview-modal""#),
        "nothing asked for yet, so no modal"
    );
}

#[test]
fn the_selected_mode_is_marked_active() {
    let overhead = render(PreviewState::Idle, PreviewMode::Overhead);
    assert_eq!(
        dokime::count(&overhead, "active"),
        1,
        "exactly one mode reads as selected"
    );
    // Switching the selection moves the marker to the other button.
    let eye = render(PreviewState::Idle, PreviewMode::EyeLevel);
    assert_eq!(dokime::count(&eye, "active"), 1);
    assert_ne!(
        overhead.find("active"),
        eye.find("active"),
        "the active marker sits on a different button"
    );
}

#[test]
fn working_shows_a_spinner_and_disables_generating_again() {
    let html = render(PreviewState::Working, PreviewMode::Overhead);
    assert!(html.contains(r#"data-testid="preview-working""#), "spinner");
    assert!(
        html.contains(r#"data-testid="preview-modal""#),
        "modal is open"
    );
    assert!(
        !html.contains(r#"data-testid="preview-image""#),
        "no image while it's still rendering"
    );
    assert!(html.contains("disabled"), "can't fire a second render");
}

#[test]
fn done_shows_the_image() {
    let uri = "data:image/png;base64,AAAA".to_string();
    let html = render(PreviewState::Done(uri), PreviewMode::Overhead);
    assert!(html.contains(r#"data-testid="preview-image""#));
    assert!(html.contains("data:image/png;base64,AAAA"), "the result");
    assert!(
        !html.contains(r#"data-testid="preview-working""#),
        "the spinner is gone"
    );
    assert!(html.contains(r#"data-testid="preview-regenerate""#));
}

#[test]
fn failure_shows_the_message_and_how_to_fix_it() {
    let html = render(
        PreviewState::Failed("Can't reach the preview backend at http://localhost:7801".into()),
        PreviewMode::Overhead,
    );
    assert!(html.contains(r#"data-testid="preview-failed""#));
    assert!(
        html.contains("Can't reach the preview backend"),
        "the cause"
    );
    assert!(html.contains("SwarmUI"), "a hint at what to do about it");
    assert!(
        !html.contains(r#"data-testid="preview-image""#),
        "no stale image behind a failure"
    );
}

#[test]
fn the_mode_wire_values_match_the_bridge() {
    // The bridge dispatches on these exact strings.
    assert_eq!(PreviewMode::Overhead.as_str(), "overhead");
    assert_eq!(PreviewMode::EyeLevel.as_str(), "eye-level");
    assert_eq!(PreviewMode::default(), PreviewMode::Overhead);
}

//! dokime component tests for `PreviewPanel`. The parent owns the state, so
//! each state renders independently — no model, no network.

use leptos::prelude::*;

use super::preview_panel::{PreviewMode, PreviewPanel, PreviewState, Shot};

fn render(state: PreviewState, mode: PreviewMode) -> String {
    render_with_photo(state, mode, None)
}

fn render_with_photo(state: PreviewState, mode: PreviewMode, photo: Option<&str>) -> String {
    render_full(state, mode, photo, Vec::new())
}

fn render_full(
    state: PreviewState,
    mode: PreviewMode,
    photo: Option<&str>,
    gallery: Vec<Shot>,
) -> String {
    let photo = photo.map(str::to_string);
    dokime::render(move || {
        let photo = photo.clone();
        let gallery = gallery.clone();
        view! {
            <PreviewPanel
                state=Signal::derive(move || state.clone())
                mode=Signal::derive(move || mode)
                on_mode=Callback::new(|_| {})
                photo=Signal::derive(move || photo.clone())
                on_photo=Callback::new(|_| {})
                gallery=Signal::derive(move || gallery.clone())
                on_select=Callback::new(|_| {})
                download_stem=Signal::derive(|| "my-yard".to_string())
                on_generate=Callback::new(|()| {})
                on_close=Callback::new(|()| {})
            />
        }
    })
}

/// A finished render for the states that need one.
fn shot(mode: PreviewMode, source: Option<&str>) -> Shot {
    Shot {
        image: "data:image/png;base64,AAAA".to_string(),
        mode,
        source: source.map(str::to_string),
    }
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
    let html = render(
        PreviewState::Done(shot(PreviewMode::Overhead, None)),
        PreviewMode::Overhead,
    );
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
    assert_eq!(PreviewMode::FromPhoto.as_str(), "from-photo");
    assert_eq!(PreviewMode::default(), PreviewMode::Overhead);
}

#[test]
fn the_photo_slot_appears_only_for_the_photo_mode() {
    for m in [PreviewMode::Overhead, PreviewMode::EyeLevel] {
        let html = render(PreviewState::Idle, m);
        assert!(
            !html.contains(r#"data-testid="preview-photo""#),
            "{m:?} needs no yard photo"
        );
    }
    let html = render(PreviewState::Idle, PreviewMode::FromPhoto);
    assert!(
        html.contains(r#"data-testid="preview-photo""#),
        "the photo picker shows"
    );
}

#[test]
fn photo_mode_cannot_generate_until_a_photo_is_chosen() {
    // Nothing to condition on yet, so the button is blocked...
    let empty = render(PreviewState::Idle, PreviewMode::FromPhoto);
    assert!(empty.contains("disabled"), "blocked without a photo");
    // ...and once chosen, the thumbnail replaces the picker and it unblocks.
    let chosen = render_with_photo(
        PreviewState::Idle,
        PreviewMode::FromPhoto,
        Some("data:image/png;base64,YARD"),
    );
    assert!(
        chosen.contains(r#"data-testid="preview-photo-thumb""#),
        "the chosen photo is shown back"
    );
    assert!(
        chosen.contains(r#"data-testid="preview-photo-clear""#),
        "and can be removed"
    );
    assert!(!chosen.contains("disabled"), "ready to generate");
}

#[test]
fn the_other_modes_never_block_on_a_missing_photo() {
    for m in [PreviewMode::Overhead, PreviewMode::EyeLevel] {
        let html = render(PreviewState::Idle, m);
        assert!(
            !html.contains("disabled"),
            "{m:?} generates without a photo"
        );
    }
}

#[test]
fn a_finished_render_can_be_downloaded_named_for_the_plan_and_mode() {
    let html = render(
        PreviewState::Done(shot(PreviewMode::EyeLevel, None)),
        PreviewMode::EyeLevel,
    );
    assert!(
        html.contains(r#"data-testid="preview-download""#),
        "a download"
    );
    assert!(
        html.contains(r#"download="my-yard-eye-level.png""#),
        "named from the plan stem and the mode, so a folder of these reads: {html}"
    );
}

#[test]
fn the_backends_own_copy_is_surfaced_when_there_is_one() {
    // SwarmUI writes full-res to disk before we fetch it — say where, rather
    // than implying the modal is the only copy.
    let with_path = render(
        PreviewState::Done(shot(
            PreviewMode::Overhead,
            Some("View/local/raw/2026-07-26/0909001-x.png"),
        )),
        PreviewMode::Overhead,
    );
    assert!(with_path.contains(r#"data-testid="preview-source""#));
    assert!(with_path.contains("View/local/raw/2026-07-26/0909001-x.png"));

    // A backend that keeps nothing says nothing.
    let without = render(
        PreviewState::Done(shot(PreviewMode::Overhead, None)),
        PreviewMode::Overhead,
    );
    assert!(!without.contains(r#"data-testid="preview-source""#));
}

#[test]
fn the_gallery_holds_this_sessions_renders() {
    // Empty until something's been generated...
    let none = render(PreviewState::Idle, PreviewMode::Overhead);
    assert!(!none.contains(r#"data-testid="preview-gallery""#));

    // ...then one thumbnail per render, so Regenerate doesn't lose the last one.
    let html = render_full(
        PreviewState::Idle,
        PreviewMode::Overhead,
        None,
        vec![
            shot(PreviewMode::Overhead, None),
            shot(PreviewMode::EyeLevel, None),
        ],
    );
    assert!(
        html.contains(r#"data-testid="preview-gallery""#),
        "the strip"
    );
    assert_eq!(
        dokime::count(&html, r#"data-testid="preview-thumb""#),
        2,
        "one thumbnail per render"
    );
}

#[test]
fn nothing_is_downloadable_before_a_render_finishes() {
    for st in [PreviewState::Idle, PreviewState::Working] {
        let html = render(st, PreviewMode::Overhead);
        assert!(
            !html.contains(r#"data-testid="preview-download""#),
            "there's no image to download yet"
        );
    }
}

//! dokime component tests for `CropEditor`, plus native unit tests for the
//! drag geometry (`drag_box`) — the pointer-to-box math is pure, so it's
//! verified here without a browser; the e2e covers the adjust→re-crop flow.

use leptos::prelude::*;

use super::CropEditor;
use super::crop_editor::{Drag, drag_box};
use crate::vision::BBox;

/// A drag that started at client (100, 100) on a 200×100 px stage, with the
/// box at 10%,20% sized 15%×15%.
fn start(resize: bool) -> Drag {
    Drag {
        resize,
        sx: 100.0,
        sy: 100.0,
        bx: 10.0,
        by: 20.0,
        bw: 15.0,
        bh: 15.0,
        rw: 200.0,
        rh: 100.0,
    }
}

#[test]
fn a_move_drag_converts_the_pixel_delta_to_stage_percent() {
    // +40px of a 200px stage = +20%; +10px of a 100px stage = +10%.
    let (x, y, w, h) = drag_box(&start(false), 140.0, 110.0);
    assert!((x - 30.0).abs() < 1e-9, "x moved to 30%, got {x}");
    assert!((y - 30.0).abs() < 1e-9, "y moved to 30%, got {y}");
    assert!((w - 15.0).abs() < 1e-9, "a move keeps the width, got {w}");
    assert!((h - 15.0).abs() < 1e-9, "a move keeps the height, got {h}");
}

#[test]
fn a_move_drag_clamps_inside_the_image() {
    // Way up-left: the box pins to the origin.
    let (x, y, ..) = drag_box(&start(false), -1000.0, -1000.0);
    assert!(x.abs() < 1e-9 && y.abs() < 1e-9, "pinned to 0,0: {x},{y}");
    // Way down-right: the box pins so it still fits (100 - size).
    let (x, y, ..) = drag_box(&start(false), 1000.0, 1000.0);
    assert!((x - 85.0).abs() < 1e-9, "x stops at 100-w, got {x}");
    assert!((y - 85.0).abs() < 1e-9, "y stops at 100-h, got {y}");
}

#[test]
fn a_resize_drag_grows_the_box_and_anchors_its_corner() {
    let (x, y, w, h) = drag_box(&start(true), 140.0, 110.0);
    assert!((x - 10.0).abs() < 1e-9, "a resize keeps x, got {x}");
    assert!((y - 20.0).abs() < 1e-9, "a resize keeps y, got {y}");
    assert!((w - 35.0).abs() < 1e-9, "w grew to 35%, got {w}");
    assert!((h - 25.0).abs() < 1e-9, "h grew to 25%, got {h}");
}

#[test]
fn a_resize_drag_clamps_between_minimum_and_the_image_edge() {
    // Collapsing inward stops at the 2% minimum.
    let (.., w, h) = drag_box(&start(true), -1000.0, -1000.0);
    assert!(
        (w - 2.0).abs() < 1e-9 && (h - 2.0).abs() < 1e-9,
        "min: {w}x{h}"
    );
    // Growing outward stops at the image edge (100 - position).
    let (.., w, h) = drag_box(&start(true), 1000.0, 1000.0);
    assert!((w - 90.0).abs() < 1e-9, "w stops at 100-x, got {w}");
    assert!((h - 80.0).abs() < 1e-9, "h stops at 100-y, got {h}");
}

fn render() -> String {
    dokime::render(|| {
        view! {
            <CropEditor
                screenshot="data:image/png;base64,AAA".to_string()
                bbox=BBox {
                    image: 0,
                    x: 0.1,
                    y: 0.2,
                    width: 0.15,
                    height: 0.15,
                }
                on_apply=Callback::new(|_: (Option<String>, BBox)| {})
                on_close=Callback::new(|()| {})
            />
        }
    })
}

#[test]
fn shows_the_screenshot_a_box_overlay_and_crop_inputs() {
    let html = render();
    // The editor is a modal: a dimmed backdrop wrapping the dialog, so the
    // stage gets real screen space instead of the catalog panel's column.
    assert!(
        html.contains(r#"data-testid="crop-backdrop""#),
        "the modal backdrop"
    );
    assert!(html.contains(r#"data-testid="crop-editor""#));
    assert!(
        html.contains(r#"data-testid="crop-box""#),
        "the crop box overlay"
    );
    assert!(
        html.contains(r#"data-testid="crop-handle""#),
        "the resize handle"
    );
    for f in ["crop-x", "crop-y", "crop-w", "crop-h"] {
        assert!(html.contains(&format!(r#"data-testid="{f}""#)), "input {f}");
    }
    assert!(html.contains(r#"data-testid="crop-apply""#));
    assert!(html.contains(r#"data-testid="crop-cancel""#));
    // The box overlay is positioned from the initial bbox (x = 10%).
    assert!(
        html.contains("left:10%"),
        "box positioned from bbox: {html}"
    );
}

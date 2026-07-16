//! dokime component tests for `CropEditor`.

use leptos::prelude::*;

use super::CropEditor;
use crate::vision::BBox;

fn render() -> String {
    dokime::render(|| {
        view! {
            <CropEditor
                screenshot="data:image/png;base64,AAA".to_string()
                bbox=BBox {
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
    assert!(html.contains(r#"data-testid="crop-editor""#));
    assert!(
        html.contains(r#"data-testid="crop-box""#),
        "the crop box overlay"
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

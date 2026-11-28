//! leptotes component tests for `ScaleBar`.

use leptos::prelude::*;

use super::{ScaleBar, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 30.0,
    }
}

#[test]
fn renders_label_and_one_line() {
    let dom = leptotes::dom(move || view! { <ScaleBar t=t() baseline_y=400.0 /> });

    assert_eq!(
        (dom.count("line"), dom.find("text").text()),
        (1, "10 ft".to_string()),
        "{dom}"
    );
}

#[test]
fn labels_a_non_default_length() {
    let dom = leptotes::dom(move || view! { <ScaleBar t=t() baseline_y=400.0 length_ft=20.0 /> });

    assert_eq!(dom.find("text").text(), "20 ft", "{dom}");
}

#[test]
fn scales_the_line_to_a_non_default_length() {
    let dom = leptotes::dom(move || view! { <ScaleBar t=t() baseline_y=400.0 length_ft=20.0 /> });

    // x0 = sx(0) = pad = 40; x2 = x0 + 20 ft * 12 px/ft = 280.
    assert_eq!(dom.find("line").attr("x2"), Some("280"), "{dom}");
}

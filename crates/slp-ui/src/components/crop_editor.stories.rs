//! theoria stories for `CropEditor`. Compiled only under the `stories` feature.

use leptos::prelude::*;

use super::CropEditor;
use crate::vision::BBox;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Panels/CropEditor/Adjust a swatch crop", || {
        view! {
            <CropEditor
                screenshot="data:image/png;base64,iVBORw0KGgo=".to_string()
                bbox=BBox {
                    image: 0,
                    x: 0.1,
                    y: 0.2,
                    width: 0.2,
                    height: 0.2,
                }
                on_apply=Callback::new(|_: (Option<String>, BBox)| {})
                on_close=Callback::new(|()| {})
            />
        }
    })]
}

//! dokime component test for `Door` (a door opening, in px space).

use leptos::prelude::*;

use super::Door;

#[test]
fn renders_a_door_with_leaf_and_swing() {
    let html = dokime::render(|| view! { <Door x1=0.0 y1=0.0 x2=10.0 y2=0.0 /> });
    assert!(html.contains(r#"class="door""#), "tagged as a door");
    assert!(html.contains(r#"class="door-leaf""#), "draws the leaf");
    assert!(
        html.contains(r#"class="door-swing""#) && html.contains("<path"),
        "draws the swing arc"
    );
}

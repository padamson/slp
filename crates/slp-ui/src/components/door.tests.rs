//! leptotes component test for `Door` (a door opening, in px space).

use leptos::prelude::*;

use super::Door;

#[test]
fn renders_a_door_with_leaf_and_swing() {
    let dom = leptotes::dom(|| view! { <Door x1=0.0 y1=0.0 x2=10.0 y2=0.0 /> });

    let door = dom.find("g.door");

    assert_eq!(
        (door.count("line.door-leaf"), door.count("path.door-swing")),
        (1, 1),
        "{dom}"
    );
}

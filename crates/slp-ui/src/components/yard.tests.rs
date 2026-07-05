//! dokime component tests for `Yard`.

use leptos::prelude::*;
use slp_core::{CatalogItem, Coord, DeckLevel, Object, StepRun};

use super::Yard;

#[test]
fn renders_the_yard_svg_with_scale_bar() {
    let html = dokime::render(|| view! { <Yard yard_w=10.0 yard_d=10.0 px_ft=12.0 pad=40.0 /> });
    assert!(
        html.contains(r#"data-testid="yard""#),
        "the yard canvas is present"
    );
    assert!(
        html.contains("10 ft"),
        "the scale bar renders inside the yard"
    );
}

#[test]
fn renders_the_house_outline_when_given_corners() {
    let house = vec![
        Coord::new(2.0, 2.0),
        Coord::new(6.0, 2.0),
        Coord::new(6.0, 5.0),
        Coord::new(2.0, 5.0),
    ];
    let html = dokime::render(
        move || view! { <Yard yard_w=10.0 yard_d=10.0 px_ft=12.0 pad=40.0 house=house /> },
    );
    assert!(
        html.contains(r#"class="house""#),
        "the house outline draws inside the yard stage"
    );
}

#[test]
fn renders_no_house_outline_by_default() {
    let html = dokime::render(|| view! { <Yard yard_w=10.0 yard_d=10.0 px_ft=12.0 pad=40.0 /> });
    assert!(
        !html.contains(r#"class="house""#),
        "a yard with no house draws no outline"
    );
}

#[test]
fn renders_placed_furniture_inside_the_stage() {
    let mut chair = CatalogItem::new("chair".to_string());
    chair.width_ft = Some(2.0);
    chair.depth_ft = Some(2.0);
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! {
            <Yard
                yard_w=10.0
                yard_d=10.0
                px_ft=12.0
                pad=40.0
                objects=objects
                catalog=vec![chair]
            />
        }
    });
    assert!(
        html.contains(r#"class="furnishings""#),
        "placed objects draw inside the yard stage"
    );
}

#[test]
fn renders_the_deck_and_a_step_run_inside_the_stage() {
    let level = DeckLevel {
        corners: vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(4.0, 3.0),
            Coord::new(0.0, 3.0),
        ],
        ..DeckLevel::new(1.5)
    };
    let run = StepRun {
        ax: 0.0,
        ay: 0.0,
        bx: 4.0,
        by: 0.0,
        elevation: 1.5,
    };
    let html = dokime::render(move || {
        view! {
            <Yard yard_w=10.0 yard_d=10.0 px_ft=12.0 pad=40.0 deck=vec![level] steps=vec![run] />
        }
    });
    assert!(
        html.contains(r#"class="deck""#),
        "the deck level draws inside the yard stage"
    );
    assert!(html.contains("<polygon"), "the deck polygon renders");
    assert!(
        html.contains(r#"class="steps""#),
        "the step run draws inside the yard stage"
    );
}

#[test]
fn renders_the_placement_overlay_while_placing() {
    let html = dokime::render(move || {
        view! {
            <Yard
                yard_w=10.0
                yard_d=10.0
                px_ft=12.0
                pad=40.0
                placed=vec![Coord::new(2.0, 2.0)]
                preview=Some(Coord::new(4.0, 4.0))
            />
        }
    });
    assert!(
        html.contains(r#"class="placement""#),
        "the placement overlay draws inside the yard stage"
    );
}

#[test]
fn renders_the_legend_by_default() {
    let html = dokime::render(|| view! { <Yard yard_w=10.0 yard_d=10.0 px_ft=12.0 pad=40.0 /> });
    assert!(
        html.contains(r#"data-testid="legend""#),
        "the legend renders inside the yard stage by default"
    );
}

#[test]
fn renders_the_selected_objects_rotation_handle() {
    let mut chair = CatalogItem::new("chair".to_string());
    chair.width_ft = Some(2.0);
    chair.depth_ft = Some(2.0);
    let objects = vec![Object::new("chair".to_string(), 5.0, 5.0)];
    let html = dokime::render(move || {
        view! {
            <Yard
                yard_w=10.0
                yard_d=10.0
                px_ft=12.0
                pad=40.0
                objects=objects
                catalog=vec![chair]
                selected=Some(0)
            />
        }
    });
    assert!(
        html.contains(r#"data-testid="rotate-handle""#),
        "the selected object's rotation handle is wired through the yard stage"
    );
}

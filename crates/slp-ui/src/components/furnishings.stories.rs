//! theoria stories for `Furnishings`. Compiled only under the `stories` feature.
//!
//! Split into one story per visual state (rather than one big scene) so each
//! status/virtual/selection/category look is individually inspectable in the
//! gallery — the same states `furnishings.tests.rs` pins in markup.

use leptos::prelude::*;
use slp_core::{CatalogItem, Coord, FootprintShape, ItemStatus, Object};

use super::{Furnishings, Transform};
use theoria::Story;

fn t() -> Transform {
    Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 30.0,
    }
}

fn canvas(children: impl IntoView) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
            <rect x="0" y="0" width="460" height="400" fill="#eef0e6" />
            {children}
        </svg>
    }
}

fn furniture(id: &str, name: &str, w_ft: f64, d_ft: f64) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c.width_ft = Some(w_ft);
    c.depth_ft = Some(d_ft);
    c
}

fn round(id: &str, name: &str, category: &str, diameter: f64) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c.category = Some(category.to_string());
    c.shape = FootprintShape::circle;
    c.width_ft = Some(diameter);
    c.depth_ft = Some(diameter);
    c
}

fn fire_pit(diameter: f64, clearance_ft: f64) -> CatalogItem {
    let mut c = round("fire-pit", "Fire pit", "fire-pit", diameter);
    c.clearance_ft = Some(clearance_ft);
    c
}

fn tree(id: &str, name: &str, canopy_ft: f64, trunk_ft: f64) -> CatalogItem {
    let mut c = round(id, name, "tree", canopy_ft);
    c.trunk_diameter_ft = Some(trunk_ft);
    c
}

#[allow(clippy::too_many_lines)]
pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Structures/Furnishings/A seating arrangement", || {
            let catalog = vec![
                furniture("sofa", "3-Seat Sofa", 7.0, 3.0),
                furniture("chair", "Armchair", 3.0, 3.0),
                furniture("table", "Coffee Table", 4.0, 2.0),
            ];
            // A little seating arrangement, including a rotated chair.
            let mut angled = Object::new("chair".to_string(), 26.0, 12.0);
            angled.rot = Some(30.0);
            let objects = vec![
                Object::new("sofa".to_string(), 14.0, 18.0),
                angled,
                Object::new("table".to_string(), 16.0, 12.0),
            ];
            canvas(view! { <Furnishings t=t() objects=objects catalog=catalog /> })
        }),
        Story::new("Structures/Furnishings/Existing (double outline)", || {
            let catalog = vec![furniture("sofa", "3-Seat Sofa", 7.0, 3.0)];
            let mut obj = Object::new("sofa".to_string(), 14.0, 15.0);
            obj.status = ItemStatus::existing;
            canvas(view! { <Furnishings t=t() objects=vec![obj] catalog=catalog /> })
        }),
        Story::new("Structures/Furnishings/Virtual (what-if ghost)", || {
            let catalog = vec![furniture("sofa", "3-Seat Sofa", 7.0, 3.0)];
            let mut obj = Object::new("sofa".to_string(), 14.0, 15.0);
            obj.is_virtual = true;
            canvas(view! { <Furnishings t=t() objects=vec![obj] catalog=catalog /> })
        }),
        Story::new(
            "Structures/Furnishings/Selected (tint + rotate handle)",
            || {
                let catalog = vec![furniture("chair", "Armchair", 3.0, 3.0)];
                let objects = vec![Object::new("chair".to_string(), 14.0, 15.0)];
                canvas(view! {
                    <Furnishings t=t() objects=objects catalog=catalog selected=Some(0) />
                })
            },
        ),
        Story::new("Structures/Furnishings/Doesn't fit its surface", || {
            let catalog = vec![furniture("table", "Coffee Table", 4.0, 4.0)];
            // A small 3x3 ft deck the 4x4 ft table overhangs on every side.
            let deck = vec![
                Coord::new(12.0, 13.5),
                Coord::new(15.0, 13.5),
                Coord::new(15.0, 16.5),
                Coord::new(12.0, 16.5),
            ];
            let objects = vec![Object::new("table".to_string(), 13.5, 15.0)];
            canvas(view! {
                <Furnishings t=t() objects=objects catalog=catalog surfaces=vec![deck] />
            })
        }),
        Story::new("Structures/Furnishings/Tree (canopy + trunk)", || {
            let catalog = vec![tree("oak-tree", "Oak tree", 16.0, 3.0)];
            let objects = vec![Object::new("oak-tree".to_string(), 14.0, 15.0)];
            canvas(view! { <Furnishings t=t() objects=objects catalog=catalog /> })
        }),
        Story::new(
            "Structures/Furnishings/Bush (green canopy, no trunk)",
            || {
                let catalog = vec![round("boxwood", "Boxwood", "bush", 6.0)];
                let objects = vec![Object::new("boxwood".to_string(), 14.0, 15.0)];
                canvas(view! { <Furnishings t=t() objects=objects catalog=catalog /> })
            },
        ),
        Story::new(
            "Structures/Furnishings/Tree, selected (resize handles)",
            || {
                let catalog = vec![tree("oak-tree", "Oak tree", 16.0, 3.0)];
                let objects = vec![Object::new("oak-tree".to_string(), 14.0, 15.0)];
                canvas(view! {
                    <Furnishings t=t() objects=objects catalog=catalog selected=Some(0) />
                })
            },
        ),
        Story::new(
            "Structures/Furnishings/Tree trunk on the house (invalid ground)",
            || {
                let catalog = vec![tree("oak-tree", "Oak tree", 16.0, 3.0)];
                let objects = vec![Object::new("oak-tree".to_string(), 14.0, 15.0)];
                // A house footprint the tree's trunk sits on — its canopy may
                // overhang freely, but the trunk itself flags red.
                let house = vec![
                    Coord::new(8.0, 10.0),
                    Coord::new(20.0, 10.0),
                    Coord::new(20.0, 20.0),
                    Coord::new(8.0, 20.0),
                ];
                canvas(view! {
                    <Furnishings t=t() objects=objects catalog=catalog house_outline=house />
                })
            },
        ),
        Story::new(
            "Structures/Furnishings/Fire pit, quiet clearance ring",
            || {
                let catalog = vec![fire_pit(3.0, 3.0)];
                let objects = vec![Object::new("fire-pit".to_string(), 14.0, 15.0)];
                canvas(view! { <Furnishings t=t() objects=objects catalog=catalog /> })
            },
        ),
        Story::new(
            "Structures/Furnishings/Fire pit, intruded clearance ring",
            || {
                let catalog = vec![fire_pit(3.0, 3.0), furniture("chair", "Armchair", 2.0, 2.0)];
                let objects = vec![
                    Object::new("fire-pit".to_string(), 14.0, 15.0),
                    // 2 ft away — inside the fire pit's 4.5 ft (radius 1.5 +
                    // clearance 3) keep-clear ring.
                    Object::new("chair".to_string(), 16.0, 15.0),
                ];
                canvas(view! { <Furnishings t=t() objects=objects catalog=catalog /> })
            },
        ),
    ]
}

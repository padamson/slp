//! theoria story for `Furnishings`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{CatalogItem, Object};

use super::{Furnishings, Transform};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Structures/Furnishings", || {
        let t = Transform {
            px_ft: 12.0,
            pad: 40.0,
            yard_d: 30.0,
        };
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
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                <rect x="0" y="0" width="460" height="400" fill="#eef0e6" />
                <Furnishings t=t objects=objects catalog=catalog />
            </svg>
        }
    })]
}

fn furniture(id: &str, name: &str, w_ft: f64, d_ft: f64) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c.width_ft = Some(w_ft);
    c.depth_ft = Some(d_ft);
    c
}

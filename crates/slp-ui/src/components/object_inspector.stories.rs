//! theoria story for `ObjectInspector`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{CatalogItem, Corner, ItemStatus, Object};

use super::ObjectInspector;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Panels/ObjectInspector", || {
        let mut object = Object::new("lounge-chair".to_string(), 14.0, 18.0);
        object.rot = Some(45.0);
        object.status = ItemStatus::planned;
        let mut item = CatalogItem::new("lounge-chair".to_string());
        item.name = Some("Lounge chair".to_string());
        item.category = Some("furniture".to_string());
        item.width_ft = Some(2.5);
        item.depth_ft = Some(3.0);
        item.height_ft = Some(2.5);
        item.unit_price = Some(199.0);
        let status = RwSignal::new(object.status.clone());
        view! {
            // A relative box stands in for the canvas the inspector floats over.
            <div style="position: relative; height: 260px; background: #eef0e6;">
                <ObjectInspector
                    object=object
                    item=Some(item)
                    corner=Corner::Nw
                    style="top: 10px; left: 10px;"
                    on_status=Callback::new(move |s| status.set(s))
                    on_virtual=Callback::new(|_| {})
                    on_reset_rotation=Callback::new(|()| {})
                    on_canopy_diameter=Callback::new(|_| {})
                    on_trunk_diameter=Callback::new(|_| {})
                    on_delete=Callback::new(|()| {})
                />
            </div>
        }
    })]
}

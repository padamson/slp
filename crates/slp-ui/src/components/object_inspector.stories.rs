//! theoria stories for `ObjectInspector`. Compiled only under the `stories`
//! feature. One story per render mode/state — the metadata dash-fallback,
//! round-item diameter, tree size inputs, status/virtual, and each floating
//! corner — since this panel has several visually distinct modes.

use leptos::prelude::*;
use slp_core::{CatalogItem, Corner, FootprintShape, ItemStatus, Object};

use super::ObjectInspector;
use theoria::Story;

/// A relative box stands in for the canvas the inspector floats over.
fn stage(children: impl IntoView) -> impl IntoView {
    view! {
        <div style="position: relative; height: 260px; background: #eef0e6;">
            {children}
        </div>
    }
}

fn chair() -> CatalogItem {
    let mut c = CatalogItem::new("lounge-chair".to_string());
    c.name = Some("Lounge chair".to_string());
    c.category = Some("furniture".to_string());
    c.width_ft = Some(2.5);
    c.depth_ft = Some(3.0);
    c.height_ft = Some(2.5);
    c.unit_price = Some(199.0);
    c
}

fn oak() -> CatalogItem {
    let mut c = CatalogItem::new("oak-tree".to_string());
    c.name = Some("Oak tree".to_string());
    c.category = Some("tree".to_string());
    c.shape = FootprintShape::circle;
    c.width_ft = Some(20.0);
    c.height_ft = Some(35.0);
    c.trunk_diameter_ft = Some(2.0);
    c.unit_price = Some(350.0);
    c
}

fn fire_pit() -> CatalogItem {
    let mut c = CatalogItem::new("fire-pit".to_string());
    c.name = Some("Fire pit".to_string());
    c.category = Some("fire-pit".to_string());
    c.shape = FootprintShape::circle;
    c.width_ft = Some(3.0);
    c.unit_price = Some(349.0);
    c
}

#[allow(clippy::too_many_lines)]
pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Panels/ObjectInspector/Furniture, planned", || {
            let mut object = Object::new("lounge-chair".to_string(), 14.0, 18.0);
            object.rot = Some(45.0);
            stage(view! {
                <ObjectInspector
                    object=object
                    item=Some(chair())
                    corner=Corner::Nw
                    style="top: 10px; left: 10px;"
                    on_status=Callback::new(|_| {})
                    on_virtual=Callback::new(|_| {})
                    on_reset_rotation=Callback::new(|()| {})
                    on_canopy_diameter=Callback::new(|_| {})
                    on_trunk_diameter=Callback::new(|_| {})
                    on_slab_thickness=Callback::new(|_| {})
                    on_slab_overhang=Callback::new(|_| {})
                    on_delete=Callback::new(|()| {})
                />
            })
        }),
        Story::new("Panels/ObjectInspector/Furniture, existing", || {
            let mut object = Object::new("lounge-chair".to_string(), 14.0, 18.0);
            object.status = ItemStatus::existing;
            stage(view! {
                <ObjectInspector
                    object=object
                    item=Some(chair())
                    corner=Corner::Nw
                    style="top: 10px; left: 10px;"
                    on_status=Callback::new(|_| {})
                    on_virtual=Callback::new(|_| {})
                    on_reset_rotation=Callback::new(|()| {})
                    on_canopy_diameter=Callback::new(|_| {})
                    on_trunk_diameter=Callback::new(|_| {})
                    on_slab_thickness=Callback::new(|_| {})
                    on_slab_overhang=Callback::new(|_| {})
                    on_delete=Callback::new(|()| {})
                />
            })
        }),
        Story::new(
            "Panels/ObjectInspector/Furniture, virtual (what-if ghost)",
            || {
                let mut object = Object::new("lounge-chair".to_string(), 14.0, 18.0);
                object.is_virtual = true;
                stage(view! {
                    <ObjectInspector
                        object=object
                        item=Some(chair())
                        corner=Corner::Nw
                        style="top: 10px; left: 10px;"
                        on_status=Callback::new(|_| {})
                        on_virtual=Callback::new(|_| {})
                        on_reset_rotation=Callback::new(|()| {})
                        on_canopy_diameter=Callback::new(|_| {})
                        on_trunk_diameter=Callback::new(|_| {})
                        on_slab_thickness=Callback::new(|_| {})
                        on_slab_overhang=Callback::new(|_| {})
                        on_delete=Callback::new(|()| {})
                    />
                })
            },
        ),
        Story::new(
            "Panels/ObjectInspector/Round item (fire pit diameter)",
            || {
                let object = Object::new("fire-pit".to_string(), 14.0, 18.0);
                stage(view! {
                    <ObjectInspector
                        object=object
                        item=Some(fire_pit())
                        corner=Corner::Nw
                        style="top: 10px; left: 10px;"
                        on_status=Callback::new(|_| {})
                        on_virtual=Callback::new(|_| {})
                        on_reset_rotation=Callback::new(|()| {})
                        on_canopy_diameter=Callback::new(|_| {})
                        on_trunk_diameter=Callback::new(|_| {})
                        on_slab_thickness=Callback::new(|_| {})
                        on_slab_overhang=Callback::new(|_| {})
                        on_delete=Callback::new(|()| {})
                    />
                })
            },
        ),
        Story::new(
            "Panels/ObjectInspector/Tree (canopy + trunk size inputs)",
            || {
                let object = Object::new("oak-tree".to_string(), 14.0, 18.0);
                stage(view! {
                    <ObjectInspector
                        object=object
                        item=Some(oak())
                        corner=Corner::Nw
                        style="top: 10px; left: 10px;"
                        on_status=Callback::new(|_| {})
                        on_virtual=Callback::new(|_| {})
                        on_reset_rotation=Callback::new(|()| {})
                        on_canopy_diameter=Callback::new(|_| {})
                        on_trunk_diameter=Callback::new(|_| {})
                        on_slab_thickness=Callback::new(|_| {})
                        on_slab_overhang=Callback::new(|_| {})
                        on_delete=Callback::new(|()| {})
                    />
                })
            },
        ),
        Story::new(
            "Panels/ObjectInspector/Missing catalog item (dashes)",
            || {
                let object = Object::new("deleted-from-catalog".to_string(), 14.0, 18.0);
                stage(view! {
                    <ObjectInspector
                        object=object
                        corner=Corner::Nw
                        style="top: 10px; left: 10px;"
                        on_status=Callback::new(|_| {})
                        on_virtual=Callback::new(|_| {})
                        on_reset_rotation=Callback::new(|()| {})
                        on_canopy_diameter=Callback::new(|_| {})
                        on_trunk_diameter=Callback::new(|_| {})
                        on_slab_thickness=Callback::new(|_| {})
                        on_slab_overhang=Callback::new(|_| {})
                        on_delete=Callback::new(|()| {})
                    />
                })
            },
        ),
        Story::new("Panels/ObjectInspector/Floats in the NE corner", || {
            let object = Object::new("lounge-chair".to_string(), 14.0, 18.0);
            stage(view! {
                <ObjectInspector
                    object=object
                    item=Some(chair())
                    corner=Corner::Ne
                    style="top: 10px; right: 10px;"
                    on_status=Callback::new(|_| {})
                    on_virtual=Callback::new(|_| {})
                    on_reset_rotation=Callback::new(|()| {})
                    on_canopy_diameter=Callback::new(|_| {})
                    on_trunk_diameter=Callback::new(|_| {})
                    on_slab_thickness=Callback::new(|_| {})
                    on_slab_overhang=Callback::new(|_| {})
                    on_delete=Callback::new(|()| {})
                />
            })
        }),
        Story::new("Panels/ObjectInspector/Floats in the SW corner", || {
            let object = Object::new("lounge-chair".to_string(), 14.0, 18.0);
            stage(view! {
                <ObjectInspector
                    object=object
                    item=Some(chair())
                    corner=Corner::Sw
                    style="bottom: 10px; left: 10px;"
                    on_status=Callback::new(|_| {})
                    on_virtual=Callback::new(|_| {})
                    on_reset_rotation=Callback::new(|()| {})
                    on_canopy_diameter=Callback::new(|_| {})
                    on_trunk_diameter=Callback::new(|_| {})
                    on_slab_thickness=Callback::new(|_| {})
                    on_slab_overhang=Callback::new(|_| {})
                    on_delete=Callback::new(|()| {})
                />
            })
        }),
    ]
}

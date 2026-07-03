//! theoria story for `ObjectPalette`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{CatalogItem, FootprintShape};

use super::ObjectPalette;
use theoria::Story;

fn item(id: &str, name: &str, category: &str, price: f64, circle: bool) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c.category = Some(category.to_string());
    c.unit_price = Some(price);
    if circle {
        c.shape = FootprintShape::circle;
    }
    c
}

pub fn stories() -> Vec<Story> {
    vec![Story::new("Panels/ObjectPalette", || {
        let catalog = vec![
            item("lounge-chair", "Lounge chair", "furniture", 199.0, false),
            item("dining-table", "Dining table", "furniture", 649.0, false),
            item("fire-pit", "Fire pit", "fire-pit", 349.0, true),
        ];
        // The dining table is armed, to show the highlighted tile.
        let armed = RwSignal::new(Some("dining-table".to_string()));
        view! {
            <div style="max-width: 420px;">
                <ObjectPalette
                    catalog=catalog
                    armed=armed
                    on_pick=Callback::new(move |id: String| {
                        armed.set(if armed.get() == Some(id.clone()) { None } else { Some(id) });
                    })
                />
            </div>
        }
    })]
}

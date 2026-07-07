//! theoria stories for `AreaInspector`. Compiled only under the `stories`
//! feature.

use leptos::prelude::*;
use slp_core::Corner;

use super::AreaInspector;
use theoria::Story;

fn noop_f64() -> Callback<f64> {
    Callback::new(|_| {})
}

fn noop() -> Callback<()> {
    Callback::new(|()| {})
}

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Panels/AreaInspector/Mulch bed", || {
            view! {
                <AreaInspector
                    title="Mulch".to_string()
                    category=Some("mulch-bed".to_string())
                    area_ft2=84.0
                    elevation=0.0
                    depth=3.0
                show_depth=true
                    cost=Some(31.11)
                    corner=Corner::Ne
                    on_elevation=noop_f64()
                    on_depth=noop_f64()
                    on_delete=noop()
                />
            }
        }),
        Story::new("Panels/AreaInspector/Paver area (no depth)", || {
            view! {
                <AreaInspector
                    title="Pavers".to_string()
                    category=Some("paver".to_string())
                    area_ft2=120.0
                    elevation=0.0
                    depth=0.0
                show_depth=false
                    cost=Some(960.0)
                    corner=Corner::Nw
                    on_elevation=noop_f64()
                    on_depth=noop_f64()
                    on_delete=noop()
                />
            }
        }),
    ]
}

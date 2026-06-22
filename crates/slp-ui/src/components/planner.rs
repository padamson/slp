//! The whole planner UI: header + yard controls + the to-scale yard canvas,
//! wired to a `slp_core::Plan` (the schema-generated model). `slp-app` just
//! mounts this — keeping the root UI here (not in the binary) makes the entire
//! app previewable in theoria and testable with dokime. The plan is persisted to
//! `localStorage` so the yard size survives a reload. Grows with the toolbar,
//! side panel, etc.

use leptos::prelude::*;
use slp_core::{Coord, House, Plan};

use super::{Yard, YardControls};

/// Pixels per foot in the SVG user space.
const PX_FT: f64 = 12.0;
/// Padding around the yard, in pixels.
const PAD: f64 = 40.0;
/// Default yard size in feet (first run, before anything is saved).
const DEFAULT_W: f64 = 70.0;
const DEFAULT_D: f64 = 30.0;
/// `localStorage` key for the persisted plan (only used in the browser build).
#[cfg(feature = "csr")]
const STORAGE_KEY: &str = "slp:plan";

#[component]
pub fn Planner() -> impl IntoView {
    let plan = load_plan().unwrap_or(Plan {
        yard_width: DEFAULT_W,
        yard_depth: DEFAULT_D,
        ..Default::default()
    });
    let (width, set_width) = signal(plan.yard_width);
    let (depth, set_depth) = signal(plan.yard_depth);
    // The house outline (drawn in a later slice); empty until then. Preserved
    // across saves so changing the yard size never wipes a drawn house.
    let house: Vec<Coord> = plan.house.map(|h| h.corners).unwrap_or_default();

    // Persist the plan whenever a dimension changes (no-op under ssr / in tests).
    let house_for_save = house.clone();
    Effect::new(move |_| {
        save_plan(&Plan {
            yard_width: width.get(),
            yard_depth: depth.get(),
            house: (!house_for_save.is_empty()).then(|| {
                Box::new(House {
                    corners: house_for_save.clone(),
                })
            }),
            ..Default::default()
        });
    });

    view! {
        <header>
            <h1>"Simple Landscape Planner"</h1>
            <p class="sub">"Set your yard size; the plan is drawn to scale."</p>
        </header>
        <YardControls width=width set_width=set_width depth=depth set_depth=set_depth />
        // Re-render the canvas whenever the dimensions change.
        {move || {
            view! {
                <Yard yard_w=width.get() yard_d=depth.get() px_ft=PX_FT pad=PAD house=house.clone() />
            }
        }}
    }
}

#[cfg(feature = "csr")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// The persisted plan, if any. `None` off the browser (ssr / tests).
fn load_plan() -> Option<Plan> {
    #[cfg(feature = "csr")]
    {
        let json = storage()?.get_item(STORAGE_KEY).ok().flatten()?;
        serde_json::from_str(&json).ok()
    }
    #[cfg(not(feature = "csr"))]
    {
        None
    }
}

/// Persist the plan as JSON (no-op off the browser).
fn save_plan(plan: &Plan) {
    #[cfg(feature = "csr")]
    {
        if let (Some(s), Ok(json)) = (storage(), serde_json::to_string(plan)) {
            let _ = s.set_item(STORAGE_KEY, &json);
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = plan;
    }
}

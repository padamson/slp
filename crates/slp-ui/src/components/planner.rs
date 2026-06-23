//! The whole planner UI: header + yard controls + the to-scale yard canvas,
//! wired to a `slp_core::Plan` (the schema-generated model). `slp-app` just
//! mounts this — keeping the root UI here (not in the binary) makes the entire
//! app previewable in theoria and testable with dokime. The plan is persisted to
//! `localStorage` so the yard size *and* the drawn house survive a reload. Grows
//! with the side panel, etc.

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
/// Snap radius (ft): clicking within this of the first corner closes the outline.
const SNAP_FT: f64 = 2.0;
/// `localStorage` key for the persisted plan (only used in the browser build).
#[cfg(feature = "csr")]
const STORAGE_KEY: &str = "slp:plan";

/// What a canvas click does while drawing the house outline.
#[derive(Debug, PartialEq)]
pub(crate) enum Pick {
    /// Add this corner to the outline.
    Add(Coord),
    /// Close the ring (the click snapped back to the first corner).
    Close,
}

/// Decide whether a click adds a corner or closes the outline. The ring closes
/// only with at least three corners and a click within `snap_ft` of the first.
pub(crate) fn classify_pick(corners: &[Coord], at: Coord, snap_ft: f64) -> Pick {
    let near_start = corners
        .first()
        .is_some_and(|c| (c.x - at.x).hypot(c.y - at.y) <= snap_ft);
    if corners.len() >= 3 && near_start {
        Pick::Close
    } else {
        Pick::Add(at)
    }
}

#[component]
pub fn Planner() -> impl IntoView {
    let plan = load_plan().unwrap_or(Plan {
        yard_width: DEFAULT_W,
        yard_depth: DEFAULT_D,
        ..Default::default()
    });
    let (width, set_width) = signal(plan.yard_width);
    let (depth, set_depth) = signal(plan.yard_depth);
    // The house outline (corners, in feet) and whether we're drawing it.
    let corners = RwSignal::new(plan.house.map(|h| h.corners).unwrap_or_default());
    let drawing = RwSignal::new(false);

    // Persist whenever the yard size or the house changes (no-op under ssr /
    // tests). The house is kept, so resizing the yard never wipes a drawn house.
    Effect::new(move |_| {
        let house = corners.get();
        save_plan(&Plan {
            yard_width: width.get(),
            yard_depth: depth.get(),
            house: (!house.is_empty()).then(|| Box::new(House { corners: house })),
            ..Default::default()
        });
    });

    // A click on the canvas (in feet) adds a corner, or closes the ring.
    let on_pick = Callback::new(move |at: Coord| {
        if !drawing.get_untracked() {
            return;
        }
        match classify_pick(&corners.get_untracked(), at, SNAP_FT) {
            Pick::Add(c) => corners.update(|v| v.push(c)),
            Pick::Close => drawing.set(false),
        }
    });

    // The Draw button starts a fresh outline; while drawing it cancels.
    let toggle_draw = move |_| {
        if drawing.get_untracked() {
            drawing.set(false);
        } else {
            corners.set(Vec::new());
            drawing.set(true);
        }
    };

    view! {
        <header>
            <h1>"Simple Landscape Planner"</h1>
            <p class="sub">"Set your yard size; the plan is drawn to scale."</p>
        </header>
        <YardControls width=width set_width=set_width depth=depth set_depth=set_depth />
        <div class="tools">
            <button data-testid="draw-house" class:active=move || drawing.get() on:click=toggle_draw>
                {move || {
                    if drawing.get() { "Click near the start to finish" } else { "Draw house" }
                }}
            </button>
        </div>
        // Re-render the canvas whenever the dimensions or the outline change.
        {move || {
            view! {
                <Yard
                    yard_w=width.get()
                    yard_d=depth.get()
                    px_ft=PX_FT
                    pad=PAD
                    house=corners.get()
                    on_pick=on_pick
                />
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

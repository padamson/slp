//! The whole planner UI: header + yard controls + the to-scale yard canvas,
//! wired to a `slp_core::Plan` (the schema-generated model). `slp-app` just
//! mounts this — keeping the root UI here (not in the binary) makes the entire
//! app previewable in theoria and testable with dokime. The plan is persisted to
//! `localStorage` so the yard size *and* the drawn house survive a reload. Grows
//! with the side panel, etc.

use leptos::prelude::*;
use slp_core::{Coord, House, Plan, snap_ortho, snap_to_grid};

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
/// Grid step (ft) that corners snap to when grid-snap is on (matches the minor grid).
const GRID_STEP: f64 = 1.0;
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
/// Closing is decided on the *raw* click so snapping never blocks it.
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

/// Apply the active snaps to a corner being added: grid first, then ortho
/// (axis-align the edge from the previous corner). Either can be off.
pub(crate) fn apply_snaps(corners: &[Coord], at: Coord, grid: bool, ortho: bool) -> Coord {
    let mut p = at;
    if grid {
        p = snap_to_grid(&p, GRID_STEP);
    }
    if ortho && let Some(prev) = corners.last() {
        p = snap_ortho(prev, &p);
    }
    p
}

#[component]
pub fn Planner() -> impl IntoView {
    planner_body()
}

// Body in a plain fn so the composition-root line count (expanded `view!`
// macros) can be allowed; signals/effects still run in the component's owner.
#[allow(clippy::too_many_lines)]
fn planner_body() -> impl IntoView {
    let plan = load_plan().unwrap_or(Plan {
        yard_width: DEFAULT_W,
        yard_depth: DEFAULT_D,
        ..Default::default()
    });
    let (width, set_width) = signal(plan.yard_width);
    let (depth, set_depth) = signal(plan.yard_depth);
    // The house outline (corners, in feet), its openings, and the draw mode.
    let (init_corners, init_openings) = plan
        .house
        .map(|h| {
            let House {
                corners, openings, ..
            } = *h;
            (corners, openings)
        })
        .unwrap_or_default();
    let corners = RwSignal::new(init_corners);
    let openings = RwSignal::new(init_openings);
    let drawing = RwSignal::new(false);
    // Snapping (on by default): most walls are on the grid and axis-aligned.
    let grid_snap = RwSignal::new(true);
    let ortho = RwSignal::new(true);
    // The node being positioned while the mouse is held (snapped), drawn as a ghost.
    let pending = RwSignal::new(None::<Coord>);

    // Persist whenever the yard size or the house changes (no-op under ssr /
    // tests). The house is kept, so resizing the yard never wipes a drawn house.
    Effect::new(move |_| {
        let cs = corners.get();
        let os = openings.get();
        let house = (!cs.is_empty() || !os.is_empty()).then(|| {
            Box::new(House {
                corners: cs,
                openings: os,
            })
        });
        save_plan(&Plan {
            yard_width: width.get(),
            yard_depth: depth.get(),
            house,
            ..Default::default()
        });
    });

    // Releasing the mouse (in feet) adds a (snapped) corner, or closes the ring.
    let on_pick = Callback::new(move |at: Coord| {
        if !drawing.get_untracked() {
            return;
        }
        let cs = corners.get_untracked();
        match classify_pick(&cs, at, SNAP_FT) {
            Pick::Close => drawing.set(false),
            Pick::Add(raw) => {
                let p = apply_snaps(&cs, raw, grid_snap.get_untracked(), ortho.get_untracked());
                corners.update(|v| v.push(p));
            }
        }
    });

    // While the mouse is held, show the snapped position the node will drop at.
    let on_preview = Callback::new(move |at: Option<Coord>| {
        if !drawing.get_untracked() {
            pending.set(None);
            return;
        }
        let cs = corners.get_untracked();
        pending.set(
            at.map(|raw| apply_snaps(&cs, raw, grid_snap.get_untracked(), ortho.get_untracked())),
        );
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
            <label>
                <input
                    type="checkbox"
                    data-testid="snap-grid"
                    prop:checked=move || grid_snap.get()
                    on:change=move |ev| grid_snap.set(event_target_checked(&ev))
                />
                " Snap to grid"
            </label>
            <label>
                <input
                    type="checkbox"
                    data-testid="snap-ortho"
                    prop:checked=move || ortho.get()
                    on:change=move |ev| ortho.set(event_target_checked(&ev))
                />
                " Straight walls"
            </label>
        </div>
        // Recreate the stage only when the yard size changes; the outline and
        // preview are read reactively inside Yard, so the <svg> persists during
        // a mouse gesture (needed for press-to-aim / release-to-drop).
        {move || {
            view! {
                <Yard
                    yard_w=width.get()
                    yard_d=depth.get()
                    px_ft=PX_FT
                    pad=PAD
                    house=corners
                    openings=openings
                    preview=pending
                    on_pick=on_pick
                    on_preview=on_preview
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

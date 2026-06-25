//! The whole planner UI: header + yard controls + drawing tools + the to-scale
//! yard canvas, wired to a `slp_core::Plan`. `slp-app` just mounts this — keeping
//! the root UI here (not in the binary) makes the entire app previewable in
//! theoria and testable with dokime. The plan is persisted to `localStorage`.
//!
//! Drawing uses one node-placement engine (`slp_core::place`): a tool (house,
//! door, window) previews the next node as the mouse moves and commits it on
//! release, until the object completes.

use leptos::prelude::*;
use slp_core::{Commit, Coord, House, Plan, Tool, commit_kind, opening_from_nodes, snap_node};

use super::{Yard, YardControls};

/// Pixels per foot in the SVG user space.
const PX_FT: f64 = 12.0;
/// Padding around the yard, in pixels.
const PAD: f64 = 40.0;
/// Default yard size in feet (first run, before anything is saved).
const DEFAULT_W: f64 = 70.0;
const DEFAULT_D: f64 = 30.0;
/// Grid step (ft) that nodes snap to when grid-snap is on (matches the minor grid).
const GRID_STEP: f64 = 1.0;
/// `localStorage` key for the persisted plan (only used in the browser build).
#[cfg(feature = "csr")]
const STORAGE_KEY: &str = "slp:plan";

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
    // The committed house: outline corners + openings.
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
    // Placement engine state: the active tool, the nodes placed this gesture,
    // and the previewed next node under the cursor.
    let tool = RwSignal::new(None::<Tool>);
    let placed = RwSignal::new(Vec::<Coord>::new());
    let preview = RwSignal::new(None::<Coord>);
    // Snapping (on by default): most walls are on the grid and axis-aligned.
    let grid_snap = RwSignal::new(true);
    let ortho = RwSignal::new(true);

    // Persist whenever the yard size or the committed house changes.
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

    // Snap the cursor to where the next node would land, for the active tool.
    let snap = move |tl: Tool, raw: &Coord| {
        snap_node(
            tl,
            &corners.get_untracked(),
            &placed.get_untracked(),
            raw,
            grid_snap.get_untracked(),
            ortho.get_untracked(),
            GRID_STEP,
        )
    };

    // Pointer move → preview the next node.
    let on_hover = Callback::new(move |raw: Coord| {
        if let Some(tl) = tool.get_untracked() {
            preview.set(Some(snap(tl, &raw)));
        }
    });

    // Pointer release → commit a node (or close / finish the object).
    let on_commit = Callback::new(move |raw: Coord| {
        let Some(tl) = tool.get_untracked() else {
            return;
        };
        let next = snap(tl, &raw);
        let pl = placed.get_untracked();
        match commit_kind(tl, &pl, &next) {
            Commit::Add => placed.update(|v| v.push(next)),
            Commit::Finish => {
                corners.set(pl); // the placed nodes become the outline
                reset(tool, placed, preview);
            }
            Commit::FinishWith => {
                if let (Some(kind), Some(start)) = (tl.opening_kind(), pl.first())
                    && let Some(o) =
                        opening_from_nodes(&corners.get_untracked(), kind, start, &next)
                {
                    openings.update(|v| v.push(o));
                }
                reset(tool, placed, preview);
            }
        }
    });

    let on_leave = Callback::new(move |()| preview.set(None));

    // Arm a tool (or toggle it off). Starting the house clears the old one;
    // starting an opening keeps the house.
    let pick_tool = move |t: Tool| {
        if tool.get_untracked() == Some(t) {
            reset(tool, placed, preview);
            return;
        }
        if t == Tool::House {
            corners.set(Vec::new());
            openings.set(Vec::new());
        }
        placed.set(Vec::new());
        preview.set(None);
        tool.set(Some(t));
    };

    let tool_active = move |t: Tool| tool.get() == Some(t);

    view! {
        <header>
            <h1>"Simple Landscape Planner"</h1>
            <p class="sub">"Set your yard size; the plan is drawn to scale."</p>
        </header>
        <YardControls width=width set_width=set_width depth=depth set_depth=set_depth />
        <div class="tools">
            <button
                data-testid="draw-house"
                class:active=move || tool_active(Tool::House)
                on:click=move |_| pick_tool(Tool::House)
            >
                "Draw house"
            </button>
            <button
                data-testid="add-door"
                class:active=move || tool_active(Tool::Door)
                on:click=move |_| pick_tool(Tool::Door)
            >
                "Add door"
            </button>
            <button
                data-testid="add-window"
                class:active=move || tool_active(Tool::Window)
                on:click=move |_| pick_tool(Tool::Window)
            >
                "Add window"
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
        <p class="hint" data-testid="hint">{move || hint(tool.get())}</p>
        // Recreate the stage only when the yard size changes; the plan, the
        // placement, and the preview are read reactively inside Yard, so the
        // <svg> persists during a pointer gesture.
        {move || {
            view! {
                <Yard
                    yard_w=width.get()
                    yard_d=depth.get()
                    px_ft=PX_FT
                    pad=PAD
                    house=corners
                    openings=openings
                    placed=placed
                    preview=preview
                    on_hover=on_hover
                    on_commit=on_commit
                    on_leave=on_leave
                />
            }
        }}
    }
}

/// Clear the active tool and any in-progress placement.
fn reset(
    tool: RwSignal<Option<Tool>>,
    placed: RwSignal<Vec<Coord>>,
    preview: RwSignal<Option<Coord>>,
) {
    placed.set(Vec::new());
    preview.set(None);
    tool.set(None);
}

/// The status hint for the active tool.
fn hint(tool: Option<Tool>) -> &'static str {
    match tool {
        None => "Pick a tool to draw.",
        Some(Tool::House) => "Click corners; click the first corner to close the outline.",
        Some(Tool::Door) => "Click two points on a wall to place the door.",
        Some(Tool::Window) => "Click two points on a wall to place the window.",
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

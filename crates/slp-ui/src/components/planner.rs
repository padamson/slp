//! The whole planner UI: header + yard controls + drawing tools + the to-scale
//! yard canvas, wired to a `slp_core::Plan`. `slp-app` just mounts this — keeping
//! the root UI here (not in the binary) makes the entire app previewable in
//! theoria and testable with dokime. The plan is persisted to `localStorage`.
//!
//! Drawing uses one node-placement engine (`slp_core::place`): a tool (house,
//! door, window) previews the next node as the mouse moves and commits it on
//! release, until the object completes.

use leptos::prelude::*;
use slp_core::{
    CatalogItem, Commit, Coord, Deck, DeckLevel, House, Object, Plan, StepRun, Tool, commit_kind,
    nearest_wall, opening_from_nodes, snap_node,
};

use super::{CatalogPicker, NumberField, Toggle, ToolButton, ToolGroup, Yard, YardControls};

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
    let (init_levels, init_steps) = plan
        .deck
        .map(|d| {
            let Deck { levels, steps, .. } = *d;
            (levels, steps)
        })
        .unwrap_or_default();
    let deck = RwSignal::new(init_levels);
    let steps = RwSignal::new(init_steps);
    // Placed objects + the catalog they reference. `selected_id` is the catalog
    // item the furniture tool will drop; `seeded` guards the one-time starter
    // catalog (see the seed effect below).
    let init_catalog = plan.catalog;
    let init_selected = init_catalog
        .first()
        .map_or_else(String::new, |c| c.id.clone());
    let objects = RwSignal::new(plan.objects);
    let catalog = RwSignal::new(init_catalog);
    let selected_id = RwSignal::new(init_selected);
    let seeded = RwSignal::new(false);
    // The elevation (ft) the next deck level is drawn at.
    let elevation = RwSignal::new(1.0_f64);
    // Placement engine state: the active tool, the nodes placed this gesture,
    // and the previewed next node under the cursor.
    let tool = RwSignal::new(None::<Tool>);
    let placed = RwSignal::new(Vec::<Coord>::new());
    let preview = RwSignal::new(None::<Coord>);
    // Snapping (on by default): most walls are on the grid and axis-aligned.
    let grid_snap = RwSignal::new(true);
    let ortho = RwSignal::new(true);

    // Persist whenever the yard size or the committed plan changes.
    Effect::new(move |_| {
        let cs = corners.get();
        let os = openings.get();
        let house = (!cs.is_empty() || !os.is_empty()).then(|| {
            Box::new(House {
                corners: cs,
                openings: os,
            })
        });
        let dk = deck.get();
        let st = steps.get();
        let deck = (!dk.is_empty() || !st.is_empty()).then(|| {
            Box::new(Deck {
                levels: dk,
                steps: st,
            })
        });
        save_plan(&Plan {
            yard_width: width.get(),
            yard_depth: depth.get(),
            house,
            deck,
            catalog: catalog.get(),
            objects: objects.get(),
            ..Default::default()
        });
    });

    // Seed a starter furniture catalog the first time a deck is drawn — the
    // surface furniture sits on. Guarded so it runs once and never fights a user
    // who clears it; a loaded plan that already has a catalog is left alone.
    Effect::new(move |_| {
        if !deck.get().is_empty() && !seeded.get_untracked() && catalog.get_untracked().is_empty() {
            let starter = starter_catalog();
            if let Some(first) = starter.first() {
                selected_id.set(first.id.clone());
            }
            catalog.set(starter);
            seeded.set(true);
        }
    });

    // Snap the cursor to where the next node would land, for the active tool.
    // Steps snap to a deck edge (the nearest level); openings to the house.
    let snap = move |tl: Tool, raw: &Coord| {
        let outline = if tl == Tool::Steps {
            let anchor = placed
                .get_untracked()
                .first()
                .cloned()
                .unwrap_or_else(|| raw.clone());
            nearest_level(&deck.get_untracked(), &anchor)
                .map(|l| l.corners)
                .unwrap_or_default()
        } else {
            corners.get_untracked()
        };
        snap_node(
            tl,
            &outline,
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
                // The placed nodes become a committed outline: a new deck level
                // (decks are multi-level — additive) or the house outline.
                if tl == Tool::Deck {
                    let level = DeckLevel {
                        corners: pl,
                        elevation: elevation.get_untracked(),
                    };
                    deck.update(|v| v.push(level));
                } else {
                    corners.set(pl);
                }
                reset(tool, placed, preview);
            }
            Commit::FinishWith if tl == Tool::Furniture => {
                // Drop the selected catalog item at the clicked point.
                let id = selected_id.get_untracked();
                if !id.is_empty() {
                    objects.update(|v| v.push(Object::new(id, next.x, next.y)));
                }
                reset(tool, placed, preview);
            }
            Commit::FinishWith if tl == Tool::Steps => {
                // A step run on the deck edge nearest the first node; its drop is
                // that level's elevation.
                if let (Some(start), Some(lvl)) =
                    (pl.first(), nearest_level(&deck.get_untracked(), &next))
                {
                    steps.update(|v| {
                        v.push(StepRun {
                            ax: start.x,
                            ay: start.y,
                            bx: next.x,
                            by: next.y,
                            elevation: lvl.elevation,
                        });
                    });
                }
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
        // Redrawing the house replaces it; decks are additive (multi-level).
        if t == Tool::House {
            corners.set(Vec::new());
            openings.set(Vec::new());
        }
        placed.set(Vec::new());
        preview.set(None);
        tool.set(Some(t));
    };

    // One callback the tool buttons share; per-button derivations live in tool_btn.
    let pick = Callback::new(pick_tool);

    view! {
        <header>
            <h1>"Simple Landscape Planner"</h1>
            <p class="sub">"Set your yard size; the plan is drawn to scale."</p>
        </header>
        <YardControls width=width set_width=set_width depth=depth set_depth=set_depth />
        <div class="tools">
            <ToolGroup label="House">
                {tool_btn(tool, pick, Tool::House, "Draw house", "draw-house")}
                {tool_btn(tool, pick, Tool::Door, "Add door", "add-door")}
                {tool_btn(tool, pick, Tool::Window, "Add window", "add-window")}
            </ToolGroup>
            <ToolGroup label="Deck">
                {tool_btn(tool, pick, Tool::Deck, "Draw deck", "draw-deck")}
                {tool_btn(tool, pick, Tool::Steps, "Add steps", "add-steps")}
                <NumberField
                    label="Elev (ft)"
                    testid="deck-elevation"
                    value=elevation
                    on_input=Callback::new(move |v| elevation.set(v))
                    step=0.5
                />
            </ToolGroup>
            // The furniture group appears once there's a catalog (seeded when a
            // deck is drawn).
            {move || {
                (!catalog.get().is_empty())
                    .then(|| {
                        view! {
                            <ToolGroup label="Furniture">
                                {tool_btn(
                                    tool,
                                    pick,
                                    Tool::Furniture,
                                    "Place furniture",
                                    "place-furniture",
                                )}
                                <CatalogPicker
                                    testid="catalog-picker"
                                    catalog=catalog
                                    selected=Signal::derive(move || selected_id.get())
                                    on_pick=Callback::new(move |id| selected_id.set(id))
                                />
                            </ToolGroup>
                        }
                    })
            }}
            <ToolGroup label="Snap">
                <Toggle
                    label="Snap to grid"
                    testid="snap-grid"
                    checked=grid_snap
                    on_toggle=Callback::new(move |v| grid_snap.set(v))
                />
                <Toggle
                    label="Straight walls"
                    testid="snap-ortho"
                    checked=ortho
                    on_toggle=Callback::new(move |v| ortho.set(v))
                />
            </ToolGroup>
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
                    deck=deck
                    steps=steps
                    openings=openings
                    objects=objects
                    catalog=catalog
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

/// A toolbar button for `t`, wired to the shared `pick` callback and highlighting
/// when `t` is the active tool.
fn tool_btn(
    tool: RwSignal<Option<Tool>>,
    pick: Callback<Tool>,
    t: Tool,
    label: &'static str,
    testid: &'static str,
) -> impl IntoView {
    let active = Signal::derive(move || tool.get() == Some(t));
    view! { <ToolButton label=label testid=testid active=active on_pick=Callback::new(move |()| pick.run(t)) /> }
}

/// The deck level whose nearest edge is closest to `anchor` (where a step run
/// attaches) — its elevation is the run's drop.
fn nearest_level(levels: &[DeckLevel], anchor: &Coord) -> Option<DeckLevel> {
    levels
        .iter()
        .filter_map(|lvl| nearest_wall(&lvl.corners, anchor).map(|(_, _, d)| (d, lvl)))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, lvl)| lvl.clone())
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
        Some(Tool::Deck) => "Click corners; click the first corner to close the deck.",
        Some(Tool::Door) => "Click two points on a wall to place the door.",
        Some(Tool::Window) => "Click two points on a wall to place the window.",
        Some(Tool::Steps) => "Click two points on a deck edge to add steps.",
        Some(Tool::Furniture) => "Click to place the selected item on the plan.",
    }
}

/// A small starter catalog of deck furniture, seeded the first time a deck is
/// drawn. Plan data the user can place, ignore, or (once catalog editing lands)
/// replace — not hardcoded geometry. Footprints are in feet, prices in dollars.
fn starter_catalog() -> Vec<CatalogItem> {
    let furniture = |id: &str, name: &str, w: f64, d: f64, h: f64, price: f64| {
        let mut c = CatalogItem::new(id.to_string());
        c.name = Some(name.to_string());
        c.category = Some("furniture".to_string());
        c.width_ft = Some(w);
        c.depth_ft = Some(d);
        c.height_ft = Some(h);
        c.unit_price = Some(price);
        c
    };
    vec![
        furniture("lounge-chair", "Lounge chair", 2.5, 3.0, 2.5, 199.0),
        furniture("outdoor-sofa", "Outdoor sofa", 7.0, 3.0, 2.5, 899.0),
        furniture("dining-table", "Dining table", 4.0, 6.0, 2.5, 649.0),
        furniture("side-table", "Side table", 1.5, 1.5, 1.5, 89.0),
        furniture("patio-umbrella", "Patio umbrella", 9.0, 9.0, 8.0, 149.0),
    ]
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

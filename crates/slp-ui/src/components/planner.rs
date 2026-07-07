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
    CatalogItem, Circle, Commit, Coord, Corner, CurveEdge, Deck, DeckLevel, FootprintShape, House,
    ItemStatus, Object, Plan, Point, PriceUnit, Shape, StepRun, Tool, are_adjacent, boundary_area,
    circle_area, commit_kind, content_points, delete_node, dragged_center, free_corner, heading,
    insert_node_between, nearest_wall, object_at, opening_from_nodes, snap_node, snap_to_grid,
    take_off,
};

use super::{
    AreaInspector, CanvasMetrics, CatalogPanel, EstimatePanel, Footprint, Modifiers, NumberField,
    ObjectInspector, ObjectPalette, Toggle, ToolButton, ToolGroup, Yard, YardControls,
};

/// Pixels per foot in the SVG user space.
const PX_FT: f64 = 12.0;
/// Grid padding (px). Zero in the app so the grid sits flush to its canvas box
/// and lines up with the page layout; the scale bar has its own reserved strip.
const PAD: f64 = 0.0;
/// Default yard size in feet (first run, before anything is saved).
const DEFAULT_W: f64 = 70.0;
const DEFAULT_D: f64 = 30.0;
/// Grid step (ft) that nodes snap to when grid-snap is on (matches the minor grid).
const GRID_STEP: f64 = 1.0;
/// Rotation snap increment (degrees) when grid-snap is on.
const ROT_STEP: f64 = 15.0;
/// Approximate footprint (px) of the object-inspector window, converted to feet
/// via the measured px-per-foot to judge which yard corner is empty enough for it.
const INSPECTOR_W_PX: f64 = 210.0;
const INSPECTOR_H_PX: f64 = 190.0;
/// Inset (px) of the inspector window from the grid corner.
const INSPECTOR_MARGIN_PX: f64 = 10.0;
/// `localStorage` key for the persisted plan (only used in the browser build).
#[cfg(feature = "csr")]
const STORAGE_KEY: &str = "slp:plan";

/// An in-progress move: which object is being dragged, and the offset (feet)
/// from the cursor to the object's center at grab time, so the object follows
/// the cursor without snapping its center under the pointer.
#[derive(Clone, Copy)]
struct Drag {
    index: usize,
    grab_x: f64,
    grab_y: f64,
}

/// Which of a selected tree's two handles is being dragged.
#[derive(Clone, Copy, PartialEq)]
enum ResizePart {
    Canopy,
    Trunk,
}

/// A tree's canopy never shrinks below this (ft) while its edge handle is
/// dragged — small enough for a sapling, never zero or negative.
const MIN_CANOPY_FT: f64 = 1.0;
/// A tree's trunk never shrinks below this (ft) while its edge handle is
/// dragged.
const MIN_TRUNK_FT: f64 = 0.2;
/// A drawn circle never shrinks below this radius (ft) while its resize
/// handle is dragged.
const MIN_CIRCLE_RADIUS_FT: f64 = 0.5;
/// A dragged edge bulge is clamped to this magnitude (|1| is a semicircle;
/// this allows a major arc without letting the radius blow up numerically).
const BULGE_LIMIT: f64 = 2.0;
/// A dragged edge bulge rounds to this step, so an arc lands on tidy curvature.
const BULGE_STEP: f64 = 0.05;

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
    // The committed house: outline corners + openings + build status. The
    // status is editable from the area inspector when the house is selected.
    let (init_corners, init_openings, init_house_status) = plan.house.map_or_else(
        || (Vec::new(), Vec::new(), ItemStatus::existing),
        |h| {
            let House {
                corners,
                openings,
                structure_status,
            } = *h;
            (corners, openings, structure_status)
        },
    );
    let corners = RwSignal::new(init_corners);
    let openings = RwSignal::new(init_openings);
    let house_status = RwSignal::new(init_house_status);
    let (init_levels, init_steps) = plan
        .deck
        .map(|d| {
            let Deck { levels, steps, .. } = *d;
            (levels, steps)
        })
        .unwrap_or_default();
    let deck = RwSignal::new(init_levels);
    let steps = RwSignal::new(init_steps);
    // Drawn areas (paver patios, mulch beds, …).
    let shapes = RwSignal::new(plan.shapes);
    // The index (into `shapes`) of the selected drawn area, if any — its
    // nodes become interactive (mirrors `selected` for objects).
    let selected_shape = RwSignal::new(None::<usize>);
    // Indices (into the selected shape's corners) of the selected nodes: 0, 1,
    // or an adjacent pair (which arms the insert-between popup).
    let selected_nodes = RwSignal::new(Vec::<usize>::new());
    // The node (index into the selected shape's corners) being dragged, if any.
    let dragging_node = RwSignal::new(None::<usize>);
    // The edge (index into the selected shape's edges) whose bulge handle is
    // being dragged, if any — dragging it bows that edge into an arc.
    let dragging_edge = RwSignal::new(None::<usize>);
    // The `(edge, which-control)` whose Bézier control handle is being dragged,
    // if any — dragging it curves that edge (which=0 is control1, 1 is control2).
    let dragging_control = RwSignal::new(None::<(usize, usize)>);
    // Whether the house is selected — its corners become interactive (mirrors
    // `selected_shape`, but there's only ever one house).
    let house_selected = RwSignal::new(false);
    // The house corner (index into `corners`) being dragged, if any.
    let dragging_house_node = RwSignal::new(None::<usize>);
    // The index (into `deck`'s levels, *before* Deck's own paint-order sort)
    // of the selected level, if any.
    let selected_deck = RwSignal::new(None::<usize>);
    // The node (index into the selected level's corners) being dragged, if any.
    let dragging_deck_node = RwSignal::new(None::<usize>);
    // Standalone circular drawn areas (round paver patios, mulch beds, …).
    let circles = RwSignal::new(plan.circles);
    // The index (into `circles`) of the selected circle, if any — it shows a
    // resize handle (mirrors `selected_shape`, one handle instead of nodes).
    let selected_circle = RwSignal::new(None::<usize>);
    // True while dragging a selected circle's resize handle.
    let circle_resizing = RwSignal::new(false);
    // True from a shape/house/deck-level body's press (or the insert/cancel
    // popup buttons) to its matching release — consumed by `on_commit` so
    // that same click doesn't also fall through to the empty-space case and
    // immediately clear the selection just set (none of these start a drag of
    // their own the way an object press does).
    let press_only = RwSignal::new(false);
    // Placed objects + the catalog they reference. A tree (or a fire pit) goes
    // straight in the yard — no deck required — so the starter catalog is
    // seeded immediately when the loaded plan has none; a plan that already
    // has a catalog is left alone. `selected_id` is the catalog item the
    // object tool will drop.
    let init_catalog = if plan.catalog.is_empty() {
        starter_catalog()
    } else {
        plan.catalog
    };
    let init_selected = init_catalog
        .first()
        .map_or_else(String::new, |c| c.id.clone());
    let objects = RwSignal::new(plan.objects);
    let catalog = RwSignal::new(init_catalog);
    let selected_id = RwSignal::new(init_selected);
    // The catalog inspector: whether its editing panel is open, and which
    // catalog item (by id) is being edited.
    let catalog_open = RwSignal::new(false);
    let catalog_selected = RwSignal::new(None::<String>);
    // The index (into `objects`) of the selected placed object, if any.
    let selected = RwSignal::new(None::<usize>);
    // The canvas's rendered geometry, measured once per resize (from Yard).
    let metrics = RwSignal::new(CanvasMetrics::default());
    // True while dragging the selected object's rotation handle.
    let rotating = RwSignal::new(false);
    // Which of a selected tree's canopy/trunk handles is being dragged, if any.
    let resizing = RwSignal::new(None::<ResizePart>);
    // The in-progress object move (its body is held), if any.
    let dragging = RwSignal::new(None::<Drag>);
    // The last cursor position in feet, used to grab a moved object at its offset.
    let hover_at = RwSignal::new(None::<Coord>);
    // The elevation (ft) the next deck level is drawn at.
    let elevation = RwSignal::new(1.0_f64);
    // The elevation (ft) the next drawn area (boundary or circle) is at —
    // grade by default (a paver/mulch area usually sits flush with the yard,
    // unlike a deck). Shared by both bed shapes: a bed's elevation doesn't
    // depend on whether it's drawn as a polygon or a circle.
    let area_elevation = RwSignal::new(0.0_f64);
    // The material (catalog id) the next drawn area is made of — mulch by
    // default (the only area material so far; pavers/gravel join as those
    // stories land). Drives the drawn shape's look and how it's costed.
    let area_material = RwSignal::new(Some("mulch".to_string()));
    // The depth (inches) of the next drawn area's material — used to cost a
    // per-yd³ material (mulch) by volume. 3 in is a typical mulch depth.
    let area_depth = RwSignal::new(3.0_f64);
    // Placement engine state: the active tool, the nodes placed this gesture,
    // and the previewed next node under the cursor.
    let tool = RwSignal::new(None::<Tool>);
    let placed = RwSignal::new(Vec::<Coord>::new());
    let preview = RwSignal::new(None::<Coord>);
    // True once a Shift-held placement has kept the object tool armed for a
    // "sticky" run; releasing Shift then ends the run (see the keyup effect).
    let sticky_run = RwSignal::new(false);
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
                structure_status: house_status.get(),
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
            shapes: shapes.get(),
            circles: circles.get(),
            ..Default::default()
        });
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

    // Pointer move → drag the held object, rotate toward the cursor while the
    // handle is held, otherwise preview the next node.
    let on_hover = Callback::new(move |raw: Coord| {
        hover_at.set(Some(raw.clone()));
        // Moving a held object: place its center at the cursor plus the grab
        // offset, snapping to the foot grid when grid-snap is on.
        if let Some(d) = dragging.get_untracked() {
            let step = if grid_snap.get_untracked() {
                GRID_STEP
            } else {
                0.0
            };
            let c = dragged_center(&raw, (d.grab_x, d.grab_y), step);
            objects.update(|v| {
                if let Some(o) = v.get_mut(d.index) {
                    o.x = c.x;
                    o.y = c.y;
                }
            });
            return;
        }
        if rotating.get_untracked() {
            if let Some(i) = selected.get_untracked() {
                objects.update(|v| {
                    if let Some(o) = v.get_mut(i) {
                        let mut deg = heading(Point::new(o.x, o.y), Point::new(raw.x, raw.y));
                        if grid_snap.get_untracked() {
                            deg = ((deg / ROT_STEP).round() * ROT_STEP).rem_euclid(360.0);
                        }
                        o.rot = Some(deg);
                    }
                });
            }
            return;
        }
        // Dragging a selected tree's canopy/trunk handle: the new diameter is
        // twice the distance from the tree's center to the cursor — the same
        // "point toward the cursor" simplicity the rotate handle uses above,
        // just read as a radius instead of a heading. Rounded to the nearest
        // tenth of a foot — a tree's size doesn't need pixel-precision decimals.
        if let Some(part) = resizing.get_untracked() {
            if let Some(i) = selected.get_untracked() {
                objects.update(|v| {
                    if let Some(o) = v.get_mut(i) {
                        let radius = Point::new(o.x, o.y).dist(Point::new(raw.x, raw.y));
                        let diameter = (radius * 2.0 * 10.0).round() / 10.0;
                        match part {
                            ResizePart::Canopy => {
                                o.canopy_diameter_ft = Some(diameter.max(MIN_CANOPY_FT));
                            }
                            ResizePart::Trunk => {
                                o.trunk_diameter_ft = Some(diameter.max(MIN_TRUNK_FT));
                            }
                        }
                    }
                });
            }
            return;
        }
        // Dragging a selected shape's node: it follows the cursor directly
        // (no grab offset — a node's position *is* the cursor), snapping to
        // the foot grid when grid-snap is on, same as every other node. Any
        // Bézier control handles attached to the node move with it (the same
        // delta) so a curved edge's tangent doesn't kink.
        if let Some(ni) = dragging_node.get_untracked() {
            if let Some(si) = selected_shape.get_untracked() {
                let at = if grid_snap.get_untracked() {
                    snap_to_grid(&raw, GRID_STEP)
                } else {
                    raw.clone()
                };
                shapes.update(|v| {
                    if let Some(s) = v.get_mut(si)
                        && let Some(old) = s.corners.get(ni).cloned()
                    {
                        let (dx, dy) = (at.x - old.x, at.y - old.y);
                        s.corners[ni] = at;
                        carry_controls(s, ni, dx, dy);
                    }
                });
            }
            return;
        }
        // Dragging a selected shape's edge (bulge) handle: the bulge is the
        // signed perpendicular offset of the cursor from the edge's chord
        // midpoint, as a fraction of the half-chord (`bulge = 2·sagitta/chord`,
        // the inverse of the apex placement in `shapes.rs`). Left of the
        // edge's travel direction is positive, matching the renderer.
        if let Some(ei) = dragging_edge.get_untracked() {
            if let Some(si) = selected_shape.get_untracked() {
                shapes.update(|v| {
                    if let Some(s) = v.get_mut(si)
                        && let Some(b) = edge_bulge_from_cursor(&s.corners, ei, &raw)
                    {
                        set_bulge(&mut s.bulges, s.corners.len(), ei, b);
                    }
                });
            }
            return;
        }
        // Dragging a selected shape's Bézier control handle: move that control
        // point to the cursor (grid-snapped like a node), promoting a still-
        // straight edge to a curve on first drag.
        if let Some((ei, which)) = dragging_control.get_untracked() {
            if let Some(si) = selected_shape.get_untracked() {
                let at = if grid_snap.get_untracked() {
                    snap_to_grid(&raw, GRID_STEP)
                } else {
                    raw.clone()
                };
                shapes.update(|v| {
                    if let Some(s) = v.get_mut(si) {
                        set_shape_control(s, ei, which, at);
                    }
                });
            }
            return;
        }
        // Dragging a selected house corner: moving it only changes wall
        // geometry — each `Wall`/opening re-derives its position live from
        // the corners + its wall index, so doors/windows just follow.
        if let Some(i) = dragging_house_node.get_untracked() {
            let at = if grid_snap.get_untracked() {
                snap_to_grid(&raw, GRID_STEP)
            } else {
                raw.clone()
            };
            corners.update(|v| {
                if let Some(c) = v.get_mut(i) {
                    *c = at;
                }
            });
            return;
        }
        // Dragging a selected deck level's node. Step runs store their own
        // literal coordinates (captured at draw time, not wall-indexed like
        // an opening), so moving a level's corner has no dependent geometry
        // to re-derive.
        if let Some(ni) = dragging_deck_node.get_untracked() {
            if let Some(li) = selected_deck.get_untracked() {
                let at = if grid_snap.get_untracked() {
                    snap_to_grid(&raw, GRID_STEP)
                } else {
                    raw.clone()
                };
                deck.update(|v| {
                    if let Some(lvl) = v.get_mut(li)
                        && let Some(c) = lvl.corners.get_mut(ni)
                    {
                        *c = at;
                    }
                });
            }
            return;
        }
        // Dragging a selected circle's resize handle: the new radius is the
        // distance from its center to the cursor, the same "point toward the
        // cursor" simplicity a tree's canopy/trunk handles use, rounded to
        // the nearest tenth of a foot.
        if circle_resizing.get_untracked() {
            if let Some(i) = selected_circle.get_untracked() {
                circles.update(|v| {
                    if let Some(c) = v.get_mut(i) {
                        let radius =
                            Point::new(c.center.x, c.center.y).dist(Point::new(raw.x, raw.y));
                        let rounded = (radius * 10.0).round() / 10.0;
                        c.radius_ft = rounded.max(MIN_CIRCLE_RADIUS_FT);
                    }
                });
            }
            return;
        }
        if let Some(tl) = tool.get_untracked() {
            preview.set(Some(snap(tl, &raw)));
        }
    });

    // Pointer release → commit a node (or close / finish the object).
    let on_commit = Callback::new(move |(raw, mods): (Coord, Modifiers)| {
        // Releasing after an object move or a rotate-handle drag just ends the
        // gesture (the object is already at its new position / angle).
        if dragging.get_untracked().is_some() {
            dragging.set(None);
            return;
        }
        if rotating.get_untracked() {
            rotating.set(false);
            return;
        }
        if resizing.get_untracked().is_some() {
            resizing.set(None);
            return;
        }
        if dragging_node.get_untracked().is_some() {
            dragging_node.set(None);
            return;
        }
        if dragging_edge.get_untracked().is_some() {
            dragging_edge.set(None);
            return;
        }
        if dragging_control.get_untracked().is_some() {
            dragging_control.set(None);
            return;
        }
        if dragging_house_node.get_untracked().is_some() {
            dragging_house_node.set(None);
            return;
        }
        if dragging_deck_node.get_untracked().is_some() {
            dragging_deck_node.set(None);
            return;
        }
        if circle_resizing.get_untracked() {
            circle_resizing.set(false);
            return;
        }
        if press_only.get_untracked() {
            press_only.set(false);
            return;
        }
        let Some(tl) = tool.get_untracked() else {
            // No tool armed: a click selects the object under the cursor, or
            // clears every selection (object, shape/node, house, deck level,
            // circle) when it lands on empty space. A shape/house/deck-level/
            // circle body or node press is gated out above (`press_only`/the
            // `dragging_*`/`circle_resizing` flags), so this only runs for a
            // click that didn't press any of them.
            selected.set(object_at(
                Point::new(raw.x, raw.y),
                &objects.get_untracked(),
                &catalog.get_untracked(),
            ));
            selected_shape.set(None);
            selected_nodes.set(Vec::new());
            house_selected.set(false);
            selected_deck.set(None);
            selected_circle.set(None);
            return;
        };
        let next = snap(tl, &raw);
        let pl = placed.get_untracked();
        match commit_kind(tl, &pl, &next) {
            Commit::Add => placed.update(|v| v.push(next)),
            Commit::Finish => {
                // The placed nodes become a committed outline: a new deck level
                // (decks are multi-level — additive), a new drawn area (also
                // additive — a plan can have several), or the house outline.
                if tl == Tool::Deck {
                    // `DeckLevel::new` sets `structure_status: existing` — a
                    // freshly-drawn level is presumed already-built, matching
                    // the schema's default.
                    let level = DeckLevel {
                        corners: pl,
                        ..DeckLevel::new(elevation.get_untracked())
                    };
                    deck.update(|v| v.push(level));
                } else if tl == Tool::Shape {
                    // A freshly-drawn area is all straight edges (no bulges/
                    // curves), tagged with the armed area material + depth.
                    let shape = Shape {
                        corners: pl,
                        elevation: area_elevation.get_untracked(),
                        bulges: Vec::new(),
                        curves: Vec::new(),
                        material_ref: area_material.get_untracked(),
                        depth_in: Some(area_depth.get_untracked()),
                    };
                    shapes.update(|v| v.push(shape));
                } else {
                    corners.set(pl);
                }
                reset(tool, placed, preview, sticky_run);
            }
            Commit::FinishWith if tl == Tool::Object => {
                // Drop the armed catalog item at the clicked point. Option/Alt
                // places it as a virtual what-if ghost instead of real.
                let id = selected_id.get_untracked();
                if !id.is_empty() {
                    let mut obj = Object::new(id, next.x, next.y);
                    obj.is_virtual = mods.alt;
                    objects.update(|v| v.push(obj));
                }
                // Shift keeps the tool armed for another placement (a "sticky"
                // run); the keyup effect below ends the run when Shift is
                // released. Otherwise this placement is one-shot, as before.
                if mods.shift {
                    placed.set(Vec::new());
                    preview.set(None);
                    sticky_run.set(true);
                } else {
                    reset(tool, placed, preview, sticky_run);
                }
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
                reset(tool, placed, preview, sticky_run);
            }
            Commit::FinishWith if tl == Tool::Circle => {
                // The first node is the center; the radius is its distance to
                // the second (the release point that just finished the gesture).
                if let Some(center) = pl.first() {
                    let radius = Point::new(center.x, center.y).dist(Point::new(next.x, next.y));
                    circles.update(|v| {
                        v.push(Circle {
                            material_ref: area_material.get_untracked(),
                            depth_in: Some(area_depth.get_untracked()),
                            ..Circle::new(
                                Box::new(center.clone()),
                                area_elevation.get_untracked(),
                                radius,
                            )
                        });
                    });
                }
                reset(tool, placed, preview, sticky_run);
            }
            Commit::FinishWith => {
                if let (Some(kind), Some(start)) = (tl.opening_kind(), pl.first())
                    && let Some(o) =
                        opening_from_nodes(&corners.get_untracked(), kind, start, &next)
                {
                    openings.update(|v| v.push(o));
                }
                reset(tool, placed, preview, sticky_run);
            }
        }
    });

    let on_leave = Callback::new(move |()| preview.set(None));

    // Every "selected thing" (object, shape, house, deck level, circle) is a
    // separate signal rather than one `Selection` enum — but that means
    // selecting one must clear all the others, so it's centralized here
    // instead of repeated at every press site.
    let clear_selection = move || {
        selected.set(None);
        selected_shape.set(None);
        selected_nodes.set(Vec::new());
        house_selected.set(false);
        selected_deck.set(None);
        selected_circle.set(None);
    };

    // Press an object's body → select it and start a move drag, grabbing it at
    // the offset from the cursor to its center so it doesn't jump under the
    // pointer. Ignored while a drawing tool is armed — that click is a placement.
    let on_object_press = Callback::new(move |i: usize| {
        if tool.get_untracked().is_some() {
            return;
        }
        clear_selection();
        selected.set(Some(i));
        let (grab_x, grab_y) = match (hover_at.get_untracked(), objects.get_untracked().get(i)) {
            (Some(c), Some(o)) => (o.x - c.x, o.y - c.y),
            _ => (0.0, 0.0),
        };
        dragging.set(Some(Drag {
            index: i,
            grab_x,
            grab_y,
        }));
    });

    // Press a shape's body → select it, resetting any node selection (a fresh
    // start on this shape). Ignored while a drawing tool is armed. Doesn't
    // start a drag — F3.1 only moves *nodes*, not a whole shape — so
    // `press_only` just gates `on_commit`'s empty-space fallback from
    // clearing the selection this same click just set.
    let on_shape_press = Callback::new(move |i: usize| {
        if tool.get_untracked().is_some() {
            return;
        }
        clear_selection();
        selected_shape.set(Some(i));
        press_only.set(true);
    });

    // Press the house's body → select it (same press-only, no-drag pattern as
    // a shape body). Its corners then render as interactive node handles.
    let on_house_press = Callback::new(move |()| {
        if tool.get_untracked().is_some() {
            return;
        }
        clear_selection();
        house_selected.set(true);
        press_only.set(true);
    });

    // Press a selected house's node handle → start a move drag. Moving a
    // corner only changes wall geometry; each `Wall`/opening re-derives its
    // position live, so doors/windows just follow (see `house.rs`).
    let on_house_node_press = Callback::new(move |i: usize| {
        if tool.get_untracked().is_some() {
            return;
        }
        dragging_house_node.set(Some(i));
    });

    // Press a deck level's body (by its original index) → select that level.
    let on_deck_level_press = Callback::new(move |i: usize| {
        if tool.get_untracked().is_some() {
            return;
        }
        clear_selection();
        selected_deck.set(Some(i));
        press_only.set(true);
    });

    // Press the selected deck level's node handle → start a move drag. Step
    // runs store their own literal coordinates (not wall-indexed like an
    // opening), so there's no dependent geometry to re-derive.
    let on_deck_node_press = Callback::new(move |ni: usize| {
        if tool.get_untracked().is_some() {
            return;
        }
        dragging_deck_node.set(Some(ni));
    });

    // Press a circle's body (by its index) → select it. Same press-only,
    // no-drag pattern as a shape/house/deck-level body.
    let on_circle_press = Callback::new(move |i: usize| {
        if tool.get_untracked().is_some() {
            return;
        }
        clear_selection();
        selected_circle.set(Some(i));
        press_only.set(true);
    });

    // Press the selected circle's resize handle → start a resize drag.
    let on_circle_handle_press = Callback::new(move |()| {
        if tool.get_untracked().is_some() {
            return;
        }
        circle_resizing.set(true);
    });

    // Press a selected shape's node → select it and start a move drag. A
    // second press on an *adjacent* node adds it (arming the insert-between
    // popup); any other press (a non-adjacent node, or a third node once a
    // pair is already selected) resets the selection to just that node.
    let on_node_press = Callback::new(move |ni: usize| {
        if tool.get_untracked().is_some() {
            return;
        }
        let Some(si) = selected_shape.get_untracked() else {
            return;
        };
        let len = shapes
            .get_untracked()
            .get(si)
            .map_or(0, |s| s.corners.len());
        selected_nodes.update(|v| match v.as_slice() {
            [a] if *a != ni && are_adjacent(len, *a, ni) => *v = vec![*a, ni],
            [a] if *a == ni => {}
            _ => *v = vec![ni],
        });
        dragging_node.set(Some(ni));
    });

    // Press a selected shape's edge (bulge) handle → start a drag that bows
    // that edge into an arc.
    let on_edge_press = Callback::new(move |ei: usize| {
        if tool.get_untracked().is_some() {
            return;
        }
        dragging_edge.set(Some(ei));
    });

    // Press a selected shape's Bézier control handle → start a drag that
    // curves that edge (promoting a straight edge on first move).
    let on_control_press = Callback::new(move |ec: (usize, usize)| {
        if tool.get_untracked().is_some() {
            return;
        }
        dragging_control.set(Some(ec));
    });

    // The insert-between popup's "Insert" button: split the edge between the
    // two selected nodes with a new one at its midpoint.
    let on_insert_node = Callback::new(move |()| {
        // Gates `on_commit`'s empty-space fallback, same as a shape body
        // press — this button has no drag of its own to consume the click.
        press_only.set(true);
        if let (Some(si), [a, b]) = (
            selected_shape.get_untracked(),
            selected_nodes.get_untracked().as_slice(),
        ) {
            let (a, b) = (*a, *b);
            shapes.update(|v| {
                if let Some(s) = v.get_mut(si)
                    && let Some(corners) = insert_node_between(&s.corners, a, b)
                {
                    s.corners = corners;
                    // Inserting a node changes the edge count, which would
                    // misalign the per-edge `bulges` / edge-indexed `curves`.
                    // Curve-aware re-indexing (split the arc/Bézier at the new
                    // node) is deferred (see F3-draw-edit-shapes.md), so for
                    // now editing the node ring resets its edges to straight —
                    // safe, never a misrendered arc/curve.
                    s.bulges.clear();
                    s.curves.clear();
                }
            });
        }
        selected_nodes.set(Vec::new());
    });

    // The insert-between popup's "Cancel" button: just deselect both nodes.
    let on_cancel_nodes = Callback::new(move |()| {
        press_only.set(true);
        selected_nodes.set(Vec::new());
    });

    // Remove the selected object and clear the selection.
    let delete_selected = Callback::new(move |()| {
        if let Some(i) = selected.get_untracked() {
            objects.update(|v| {
                if i < v.len() {
                    v.remove(i);
                }
            });
            selected.set(None);
        }
    });

    // The area inspector's edit callbacks act on whichever drawn area is
    // selected — a `Shape` (boundary) or a `Circle` — since only one can be
    // selected at a time.
    let set_area_elevation = Callback::new(move |v: f64| {
        if let Some(i) = selected_shape.get_untracked() {
            shapes.update(|list| {
                if let Some(s) = list.get_mut(i) {
                    s.elevation = v;
                }
            });
        } else if let Some(i) = selected_circle.get_untracked() {
            circles.update(|list| {
                if let Some(c) = list.get_mut(i) {
                    c.elevation = v;
                }
            });
        }
    });
    let set_area_depth = Callback::new(move |v: f64| {
        if let Some(i) = selected_shape.get_untracked() {
            shapes.update(|list| {
                if let Some(s) = list.get_mut(i) {
                    s.depth_in = Some(v);
                }
            });
        } else if let Some(i) = selected_circle.get_untracked() {
            circles.update(|list| {
                if let Some(c) = list.get_mut(i) {
                    c.depth_in = Some(v);
                }
            });
        }
    });
    let delete_selected_area = Callback::new(move |()| {
        if let Some(i) = selected_shape.get_untracked() {
            shapes.update(|v| {
                if i < v.len() {
                    v.remove(i);
                }
            });
            selected_shape.set(None);
        } else if let Some(i) = selected_circle.get_untracked() {
            circles.update(|v| {
                if i < v.len() {
                    v.remove(i);
                }
            });
            selected_circle.set(None);
        }
    });

    // Structure (house / deck level) inspector edits. The house has one status
    // and no elevation; a deck level has both.
    let set_house_status = Callback::new(move |s: ItemStatus| house_status.set(s));
    let delete_house = Callback::new(move |()| {
        corners.set(Vec::new());
        openings.set(Vec::new());
        house_selected.set(false);
    });
    let set_deck_elevation = Callback::new(move |v: f64| {
        if let Some(i) = selected_deck.get_untracked() {
            deck.update(|levels| {
                if let Some(l) = levels.get_mut(i) {
                    l.elevation = v;
                }
            });
        }
    });
    let set_deck_status = Callback::new(move |s: ItemStatus| {
        if let Some(i) = selected_deck.get_untracked() {
            deck.update(|levels| {
                if let Some(l) = levels.get_mut(i) {
                    l.structure_status = s;
                }
            });
        }
    });
    let delete_deck_level = Callback::new(move |()| {
        if let Some(i) = selected_deck.get_untracked() {
            deck.update(|levels| {
                if i < levels.len() {
                    levels.remove(i);
                }
            });
            selected_deck.set(None);
        }
    });

    // Catalog inspector: select an item to edit, close the panel, and edit the
    // selected item's fields by id. Objects reference their catalog item by
    // `catalog_ref` (not a copy), so every edit reprices and re-renders every
    // object placed from it, live.
    let select_catalog_item = Callback::new(move |id: String| catalog_selected.set(Some(id)));
    let close_catalog = Callback::new(move |()| catalog_open.set(false));
    // Apply `edit` to the catalog item currently selected in the panel.
    let edit_selected_catalog = move |edit: &dyn Fn(&mut CatalogItem)| {
        if let Some(id) = catalog_selected.get_untracked() {
            catalog.update(|list| {
                if let Some(c) = list.iter_mut().find(|c| c.id == id) {
                    edit(c);
                }
            });
        }
    };
    // An empty name/category clears the field (back to a fallback / uncategorized)
    // rather than storing a blank string.
    let set_catalog_name =
        Callback::new(move |v: String| edit_selected_catalog(&|c| c.name = non_empty(&v)));
    let set_catalog_category =
        Callback::new(move |v: String| edit_selected_catalog(&|c| c.category = non_empty(&v)));
    let set_catalog_price =
        Callback::new(move |v: f64| edit_selected_catalog(&|c| c.unit_price = Some(v)));
    let set_catalog_width =
        Callback::new(move |v: f64| edit_selected_catalog(&|c| c.width_ft = Some(v)));
    let set_catalog_depth =
        Callback::new(move |v: f64| edit_selected_catalog(&|c| c.depth_ft = Some(v)));
    let set_catalog_height =
        Callback::new(move |v: f64| edit_selected_catalog(&|c| c.height_ft = Some(v)));
    let set_catalog_price_unit =
        Callback::new(move |u: PriceUnit| edit_selected_catalog(&|c| c.price_unit = u.clone()));
    // Hand-add a blank catalog item (a new material to author), give it the
    // first free `material-N` id, and select it for editing.
    let add_material = Callback::new(move |()| {
        let id = catalog.with_untracked(|list| {
            // `list.len() + 1` distinct candidates guarantees a free one (only
            // `list.len()` ids are taken), so the search is bounded.
            (1..=list.len() + 1)
                .map(|n| format!("material-{n}"))
                .find(|id| !list.iter().any(|c| c.id == *id))
                .unwrap_or_else(|| "material".to_string())
        });
        let mut item = CatalogItem::new(id.clone());
        item.name = Some("New material".to_string());
        item.category = Some("material".to_string());
        item.unit_price = Some(0.0);
        // A material is measured by volume/area, not placed as an object — so it
        // just lives in the catalog for a course/area to reference, never a
        // per-item palette tile. Default to bulk (per yd³); editable via the
        // price-unit control. (Per-item would wrongly make it placeable.)
        item.price_unit = PriceUnit::per_cubic_yard;
        catalog.update(|list| list.push(item));
        catalog_selected.set(Some(id));
    });

    // Remove the selected shape's selected node (refused, per `delete_node`,
    // if it would leave the shape below its 3-node drawable minimum).
    let delete_selected_node = Callback::new(move |()| {
        if let (Some(si), [ni]) = (
            selected_shape.get_untracked(),
            selected_nodes.get_untracked().as_slice(),
        ) {
            let ni = *ni;
            shapes.update(|v| {
                if let Some(s) = v.get_mut(si)
                    && let Some(corners) = delete_node(&s.corners, ni)
                {
                    s.corners = corners;
                    // Deleting a node changes the edge count — reset the
                    // per-edge bulges/curves to straight (see the insert
                    // callback).
                    s.bulges.clear();
                    s.curves.clear();
                }
            });
            selected_nodes.set(Vec::new());
        }
    });

    // Delete / Backspace removes the selected object, or (if no object is
    // selected but exactly one shape node is) that node — but not while a
    // text field or picker is focused, so it can't eat a keypress meant for
    // editing.
    #[cfg(feature = "csr")]
    Effect::new(move |_| {
        let handle =
            window_event_listener(leptos::ev::keydown, move |ev: leptos::ev::KeyboardEvent| {
                if is_editing_field() {
                    return;
                }
                let key = ev.key();
                if key != "Delete" && key != "Backspace" {
                    return;
                }
                if selected.get_untracked().is_some() {
                    ev.prevent_default();
                    delete_selected.run(());
                } else if selected_nodes.get_untracked().len() == 1 {
                    ev.prevent_default();
                    delete_selected_node.run(());
                }
            });
        on_cleanup(move || handle.remove());
    });

    // Releasing Shift ends an in-progress sticky placement run (see the
    // Object commit branch below) — the object tool disarms the instant the
    // key comes up, with no Esc needed.
    #[cfg(feature = "csr")]
    Effect::new(move |_| {
        let handle =
            window_event_listener(leptos::ev::keyup, move |ev: leptos::ev::KeyboardEvent| {
                if ev.key() == "Shift" && sticky_run.get_untracked() {
                    sticky_run.set(false);
                    reset(tool, placed, preview, sticky_run);
                }
            });
        on_cleanup(move || handle.remove());
    });

    // Escape cancels the armed tool (a palette tile or a drawing tool) without
    // placing anything — the keyboard equivalent of clicking it again.
    #[cfg(feature = "csr")]
    Effect::new(move |_| {
        let handle =
            window_event_listener(leptos::ev::keydown, move |ev: leptos::ev::KeyboardEvent| {
                if ev.key() == "Escape" && tool.get_untracked().is_some() && !is_editing_field() {
                    reset(tool, placed, preview, sticky_run);
                }
            });
        on_cleanup(move || handle.remove());
    });

    // Arm a tool (or toggle it off). Starting the house clears the old one;
    // starting an opening keeps the house.
    let pick_tool = move |t: Tool| {
        if tool.get_untracked() == Some(t) {
            reset(tool, placed, preview, sticky_run);
            return;
        }
        // Redrawing the house replaces it; decks are additive (multi-level).
        if t == Tool::House {
            corners.set(Vec::new());
            openings.set(Vec::new());
        }
        placed.set(Vec::new());
        preview.set(None);
        clear_selection();
        tool.set(Some(t));
    };

    // One callback the tool buttons share; per-button derivations live in tool_btn.
    let pick = Callback::new(pick_tool);

    // The armed catalog item id, if the object tool is active — drives which
    // palette tile highlights.
    let armed =
        Signal::derive(move || (tool.get() == Some(Tool::Object)).then(|| selected_id.get()));

    // The armed item's footprint, if any — drives the placement preview ghost
    // (a shape-aware outline instead of a plain node marker).
    let armed_footprint = Signal::derive(move || {
        let id = armed.get()?;
        catalog.get().iter().find(|c| c.id == id).map(Footprint::of)
    });

    // Click a palette tile → arm that item for placement (click the armed tile
    // again to disarm). Arming the object tool clears any current selection and
    // the in-progress placement, like picking a drawing tool.
    let pick_object = Callback::new(move |id: String| {
        let already_armed =
            tool.get_untracked() == Some(Tool::Object) && selected_id.get_untracked() == id;
        if already_armed {
            reset(tool, placed, preview, sticky_run);
            return;
        }
        selected_id.set(id);
        placed.set(Vec::new());
        preview.set(None);
        clear_selection();
        sticky_run.set(false);
        tool.set(Some(Tool::Object));
    });

    // The live bill of materials — recomputed whenever the objects, drawn
    // areas, or catalog change, so the estimate panel reacts as furniture is
    // placed/removed and as mulch beds (and later paver areas) are drawn.
    let bom = Signal::derive(move || {
        take_off(&Plan {
            catalog: catalog.get(),
            objects: objects.get(),
            shapes: shapes.get(),
            circles: circles.get(),
            ..Default::default()
        })
    });

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
            // A drawn area is tagged with the armed material (mulch or paver);
            // that drives its look (mulch brown vs. paver gray) and how it's
            // costed (mulch per yd³ by depth, pavers per ft²). The
            // `draw-shape`/`draw-circle` tools draw either as a boundary or a
            // circle; more materials join the picker as their stories land.
            <ToolGroup label="Area">
                {material_btn(area_material, "mulch", "Mulch", "area-mat-mulch")}
                {material_btn(area_material, "paver", "Pavers", "area-mat-paver")}
                {tool_btn(tool, pick, Tool::Shape, "Draw area", "draw-shape")}
                {tool_btn(tool, pick, Tool::Circle, "Round area", "draw-circle")}
                <NumberField
                    label="Depth (in)"
                    testid="area-depth"
                    value=area_depth
                    on_input=Callback::new(move |v| area_depth.set(v))
                    step=1.0
                />
                <NumberField
                    label="Elev (ft)"
                    testid="area-elevation"
                    value=area_elevation
                    on_input=Callback::new(move |v| area_elevation.set(v))
                    step=0.5
                />
            </ToolGroup>
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
            <ToolGroup label="Catalog">
                <ToolButton
                    label="Edit catalog"
                    testid="edit-catalog"
                    active=Signal::derive(move || catalog_open.get())
                    on_pick=Callback::new(move |()| catalog_open.update(|o| *o = !*o))
                />
            </ToolGroup>
        </div>
        // The object palette appears once there's a catalog (seeded on load):
        // click a tile to arm it, then click the canvas to place.
        {move || {
            (!catalog.get().is_empty())
                .then(|| view! { <ObjectPalette catalog=catalog armed=armed on_pick=pick_object /> })
        }}
        <p class="hint" data-testid="hint">{move || hint(tool.get())}</p>
        <div class="stage">
            <div class="canvas">
                // Recreate the canvas only when the yard size changes; the plan,
                // the placement, and the preview are read reactively inside Yard,
                // so the <svg> persists during a pointer gesture.
                {move || {
                    view! {
                        <Yard
                            yard_w=width.get()
                            yard_d=depth.get()
                            px_ft=PX_FT
                            pad=PAD
                            house=corners
                            house_selected=house_selected
                            deck=deck
                            selected_deck=selected_deck
                            steps=steps
                            shapes=shapes
                            selected_shape=selected_shape
                            selected_nodes=selected_nodes
                            circles=circles
                            selected_circle=selected_circle
                            openings=openings
                            objects=objects
                            catalog=catalog
                            selected=selected
                            placed=placed
                            preview=preview
                            object_footprint=armed_footprint
                            on_hover=on_hover
                            on_commit=on_commit
                            on_leave=on_leave
                            on_metrics=Callback::new(move |m| metrics.set(m))
                            on_handle_press=Callback::new(move |()| rotating.set(true))
                            on_canopy_handle_press=Callback::new(move |()| {
                                resizing.set(Some(ResizePart::Canopy));
                            })
                            on_trunk_handle_press=Callback::new(move |()| {
                                resizing.set(Some(ResizePart::Trunk));
                            })
                            on_object_press=on_object_press
                            on_shape_press=on_shape_press
                            on_node_press=on_node_press
                            on_insert_node=on_insert_node
                            on_cancel_nodes=on_cancel_nodes
                            on_edge_press=on_edge_press
                            on_control_press=on_control_press
                            on_house_press=on_house_press
                            on_house_node_press=on_house_node_press
                            on_deck_level_press=on_deck_level_press
                            on_deck_node_press=on_deck_node_press
                            on_circle_press=on_circle_press
                            on_circle_handle_press=on_circle_handle_press
                        />
                    }
                }}
                // When an object is selected, float its inspector in the first
                // empty corner of the canvas.
                {move || {
                    let objs = objects.get();
                    let i = selected.get()?;
                    let object = objs.get(i)?.clone();
                    let item = catalog.get().iter().find(|c| c.id == object.catalog_ref).cloned();
                    let m = metrics.get();
                    // Avoid every placed/drawn feature — house, deck, objects,
                    // and drawn areas (shapes + circles).
                    let points = content_points(
                        &corners.get(),
                        &deck.get(),
                        &objs,
                        &shapes.get(),
                        &circles.get(),
                    );
                    let (corner, style) =
                        inspector_placement(&points, width.get(), depth.get(), &m);
                    Some(
                        view! {
                            <ObjectInspector
                                object=object
                                item=item
                                corner=corner
                                style=style
                                on_status=Callback::new(move |s| {
                                    if let Some(i) = selected.get_untracked() {
                                        objects
                                            .update(|v| {
                                                if let Some(o) = v.get_mut(i) {
                                                    o.status = s;
                                                }
                                            });
                                    }
                                })
                                on_virtual=Callback::new(move |is_virtual| {
                                    if let Some(i) = selected.get_untracked() {
                                        objects
                                            .update(|v| {
                                                if let Some(o) = v.get_mut(i) {
                                                    o.is_virtual = is_virtual;
                                                }
                                            });
                                    }
                                })
                                on_reset_rotation=Callback::new(move |()| {
                                    if let Some(i) = selected.get_untracked() {
                                        objects
                                            .update(|v| {
                                                if let Some(o) = v.get_mut(i) {
                                                    o.rot = Some(0.0);
                                                }
                                            });
                                    }
                                })
                                on_canopy_diameter=Callback::new(move |d| {
                                    if let Some(i) = selected.get_untracked() {
                                        objects
                                            .update(|v| {
                                                if let Some(o) = v.get_mut(i) {
                                                    o.canopy_diameter_ft = Some(d);
                                                }
                                            });
                                    }
                                })
                                on_trunk_diameter=Callback::new(move |d| {
                                    if let Some(i) = selected.get_untracked() {
                                        objects
                                            .update(|v| {
                                                if let Some(o) = v.get_mut(i) {
                                                    o.trunk_diameter_ft = Some(d);
                                                }
                                            });
                                    }
                                })
                                on_delete=delete_selected
                            />
                        },
                    )
                }}
                // When a drawn area (a mulch bed / paver patio) is selected,
                // float its inspector in the first empty corner.
                {move || {
                    let area = selected_area(
                        selected_shape.get(),
                        selected_circle.get(),
                        &shapes.get(),
                        &circles.get(),
                        &catalog.get(),
                    )?;
                    let m = metrics.get();
                    // Avoid every placed/drawn feature (including other areas),
                    // exactly like the object inspector.
                    let points = content_points(
                        &corners.get(),
                        &deck.get(),
                        &objects.get(),
                        &shapes.get(),
                        &circles.get(),
                    );
                    let (corner, style) =
                        inspector_placement(&points, width.get(), depth.get(), &m);
                    Some(
                        view! {
                            <AreaInspector
                                title=area.title
                                category=area.category
                                area_ft2=area.area_ft2
                                elevation=area.elevation
                                depth=area.depth
                                show_depth=area.show_depth
                                cost=area.cost
                                corner=corner
                                style=style
                                on_elevation=set_area_elevation
                                on_depth=set_area_depth
                                on_delete=delete_selected_area
                            />
                        },
                    )
                }}
                // When the house is selected, float its inspector — its
                // structure status + footprint (it sits at grade, no elevation).
                {move || {
                    if !house_selected.get() {
                        return None;
                    }
                    let cs = corners.get();
                    if cs.is_empty() {
                        return None;
                    }
                    let area_ft2 = boundary_area(&cs, &[], &[]);
                    let m = metrics.get();
                    let points = content_points(
                        &cs,
                        &deck.get(),
                        &objects.get(),
                        &shapes.get(),
                        &circles.get(),
                    );
                    let (corner, style) =
                        inspector_placement(&points, width.get(), depth.get(), &m);
                    Some(
                        view! {
                            <AreaInspector
                                title="House"
                                area_ft2=area_ft2
                                elevation=0.0
                                show_elevation=false
                                depth=0.0
                                status=Some(house_status.get())
                                corner=corner
                                style=style
                                on_elevation=Callback::new(|_| {})
                                on_depth=Callback::new(|_| {})
                                on_status=set_house_status
                                on_delete=delete_house
                            />
                        },
                    )
                }}
                // When a deck level is selected, float its inspector — status,
                // elevation, and footprint.
                {move || {
                    let i = selected_deck.get()?;
                    let levels = deck.get();
                    let level = levels.get(i)?;
                    let area_ft2 = boundary_area(&level.corners, &[], &[]);
                    let elevation = level.elevation;
                    let status = level.structure_status.clone();
                    let m = metrics.get();
                    let points = content_points(
                        &corners.get(),
                        &levels,
                        &objects.get(),
                        &shapes.get(),
                        &circles.get(),
                    );
                    let (corner, style) =
                        inspector_placement(&points, width.get(), depth.get(), &m);
                    Some(
                        view! {
                            <AreaInspector
                                title="Deck"
                                area_ft2=area_ft2
                                elevation=elevation
                                depth=0.0
                                status=Some(status)
                                corner=corner
                                style=style
                                on_elevation=set_deck_elevation
                                on_depth=Callback::new(|_| {})
                                on_status=set_deck_status
                                on_delete=delete_deck_level
                            />
                        },
                    )
                }}
            </div>
            // The estimate appears alongside the canvas once there's a catalog.
            {move || { (!catalog.get().is_empty()).then(|| view! { <EstimatePanel bom=bom /> }) }}
            // The catalog inspector, when opened from the toolbar.
            {move || {
                catalog_open.get().then(|| {
                    view! {
                        <CatalogPanel
                            catalog=catalog
                            selected=catalog_selected
                            on_select=select_catalog_item
                            on_name=set_catalog_name
                            on_category=set_catalog_category
                            on_price=set_catalog_price
                            on_price_unit=set_catalog_price_unit
                            on_add=add_material
                            on_width=set_catalog_width
                            on_depth=set_catalog_depth
                            on_height=set_catalog_height
                            on_close=close_catalog
                        />
                    }
                })
            }}
        </div>
    }
}

/// The corner + inline `top/left` style for a floating inspector window:
/// picks the first empty yard corner (avoiding `avoid` content points, sized
/// to the window via the measured px-per-foot) and positions the window inside
/// the grid's screen rect. Shared by the object and area inspectors.
fn inspector_placement(
    avoid: &[Point],
    yard_w: f64,
    yard_d: f64,
    m: &CanvasMetrics,
) -> (Corner, String) {
    let corner = if m.px_ft > 0.0 {
        free_corner(
            avoid,
            yard_w,
            yard_d,
            INSPECTOR_W_PX / m.px_ft,
            INSPECTOR_H_PX / m.px_ft,
        )
    } else {
        Corner::Nw
    };
    let mgn = INSPECTOR_MARGIN_PX;
    let grid_w = yard_w * m.px_ft;
    let grid_h = yard_d * m.px_ft;
    let (left_edge, right_edge) = (m.left + mgn, m.left + grid_w - INSPECTOR_W_PX - mgn);
    let (top_edge, bottom_edge) = (m.top + mgn, m.top + grid_h - INSPECTOR_H_PX - mgn);
    let (top, left) = match corner {
        Corner::Nw => (top_edge, left_edge),
        Corner::Ne => (top_edge, right_edge),
        Corner::Sw => (bottom_edge, left_edge),
        Corner::Se => (bottom_edge, right_edge),
    };
    (corner, format!("top: {top}px; left: {left}px;"))
}

/// The display data the area inspector needs for the currently selected drawn
/// area, resolved from its material through the catalog.
struct AreaInfo {
    title: String,
    category: Option<String>,
    area_ft2: f64,
    elevation: f64,
    depth: f64,
    /// Show the editable depth field — true for a volume-priced material.
    show_depth: bool,
    /// This area's material cost, if priced.
    cost: Option<f64>,
}

/// Resolve the selected `Shape`/`Circle` (only one can be selected) into the
/// area inspector's display data: material name/category, enclosed area, cost
/// (per its material's `price_unit`), and its editable elevation/depth. `None`
/// when nothing is selected.
fn selected_area(
    sel_shape: Option<usize>,
    sel_circle: Option<usize>,
    shapes: &[Shape],
    circles: &[Circle],
    catalog: &[CatalogItem],
) -> Option<AreaInfo> {
    let (material_ref, elevation, depth, area_ft2) = if let Some(i) = sel_shape {
        let s = shapes.get(i)?;
        (
            s.material_ref.as_deref(),
            s.elevation,
            s.depth_in.unwrap_or(0.0),
            shape_area_ft2(s),
        )
    } else if let Some(i) = sel_circle {
        let c = circles.get(i)?;
        (
            c.material_ref.as_deref(),
            c.elevation,
            c.depth_in.unwrap_or(0.0),
            circle_area(c.radius_ft),
        )
    } else {
        return None;
    };
    let item = material_ref.and_then(|m| catalog.iter().find(|c| c.id == m));
    let title = item
        .and_then(|i| i.name.clone())
        .unwrap_or_else(|| "Area".to_string());
    let category = item.and_then(|i| i.category.clone());
    let price_unit = item.map(|i| i.price_unit.clone());
    let unit_price = item.and_then(|i| i.unit_price);
    let show_depth = price_unit == Some(PriceUnit::per_cubic_yard);
    // Cost of this one area, by its material's pricing: per-ft² of surface, or
    // per-yd³ of volume at its depth (`yd³ = ft²·in/324`). A surface material
    // (pavers) adds the cost of its base + bedding courses beneath this area,
    // so the panel shows the area's all-in cost — mirroring the itemized
    // pavers/gravel/sand lines in the estimate.
    let cost = match (&price_unit, unit_price) {
        (Some(PriceUnit::per_square_foot), Some(p)) => {
            let mut total = area_ft2 * p;
            if let Some(i) = item {
                total += course_cost(
                    catalog,
                    i.base_material_ref.as_deref(),
                    i.base_depth_in,
                    area_ft2,
                );
                total += course_cost(
                    catalog,
                    i.bedding_material_ref.as_deref(),
                    i.bedding_depth_in,
                    area_ft2,
                );
            }
            Some(total)
        }
        (Some(PriceUnit::per_cubic_yard), Some(p)) => Some(area_ft2 * depth / 324.0 * p),
        _ => None,
    };
    Some(AreaInfo {
        title,
        category,
        area_ft2,
        elevation,
        depth,
        show_depth,
        cost,
    })
}

/// The dollar cost of a sub-base course (gravel base / bedding sand) beneath a
/// `area_ft2` surface: `yd³ = ft²·depth_in/324` at the referenced material's
/// price. Zero when the surface names no such course, or it isn't in the
/// catalog / has no price.
fn course_cost(
    catalog: &[CatalogItem],
    material_ref: Option<&str>,
    depth_in: Option<f64>,
    area_ft2: f64,
) -> f64 {
    let Some(id) = material_ref else {
        return 0.0;
    };
    let Some(price) = catalog
        .iter()
        .find(|c| c.id == id)
        .and_then(|c| c.unit_price)
    else {
        return 0.0;
    };
    area_ft2 * depth_in.unwrap_or(0.0) / 324.0 * price
}

/// A shape's enclosed area (ft²), accounting for any arc/curve edges — the UI
/// counterpart of `slp_core`'s internal shape-area helper.
fn shape_area_ft2(s: &Shape) -> f64 {
    let curves: Vec<(usize, Point, Point)> = s
        .curves
        .iter()
        .filter_map(|c| {
            usize::try_from(c.edge).ok().map(|e| {
                (
                    e,
                    Point::new(c.control1.x, c.control1.y),
                    Point::new(c.control2.x, c.control2.y),
                )
            })
        })
        .collect();
    boundary_area(&s.corners, &s.bulges, &curves)
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

/// A material-picker toggle: arms `area_material` to the catalog id `id`, so
/// the next drawn area is tagged with (and looks/costs like) that material.
/// Highlights when it's the armed material.
fn material_btn(
    area_material: RwSignal<Option<String>>,
    id: &'static str,
    label: &'static str,
    testid: &'static str,
) -> impl IntoView {
    let active = Signal::derive(move || area_material.get().as_deref() == Some(id));
    view! {
        <ToolButton
            label=label
            testid=testid
            active=active
            on_pick=Callback::new(move |()| area_material.set(Some(id.to_string())))
        />
    }
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

/// The bulge for edge `ei` (node `ei`→next) implied by the cursor at `raw`:
/// the signed perpendicular offset of the cursor from the edge's chord
/// midpoint, as a fraction of the half-chord (`bulge = 2·sagitta/chord`).
/// Positive is left of the edge's travel direction (matching the renderer).
/// `None` for a degenerate (zero-length) edge. Clamped to a sane range and
/// rounded so a dragged arc lands on tidy curvature values.
fn edge_bulge_from_cursor(corners: &[Coord], ei: usize, raw: &Coord) -> Option<f64> {
    let n = corners.len();
    let from = &corners[ei];
    let to = &corners[(ei + 1) % n];
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let chord = dx.hypot(dy);
    if chord < 1e-9 {
        return None;
    }
    let (mx, my) = (from.x.midpoint(to.x), from.y.midpoint(to.y));
    // Left normal of `from`→`to`; the signed perpendicular offset is the sagitta.
    let (nx, ny) = (-dy / chord, dx / chord);
    let sagitta = (raw.x - mx).mul_add(nx, (raw.y - my) * ny);
    let bulge = 2.0 * sagitta / chord;
    // Clamp to a major-arc-ish range and round to a tidy step (0.05).
    let clamped = bulge.clamp(-BULGE_LIMIT, BULGE_LIMIT);
    Some((clamped / BULGE_STEP).round() * BULGE_STEP)
}

/// Set edge `ei`'s bulge in `bulges`, growing it to `edge_count` (padding with
/// zeros) so the parallel per-edge array stays aligned with the corner ring.
fn set_bulge(bulges: &mut Vec<f64>, edge_count: usize, ei: usize, bulge: f64) {
    if ei >= edge_count {
        return;
    }
    if bulges.len() < edge_count {
        bulges.resize(edge_count, 0.0);
    }
    bulges[ei] = bulge;
}

/// Set control `which` (0 = control1, 1 = control2) of edge `ei`'s Bézier to
/// `at`, promoting a still-straight edge to a curve: a new `CurveEdge` is
/// created with both controls at the chord's thirds (so the untouched control
/// starts sensibly), any arc bulge on that edge is cleared (a curve wins), and
/// then the dragged control is overwritten.
fn set_shape_control(shape: &mut Shape, ei: usize, which: usize, at: Coord) {
    let n = shape.corners.len();
    if ei >= n {
        return;
    }
    if !shape
        .curves
        .iter()
        .any(|c| usize::try_from(c.edge) == Ok(ei))
    {
        let from = &shape.corners[ei];
        let to = &shape.corners[(ei + 1) % n];
        let third = |t: f64| Coord::new(from.x + t * (to.x - from.x), from.y + t * (to.y - from.y));
        shape.curves.push(CurveEdge {
            edge: i64::try_from(ei).unwrap_or(0),
            control1: Box::new(third(1.0 / 3.0)),
            control2: Box::new(third(2.0 / 3.0)),
        });
        if let Some(b) = shape.bulges.get_mut(ei) {
            *b = 0.0;
        }
    }
    if let Some(c) = shape
        .curves
        .iter_mut()
        .find(|c| usize::try_from(c.edge) == Ok(ei))
    {
        if which == 0 {
            *c.control1 = at;
        } else {
            *c.control2 = at;
        }
    }
}

/// Translate the Bézier control points attached to node `ni` by `(dx, dy)`,
/// so a moved corner carries its curve handles and the curve doesn't kink.
/// The controls near node `ni` are `control1` of edge `ni` (which starts at
/// `ni`) and `control2` of the previous edge (which ends at `ni`).
fn carry_controls(shape: &mut Shape, ni: usize, dx: f64, dy: f64) {
    let n = shape.corners.len();
    if n == 0 {
        return;
    }
    let prev = (ni + n - 1) % n;
    for c in &mut shape.curves {
        let edge = usize::try_from(c.edge).ok();
        if edge == Some(ni) {
            c.control1.x += dx;
            c.control1.y += dy;
        }
        if edge == Some(prev) {
            c.control2.x += dx;
            c.control2.y += dy;
        }
    }
}

/// Clear the active tool and any in-progress placement.
fn reset(
    tool: RwSignal<Option<Tool>>,
    placed: RwSignal<Vec<Coord>>,
    preview: RwSignal<Option<Coord>>,
    sticky_run: RwSignal<bool>,
) {
    placed.set(Vec::new());
    preview.set(None);
    tool.set(None);
    sticky_run.set(false);
}

/// A trimmed-to-`Option` string: `Some(s)` unless it's empty, so clearing a
/// catalog item's name/category stores `None` (a fallback / uncategorized)
/// rather than a blank string.
fn non_empty(s: &str) -> Option<String> {
    (!s.is_empty()).then(|| s.to_string())
}

/// The status hint for the active tool.
fn hint(tool: Option<Tool>) -> &'static str {
    match tool {
        None => "Pick a tool to draw.",
        Some(Tool::House) => "Click corners; click the first corner to close the outline.",
        Some(Tool::Deck) => "Click corners; click the first corner to close the deck.",
        Some(Tool::Shape) => "Click corners; click the first corner to close the area.",
        Some(Tool::Door) => "Click two points on a wall to place the door.",
        Some(Tool::Window) => "Click two points on a wall to place the window.",
        Some(Tool::Steps) => "Click two points on a deck edge to add steps.",
        Some(Tool::Circle) => "Click the center, then click again to set the radius.",
        Some(Tool::Object) => {
            "Click to place the armed item (its tile again, or Esc, to cancel) \
             · hold Shift to place several · ⌥/Alt to place a what-if ghost."
        }
    }
}

/// A small starter catalog (furniture, a fire pit, a few trees), seeded once
/// on load. Plan data the user can place, ignore, or (once catalog editing
/// lands) replace — not hardcoded geometry. Footprints are in feet, prices in
/// dollars.
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
    // A round object (fire pit): a circular footprint of diameter `width_ft`.
    let round = |id: &str, name: &str, category: &str, diameter: f64, h: f64, price: f64| {
        let mut c = CatalogItem::new(id.to_string());
        c.name = Some(name.to_string());
        c.category = Some(category.to_string());
        c.shape = FootprintShape::circle;
        c.width_ft = Some(diameter);
        c.depth_ft = Some(diameter);
        c.height_ft = Some(h);
        c.unit_price = Some(price);
        c
    };
    // A tree: a round canopy (like `round`) plus its trunk diameter — both
    // adjustable per placed tree once it's in the yard (this is just the
    // species default).
    let tree = |id: &str, name: &str, canopy: f64, trunk: f64, h: f64, price: f64| {
        let mut c = round(id, name, "tree", canopy, h, price);
        c.trunk_diameter_ft = Some(trunk);
        c
    };
    // An area material (not a placeable object): drawn areas reference it by
    // `material_ref`, and it's costed by its `price_unit` — mulch per yd³.
    let material = |id: &str, name: &str, category: &str, unit: PriceUnit, price: f64| {
        let mut c = CatalogItem::new(id.to_string());
        c.name = Some(name.to_string());
        c.category = Some(category.to_string());
        c.price_unit = unit;
        c.unit_price = Some(price);
        c
    };
    vec![
        furniture("lounge-chair", "Lounge chair", 2.5, 3.0, 2.5, 199.0),
        furniture("outdoor-sofa", "Outdoor sofa", 7.0, 3.0, 2.5, 899.0),
        furniture("dining-table", "Dining table", 4.0, 6.0, 2.5, 649.0),
        furniture("side-table", "Side table", 1.5, 1.5, 1.5, 89.0),
        furniture("patio-umbrella", "Patio umbrella", 9.0, 9.0, 8.0, 149.0),
        {
            let diameter = 3.0;
            let mut fire_pit = round("fire-pit", "Fire pit", "fire-pit", diameter, 1.5, 349.0);
            // Default keep-clear guideline: clearance_ft = the fire pit's own
            // radius, so the total stay-out zone (radius + clearance) is 2x
            // its radius — a reasonable baseline, editable per fire pit once
            // catalog authoring lands (a bigger or more sensitive unit might
            // need a wider zone).
            fire_pit.clearance_ft = Some(diameter / 2.0);
            fire_pit
        },
        // Trees: a round canopy + trunk, no clearance ring — a keep-clear
        // safety zone is a fire pit's concept, not a tree's.
        tree("japanese-maple", "Japanese maple", 8.0, 0.5, 12.0, 150.0),
        tree(
            "flowering-dogwood",
            "Flowering dogwood",
            12.0,
            0.6,
            18.0,
            220.0,
        ),
        tree("oak-tree", "Oak tree", 20.0, 2.0, 35.0, 350.0),
        // Area materials, costed by measure rather than per item: mulch per
        // yd³ (a bagged/bulk mulch at a typical ~$40/yd³ delivered), pavers
        // per ft² of surface (~$8/ft² for the pavers themselves).
        material(
            "mulch",
            "Mulch",
            "mulch-bed",
            PriceUnit::per_cubic_yard,
            40.0,
        ),
        {
            // A paver patio is a three-layer assembly: the pavers themselves
            // (per ft²) over a compacted gravel base (~4 in) over a bedding
            // sand layer (~1 in). The base/bedding materials are seeded right
            // after this so the estimate lists all three lines together; their
            // volume follows every paver area automatically (see `take_off`).
            let mut paver = material("paver", "Pavers", "paver", PriceUnit::per_square_foot, 8.0);
            paver.base_material_ref = Some("paver-base".to_string());
            paver.base_depth_in = Some(4.0);
            paver.bedding_material_ref = Some("paver-sand".to_string());
            paver.bedding_depth_in = Some(1.0);
            paver
        },
        // The paver sub-base courses, costed per yd³ (typical bulk/delivered
        // prices): crushed gravel base, then bedding sand.
        material(
            "paver-base",
            "Gravel base",
            "paver-base",
            PriceUnit::per_cubic_yard,
            55.0,
        ),
        material(
            "paver-sand",
            "Bedding sand",
            "paver-sand",
            PriceUnit::per_cubic_yard,
            42.0,
        ),
    ]
}

#[cfg(feature = "csr")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Whether the keyboard focus is in a text field / picker, so global shortcuts
/// (Delete/Backspace) don't hijack a keypress meant for editing.
#[cfg(feature = "csr")]
fn is_editing_field() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element())
        .is_some_and(|el| {
            matches!(
                el.tag_name().to_uppercase().as_str(),
                "INPUT" | "SELECT" | "TEXTAREA"
            )
        })
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

//! A legend along the bottom strip (screen-space chrome, not to scale) that
//! explains the plan's visual conventions — what each shape's color, fill, and
//! outline mean, whether its corners are square (a placed footprint) or
//! **node**-marked (a user-drawn outline, since those corners are individually
//! draggable points), and — for furniture — whether its outline is **single**
//! (planned) or **double** (existing) and **solid** (real) or **dashed**
//! (virtual). Reads from the shared [`crate::style`] module — the same source
//! `Furnishings`/`House`/`Deck` draw from — so it can never drift from the
//! canvas: change a look there, and the legend updates with it.

use leptos::prelude::*;
use slp_core::ItemStatus;

use crate::style::{
    CLEARANCE_STROKE, DECK_FILL, DECK_FILL_OPACITY, DECK_STROKE, DOUBLE_LINE_GAP_PX,
    DOUBLE_LINE_STROKE_W, FURNITURE_FILL, FURNITURE_STROKE, HOUSE_FILL, HOUSE_FILL_OPACITY,
    HOUSE_STROKE, OVERFLOW_STROKE, SELECTED_FILL, SELECTED_STROKE, furniture_style,
};

/// Icon footprint (screen px — fixed chrome, not drawn to the world scale).
const ICON_W: f64 = 14.0;
const ICON_H: f64 = 9.0;
/// Horizontal space reserved per entry (icon + gap + label + trailing gap).
const SLOT_W: f64 = 84.0;

/// An entry's outline treatment: a **node**-marked outline (house, deck — a
/// user-drawn shape whose corners are individually draggable points), a plain
/// **square** footprint (furniture and its selected/overflow modifiers), each
/// either a single or double stroke and solid or dashed, or a **ring** (an
/// unfilled dashed circle — a safety clearance zone, not a footprint at all).
#[derive(Clone, Copy)]
enum Outline {
    Node,
    Square { dash: &'static str, double: bool },
    Ring,
}

#[derive(Clone, Copy)]
struct Entry {
    testid: &'static str,
    label: &'static str,
    fill: &'static str,
    fill_opacity: &'static str,
    stroke: &'static str,
    stroke_width: &'static str,
    outline: Outline,
}

#[component]
pub fn Legend(
    /// Where the first entry starts (screen px) — the caller places this to
    /// the right of the scale bar.
    start_x: f64,
    /// The bottom of every icon sits on this baseline (matches the scale bar).
    baseline_y: f64,
) -> impl IntoView {
    let planned_real = furniture_style(&ItemStatus::planned, false);
    let existing_real = furniture_style(&ItemStatus::existing, false);
    let planned_ghost = furniture_style(&ItemStatus::planned, true);
    let existing_ghost = furniture_style(&ItemStatus::existing, true);

    let furniture_entry = |testid, label, style: &crate::style::FurnitureStyle| Entry {
        testid,
        label,
        fill: FURNITURE_FILL,
        fill_opacity: style.fill_opacity,
        stroke: FURNITURE_STROKE,
        // Double (existing) icons use the same thin lines as the canvas.
        stroke_width: if style.double {
            DOUBLE_LINE_STROKE_W
        } else {
            "1.5"
        },
        outline: Outline::Square {
            dash: style.dash,
            double: style.double,
        },
    };

    let entries = [
        Entry {
            testid: "house",
            label: "House",
            fill: HOUSE_FILL,
            fill_opacity: HOUSE_FILL_OPACITY,
            stroke: HOUSE_STROKE,
            stroke_width: "1.5",
            outline: Outline::Node,
        },
        Entry {
            testid: "deck",
            label: "Deck",
            fill: DECK_FILL,
            fill_opacity: DECK_FILL_OPACITY,
            stroke: DECK_STROKE,
            stroke_width: "1.5",
            outline: Outline::Node,
        },
        furniture_entry("planned", "Planned", &planned_real),
        furniture_entry("existing", "Existing", &existing_real),
        furniture_entry("planned-virtual", "Planned (virtual)", &planned_ghost),
        furniture_entry("existing-virtual", "Existing (virtual)", &existing_ghost),
        Entry {
            testid: "selected",
            label: "Selected",
            fill: SELECTED_FILL,
            fill_opacity: "0.7",
            stroke: SELECTED_STROKE,
            stroke_width: "2",
            outline: Outline::Square {
                dash: "none",
                double: false,
            },
        },
        Entry {
            testid: "overflow",
            label: "Doesn't fit",
            fill: FURNITURE_FILL,
            fill_opacity: "0.7",
            stroke: OVERFLOW_STROKE,
            stroke_width: "2.5",
            outline: Outline::Square {
                dash: "none",
                double: false,
            },
        },
        Entry {
            testid: "clearance",
            label: "Keep-clear zone",
            fill: "none",
            fill_opacity: "1",
            stroke: CLEARANCE_STROKE,
            stroke_width: "1.5",
            outline: Outline::Ring,
        },
    ];

    let items = entries
        .into_iter()
        .enumerate()
        .map(|(i, entry)| legend_item(start_x + i as f64 * SLOT_W, baseline_y, entry))
        .collect::<Vec<_>>();

    view! {
        <g class="legend" data-testid="legend">
            {items}
        </g>
    }
}

fn legend_item(x: f64, baseline_y: f64, entry: Entry) -> impl IntoView {
    let top = baseline_y - ICON_H;
    let testid = format!("legend-item-{}", entry.testid);
    let dash = match entry.outline {
        Outline::Node => "none",
        Outline::Square { dash, .. } => dash,
        Outline::Ring => "5,3", // matches Furnishings's clearance-ring dash
    };
    // A ring is an unfilled dashed circle (a keep-clear zone, not a
    // footprint); everything else is the plain rect icon.
    let icon = if matches!(entry.outline, Outline::Ring) {
        view! {
            <circle
                cx=x + ICON_W / 2.0
                cy=top + ICON_H / 2.0
                r=ICON_H / 2.0
                fill="none"
                stroke=entry.stroke
                stroke-width=entry.stroke_width
                stroke-dasharray=dash
            />
        }
        .into_any()
    } else {
        view! {
            <rect
                x=x
                y=top
                width=ICON_W
                height=ICON_H
                fill=entry.fill
                fill-opacity=entry.fill_opacity
                stroke=entry.stroke
                stroke-width=entry.stroke_width
                stroke-dasharray=dash
            />
        }
        .into_any()
    };
    // A node outline gets a corner dot at each of the icon's 4 vertices — the
    // contrast with the plain square icon (no dots) is the point.
    let dots = matches!(entry.outline, Outline::Node).then(|| {
        [
            (x, top),
            (x + ICON_W, top),
            (x + ICON_W, top + ICON_H),
            (x, top + ICON_H),
        ]
        .into_iter()
        .map(|(cx, cy)| view! { <circle cx=cx cy=cy r="1.3" fill=entry.stroke /> })
        .collect::<Vec<_>>()
    });
    // A double outline (existing) gets a second, inset stroke — matching the
    // nested-rect technique `Furnishings` uses on the real canvas.
    let inner = matches!(entry.outline, Outline::Square { double: true, .. }).then(|| {
        let gap = DOUBLE_LINE_GAP_PX;
        view! {
            <rect
                x=x + gap
                y=top + gap
                width=ICON_W - 2.0 * gap
                height=ICON_H - 2.0 * gap
                fill="none"
                stroke=entry.stroke
                stroke-width=entry.stroke_width
                stroke-dasharray=dash
            />
        }
    });
    view! {
        <g class="legend-item" data-testid=testid>
            {icon}
            {dots}
            {inner}
            <text x=x + ICON_W + 4.0 y=baseline_y - ICON_H / 2.0 + 3.0 font-size="9" fill="#555">
                {entry.label}
            </text>
        </g>
    }
}

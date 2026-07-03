//! A legend along the bottom strip (screen-space chrome, not to scale) that
//! explains the plan's visual conventions — what each shape's color, fill, and
//! outline mean, and whether its corners are square (a placed footprint) or
//! **node**-marked (a user-drawn outline, since those corners are individually
//! draggable points). Reads from the shared [`crate::style`] module — the same
//! source `Furnishings`/`House`/`Deck` draw from — so it can never drift from
//! the canvas: change a look there, and the legend updates with it.

use leptos::prelude::*;
use slp_core::ItemStatus;

use crate::style::{
    DECK_FILL, DECK_FILL_OPACITY, DECK_STROKE, FURNITURE_FILL, FURNITURE_STROKE, HOUSE_FILL,
    HOUSE_FILL_OPACITY, HOUSE_STROKE, OVERFLOW_STROKE, SELECTED_FILL, SELECTED_STROKE,
    furniture_style,
};

/// Icon footprint (screen px — fixed chrome, not drawn to the world scale).
const ICON_W: f64 = 14.0;
const ICON_H: f64 = 9.0;
/// Horizontal space reserved per entry (icon + gap + label + trailing gap).
const SLOT_W: f64 = 76.0;

/// An entry's outline treatment: a **node**-marked outline (house, deck — a
/// user-drawn shape whose corners are individually draggable points) or a
/// plain **square** footprint (furniture and its selected/overflow modifiers).
#[derive(Clone, Copy)]
enum Outline {
    Node,
    Square { dash: &'static str },
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
    let planned = furniture_style(&ItemStatus::planned);
    let existing = furniture_style(&ItemStatus::existing);
    let ghost = furniture_style(&ItemStatus::r#virtual);

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
        Entry {
            testid: "planned",
            label: "Planned",
            fill: FURNITURE_FILL,
            fill_opacity: planned.fill_opacity,
            stroke: FURNITURE_STROKE,
            stroke_width: "1.5",
            outline: Outline::Square { dash: planned.dash },
        },
        Entry {
            testid: "existing",
            label: "Existing",
            fill: FURNITURE_FILL,
            fill_opacity: existing.fill_opacity,
            stroke: FURNITURE_STROKE,
            stroke_width: "1.5",
            outline: Outline::Square {
                dash: existing.dash,
            },
        },
        Entry {
            testid: "virtual",
            label: "Virtual",
            fill: FURNITURE_FILL,
            fill_opacity: ghost.fill_opacity,
            stroke: FURNITURE_STROKE,
            stroke_width: "1.5",
            outline: Outline::Square { dash: ghost.dash },
        },
        Entry {
            testid: "selected",
            label: "Selected",
            fill: SELECTED_FILL,
            fill_opacity: "0.7",
            stroke: SELECTED_STROKE,
            stroke_width: "2",
            outline: Outline::Square { dash: "none" },
        },
        Entry {
            testid: "overflow",
            label: "Doesn't fit",
            fill: FURNITURE_FILL,
            fill_opacity: "0.7",
            stroke: OVERFLOW_STROKE,
            stroke_width: "2.5",
            outline: Outline::Square { dash: "none" },
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
    let icon = view! {
        <rect
            x=x
            y=top
            width=ICON_W
            height=ICON_H
            fill=entry.fill
            fill-opacity=entry.fill_opacity
            stroke=entry.stroke
            stroke-width=entry.stroke_width
            stroke-dasharray=match entry.outline {
                Outline::Node => "none",
                Outline::Square { dash } => dash,
            }
        />
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
    view! {
        <g class="legend-item" data-testid=testid>
            {icon}
            {dots}
            <text x=x + ICON_W + 4.0 y=baseline_y - ICON_H / 2.0 + 3.0 font-size="9" fill="#555">
                {entry.label}
            </text>
        </g>
    }
}

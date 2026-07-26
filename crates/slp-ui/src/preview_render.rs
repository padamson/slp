//! How to turn the on-screen plan into an image an image-model can work from
//! (R4.2). The live SVG is a *drawing tool*: it carries grid lines, drag
//! handles, node labels and status tints that would render as artifacts, and it
//! paints the lawn near-white. So the preview raster is a **derived** render —
//! strip the chrome, remap UI colors to the real colors of the things they
//! stand for — not a screenshot.
//!
//! The rules live here (pure data, unit-tested); `window.slpRender` in the app
//! shell applies them to a clone of the live SVG before rasterizing. Colors are
//! matched as literal `#rrggbb` strings, exactly as [`crate::style`] emits them.
//!
//! Why it matters: at img2img the model photorealizes what it's given, so a
//! near-white lawn comes back as pale concrete rather than grass. See
//! `docs/notebooks/2026-07-26-preview-conditioning.md`.

/// UI colors → the real-world color of what they represent, applied to the
/// preview raster. Only entries that actually mislead the model are remapped;
/// SLP's material fills (mulch brown, paver gray, deck tan, water blue) are
/// already close enough to reality to leave alone.
///
/// - the yard's near-white → lawn green, so it renders as grass
/// - the grid's line color → the same lawn green, so leftover rules vanish
/// - selection blue → the ordinary furniture fill, so a selected object doesn't
///   come back as a blue prop
/// - the two red warning strokes → ordinary outlines; "doesn't fit" and
///   "something's in the keep-clear zone" are UI signals, not things in the yard
pub const PREVIEW_PALETTE: &[(&str, &str)] = &[
    ("#eef0e6", "#6f9c4a"), // yard fill        -> lawn green
    ("#cfd3c0", "#6f9c4a"), // grid lines       -> lawn green
    ("#7ea9d4", "#a8927a"), // selected fill    -> furniture fill
    ("#2b6cb0", "#5a4a3a"), // selected stroke  -> furniture stroke
    ("#d4351c", "#5a4a3a"), // overflow stroke  -> furniture stroke
    ("#7a1216", "#5a4a3a"), // intrusion stroke -> furniture stroke
];

/// Elements dropped from the preview raster, by `data-testid`. These are
/// editing affordances and advisory overlays — handles, node numbers, the
/// insert/cancel affordances, and the dashed keep-clear ring. Everything
/// physical (footprints, borders, the hot tub's pad, a tree's trunk) stays.
pub const PREVIEW_STRIP_TESTIDS: &[&str] = &[
    "shape-node",
    "shape-node-index",
    "shape-edge-handle",
    "shape-control-handle",
    "shape-seam-handle",
    "circle-resize-handle",
    "house-node",
    "deck-node",
    "insert-node",
    "cancel-node-select",
    "rotate-handle",
    "canopy-handle",
    "trunk-handle",
    "clearance-ring",
];

/// The preview-raster rules as JSON, for the `window.slpRender` bridge:
/// `{"palette":[["#from","#to"],…],"stripTestids":["…"],"background":"#rrggbb"}`.
///
/// `background` is painted before the SVG is drawn, so any transparent area
/// (and the canvas itself) reads as lawn rather than black — a transparent PNG
/// flattens to black on a canvas, which would photorealize as asphalt.
#[must_use]
pub fn preview_raster_config() -> String {
    let palette = PREVIEW_PALETTE
        .iter()
        .map(|(from, to)| format!("[\"{from}\",\"{to}\"]"))
        .collect::<Vec<_>>()
        .join(",");
    let strip = PREVIEW_STRIP_TESTIDS
        .iter()
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"palette\":[{palette}],\"stripTestids\":[{strip}],\"background\":\"{LAWN}\"}}")
}

/// The lawn green the preview paints grass with — a mid, slightly muted green
/// that reads as turf rather than a highlighter.
pub const LAWN: &str = "#6f9c4a";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_yard_and_grid_become_lawn() {
        // The near-white yard is the whole reason this module exists: left
        // alone it photorealizes as pale concrete, not grass.
        let map = |k: &str| {
            PREVIEW_PALETTE
                .iter()
                .find(|(f, _)| *f == k)
                .map(|(_, t)| *t)
        };
        assert_eq!(map("#eef0e6"), Some(LAWN), "yard fill -> lawn");
        assert_eq!(map("#cfd3c0"), Some(LAWN), "grid lines -> lawn");
    }

    #[test]
    fn ui_only_signals_are_neutralized() {
        // Selection tint and the two warning reds describe app state, not the
        // yard — none may survive into the raster.
        for ui in ["#7ea9d4", "#2b6cb0", "#d4351c", "#7a1216"] {
            let hit = PREVIEW_PALETTE.iter().find(|(f, _)| *f == ui);
            assert!(hit.is_some(), "{ui} is remapped");
            assert_ne!(hit.unwrap().1, ui, "{ui} maps to something else");
        }
    }

    #[test]
    fn material_fills_are_left_alone() {
        // Mulch, pavers, deck, water, concrete already read true; remapping
        // them would throw away the signal that makes img2img work.
        for keep in ["#6b4a2f", "#9a9ca0", "#c8a97e", "#6faec5", "#cfccc4"] {
            assert!(
                !PREVIEW_PALETTE.iter().any(|(f, _)| *f == keep),
                "{keep} must not be remapped"
            );
        }
    }

    #[test]
    fn handles_are_stripped_but_physical_things_are_kept() {
        for chrome in ["shape-node", "rotate-handle", "clearance-ring"] {
            assert!(PREVIEW_STRIP_TESTIDS.contains(&chrome), "{chrome} stripped");
        }
        for real in ["hot-tub-pad", "tree-trunk", "shape-border", "yard"] {
            assert!(
                !PREVIEW_STRIP_TESTIDS.contains(&real),
                "{real} is physical — it stays"
            );
        }
    }

    #[test]
    fn the_config_is_well_formed_json_carrying_every_rule() {
        let cfg = preview_raster_config();
        assert!(cfg.starts_with('{') && cfg.ends_with('}'));
        assert!(cfg.contains("\"palette\":[["), "palette is a list of pairs");
        assert!(cfg.contains(&format!("\"background\":\"{LAWN}\"")));
        for (from, to) in PREVIEW_PALETTE {
            assert!(cfg.contains(&format!("[\"{from}\",\"{to}\"]")), "{from}");
        }
        for t in PREVIEW_STRIP_TESTIDS {
            assert!(cfg.contains(&format!("\"{t}\"")), "{t}");
        }
    }
}

//! Scene description for the photorealistic preview (R4): turn a [`Plan`] into a
//! natural-language prompt plus the reference images it should carry, for an
//! image-generation model. Backend-agnostic — the app hands these to whichever
//! model is configured (a local Z-Image/ControlNet run, or a hosted API). Pure
//! and deterministic, so it's unit- and mutation-tested; it never touches the
//! network or the DOM.

use crate::generated::slp::{CatalogItem, Plan};

/// A reference image to condition the preview on: a catalog product/material
/// photo that appears in the plan, with a short label describing what it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    /// A short human label, e.g. "paver texture" or "hot tub".
    pub label: String,
    /// The image as a `data:` URI (or URL), taken from the catalog item.
    pub image: String,
}

/// The most references we hand a model — enough to cover a plan's materials and
/// signature objects without blowing past model input limits.
pub const MAX_REFERENCES: usize = 8;

/// Which third of a span `v` falls in: `0` (first), `1` (middle), `2` (last).
/// Values at or past the ends clamp to `0`/`2`; a non-positive span is all
/// middle (nothing to divide).
fn third(v: f64, span: f64) -> u8 {
    if span <= 0.0 {
        return 1;
    }
    let r = v / span;
    if r < 1.0 / 3.0 {
        0
    } else if r < 2.0 / 3.0 {
        1
    } else {
        2
    }
}

/// Human phrasing for where `(x, y)` sits within a yard `width`×`depth` feet: a coarse
/// 3×3 bucket — back/middle/front (north, small `y`, is the back) by
/// left/center/right (west, small `x`, is the left) — e.g. `"the back-left"`,
/// `"the middle"`, `"the front-right"`, `"the left"`.
#[must_use]
pub fn describe_position(x: f64, y: f64, width: f64, depth: f64) -> String {
    let col = third(x, width); // 0 left, 1 center, 2 right
    let row = third(y, depth); // 0 back, 1 middle, 2 front
    let depth = match row {
        0 => "back",
        2 => "front",
        _ => "",
    };
    let side = match col {
        0 => "left",
        2 => "right",
        _ => "",
    };
    let place = match (depth, side) {
        ("", "") => "middle".to_string(),
        ("", s) => s.to_string(),
        (dp, "") => dp.to_string(),
        (dp, s) => format!("{dp}-{s}"),
    };
    format!("the {place}")
}

/// Look up a catalog item by id.
fn item<'a>(plan: &'a Plan, id: &str) -> Option<&'a CatalogItem> {
    plan.catalog.iter().find(|c| c.id == id)
}

/// A readable form of a catalog `category` slug for prose: hyphens to spaces
/// (`"mulch-bed"` → `"mulch bed"`, `"hot-tub"` → `"hot tub"`). Empty stays empty.
fn humanize(category: &str) -> String {
    category.replace('-', " ")
}

/// The mean of a set of corner points — a coarse center good enough for
/// position bucketing. `None` for an empty set.
fn centroid(corners: &[crate::Coord]) -> Option<(f64, f64)> {
    if corners.is_empty() {
        return None;
    }
    // `corners.len()` is small (a drawn outline); the cast is exact.
    #[allow(clippy::cast_precision_loss)]
    let n = corners.len() as f64;
    let sx: f64 = corners.iter().map(|c| c.x).sum();
    let sy: f64 = corners.iter().map(|c| c.y).sum();
    Some((sx / n, sy / n))
}

/// A natural-language description of the plan for an image model: the yard's
/// size, the house and deck if drawn, each surfaced material area (with its
/// laying pattern), and the placed objects grouped by category with a rough
/// position — then a fixed photorealism/style line. Deterministic.
#[must_use]
pub fn scene_prompt(plan: &Plan) -> String {
    let (width, depth) = (plan.yard_width, plan.yard_depth);
    let mut clauses: Vec<String> = Vec::new();
    clauses.push(format!(
        "A photorealistic backyard landscape, roughly {width:.0} by {depth:.0} feet"
    ));

    if plan.house.is_some() {
        clauses.push("bordered by a house".to_string());
    }
    if let Some(deck) = &plan.deck
        && !deck.levels.is_empty()
    {
        let kind = if deck.levels.len() > 1 {
            "a multi-level wood deck"
        } else {
            "a wood deck"
        };
        clauses.push(kind.to_string());
    }

    // Surfaced material areas (shapes then circles): "a paver patio in a
    // herringbone pattern in the front-right".
    for shape in &plan.shapes {
        let center = centroid(&shape.corners);
        if let Some(clause) = describe_area(
            plan,
            shape.material_ref.as_deref(),
            shape.pattern.as_deref(),
            center,
            width,
            depth,
        ) {
            clauses.push(clause);
        }
    }
    for circle in &plan.circles {
        let center = Some((circle.center.x, circle.center.y));
        if let Some(clause) = describe_area(
            plan,
            circle.material_ref.as_deref(),
            circle.pattern.as_deref(),
            center,
            width,
            depth,
        ) {
            clauses.push(clause);
        }
    }

    // Placed objects grouped by catalog category, in first-seen order: a lone
    // one gets a position, several get a count.
    for clause in object_clauses(plan, width, depth) {
        clauses.push(clause);
    }

    clauses.push(
        "Photorealistic, natural daylight, professionally landscaped, wide-angle view".to_string(),
    );
    format!("{}.", clauses.join(", "))
}

/// One area's clause, or `None` when it has no resolvable material.
fn describe_area(
    plan: &Plan,
    material_ref: Option<&str>,
    pattern: Option<&str>,
    center: Option<(f64, f64)>,
    width: f64,
    depth: f64,
) -> Option<String> {
    use std::fmt::Write as _;
    let category = item(plan, material_ref?)?.category.as_deref()?;
    let surface = humanize(category);
    let mut clause = format!("a {surface} area");
    if let Some(pat) = pattern {
        let _ = write!(clause, " in a {} pattern", pat.to_lowercase());
    }
    if let Some((cx, cy)) = center {
        let _ = write!(clause, " in {}", describe_position(cx, cy, width, depth));
    }
    Some(clause)
}

/// Object clauses grouped by category (first-seen order), counting the
/// **planned, non-virtual** objects — the ones actually in the design.
fn object_clauses(plan: &Plan, width: f64, depth: f64) -> Vec<String> {
    use crate::generated::slp::ItemStatus;
    // Category → (count, first position), preserving first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut counts: std::collections::HashMap<String, (usize, (f64, f64))> =
        std::collections::HashMap::new();
    for obj in &plan.objects {
        if obj.status != ItemStatus::planned || obj.is_virtual {
            continue;
        }
        let Some(category) = item(plan, &obj.catalog_ref).and_then(|i| i.category.clone()) else {
            continue;
        };
        let entry = counts.entry(category.clone()).or_insert_with(|| {
            order.push(category.clone());
            (0, (obj.x, obj.y))
        });
        entry.0 += 1;
    }
    order
        .into_iter()
        .map(|category| {
            let (count, (ox, oy)) = counts[&category];
            let name = humanize(&category);
            if count == 1 {
                format!("a {name} in {}", describe_position(ox, oy, width, depth))
            } else {
                format!("{count} {name} items")
            }
        })
        .collect()
}

/// The distinct catalog images the plan references — one per catalog item used
/// by a drawn area (`material_ref`) or a placed object (`catalog_ref`) that
/// carries an `image`, deduped by id in first-seen order and capped at
/// [`MAX_REFERENCES`]. Each labeled by the item's category (falling back to its
/// name). These become the model's reference images.
#[must_use]
pub fn reference_images(plan: &Plan) -> Vec<Reference> {
    use crate::generated::slp::ItemStatus;
    let mut ids: Vec<&str> = Vec::new();
    for s in &plan.shapes {
        if let Some(m) = s.material_ref.as_deref() {
            ids.push(m);
        }
    }
    for c in &plan.circles {
        if let Some(m) = c.material_ref.as_deref() {
            ids.push(m);
        }
    }
    for o in &plan.objects {
        if o.status == ItemStatus::planned && !o.is_virtual {
            ids.push(&o.catalog_ref);
        }
    }

    let mut seen: Vec<&str> = Vec::new();
    let mut out: Vec<Reference> = Vec::new();
    for id in ids {
        if seen.contains(&id) {
            continue;
        }
        seen.push(id);
        let Some(it) = item(plan, id) else { continue };
        let Some(image) = it.image.clone() else {
            continue;
        };
        let label = it
            .category
            .as_deref()
            .map(humanize)
            .or_else(|| it.name.clone())
            .unwrap_or_else(|| id.to_string());
        out.push(Reference { label, image });
        if out.len() >= MAX_REFERENCES {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Coord;
    use crate::generated::slp::{Circle, ItemStatus, Object, Shape};

    fn cat(id: &str, category: &str) -> CatalogItem {
        let mut c = CatalogItem::new(id.to_string());
        c.name = Some(id.to_string());
        c.category = Some(category.to_string());
        c
    }

    fn rect(mat: &str, pattern: Option<&str>, cx: f64, cy: f64) -> Shape {
        // A 2×2 square centered at (cx, cy) → its centroid is (cx, cy).
        Shape {
            corners: vec![
                Coord::new(cx - 1.0, cy - 1.0),
                Coord::new(cx + 1.0, cy - 1.0),
                Coord::new(cx + 1.0, cy + 1.0),
                Coord::new(cx - 1.0, cy + 1.0),
            ],
            material_ref: Some(mat.to_string()),
            pattern: pattern.map(str::to_string),
            ..Shape::new(0.0)
        }
    }

    #[test]
    fn position_buckets_cover_the_nine_regions_and_center() {
        // 30×30 yard: thirds at 10 and 20.
        assert_eq!(describe_position(5.0, 5.0, 30.0, 30.0), "the back-left");
        assert_eq!(describe_position(15.0, 5.0, 30.0, 30.0), "the back");
        assert_eq!(describe_position(25.0, 5.0, 30.0, 30.0), "the back-right");
        assert_eq!(describe_position(5.0, 15.0, 30.0, 30.0), "the left");
        assert_eq!(describe_position(15.0, 15.0, 30.0, 30.0), "the middle");
        assert_eq!(describe_position(25.0, 15.0, 30.0, 30.0), "the right");
        assert_eq!(describe_position(5.0, 25.0, 30.0, 30.0), "the front-left");
        assert_eq!(describe_position(15.0, 25.0, 30.0, 30.0), "the front");
        assert_eq!(describe_position(25.0, 25.0, 30.0, 30.0), "the front-right");
    }

    #[test]
    fn position_clamps_out_of_bounds_and_survives_a_zero_span() {
        // Past the far corner clamps to front-right; a degenerate span is middle.
        assert_eq!(describe_position(-5.0, -5.0, 30.0, 30.0), "the back-left");
        assert_eq!(
            describe_position(100.0, 100.0, 30.0, 30.0),
            "the front-right"
        );
        assert_eq!(describe_position(5.0, 5.0, 0.0, 0.0), "the middle");
    }

    #[test]
    fn humanize_despashes_category_slugs() {
        assert_eq!(humanize("hot-tub"), "hot tub");
        assert_eq!(humanize("mulch-bed"), "mulch bed");
        assert_eq!(humanize("paver"), "paver");
    }

    #[test]
    fn a_prompt_describes_the_yard_house_deck_and_style() {
        let mut plan = Plan::new(30.0, 30.0);
        plan.yard_width = 40.0;
        plan.yard_depth = 30.0;
        plan.house = Some(Box::new(crate::generated::slp::House {
            corners: vec![Coord::new(0.0, 0.0), Coord::new(40.0, 0.0)],
            openings: vec![],
            structure_status: ItemStatus::existing,
        }));
        let mut deck = crate::generated::slp::Deck {
            levels: vec![],
            steps: vec![],
        };
        deck.levels.push(crate::generated::slp::DeckLevel::new(1.0));
        deck.levels.push(crate::generated::slp::DeckLevel::new(2.0));
        plan.deck = Some(Box::new(deck));

        let p = scene_prompt(&plan);
        assert!(p.contains("40 by 30 feet"), "yard size: {p}");
        assert!(p.contains("bordered by a house"), "house: {p}");
        assert!(p.contains("multi-level wood deck"), "two levels: {p}");
        assert!(p.contains("Photorealistic"), "style line: {p}");
        assert!(p.ends_with('.'), "one sentence: {p}");
    }

    #[test]
    fn a_single_deck_level_reads_singular_and_no_deck_is_omitted() {
        let mut one = Plan::new(20.0, 20.0);
        let mut deck = crate::generated::slp::Deck {
            levels: vec![crate::generated::slp::DeckLevel::new(1.0)],
            steps: vec![],
        };
        deck.steps.clear();
        one.deck = Some(Box::new(deck));
        let p = scene_prompt(&one);
        assert!(p.contains("a wood deck"), "singular: {p}");
        assert!(!p.contains("multi-level"), "not multi: {p}");

        let none = Plan::new(20.0, 20.0);
        assert!(!scene_prompt(&none).contains("deck"), "no deck clause");
    }

    #[test]
    fn a_paver_area_names_its_material_pattern_and_position() {
        let mut plan = Plan::new(30.0, 30.0);
        plan.catalog = vec![cat("paver", "paver")];
        // Centered at (25, 25) → front-right.
        plan.shapes = vec![rect("paver", Some("Herringbone"), 25.0, 25.0)];
        let p = scene_prompt(&plan);
        assert!(p.contains("a paver area"), "material: {p}");
        assert!(p.contains("herringbone pattern"), "lowercased pattern: {p}");
        assert!(p.contains("front-right"), "position: {p}");
    }

    #[test]
    fn a_circle_area_and_an_unresolved_material_are_handled() {
        let mut plan = Plan::new(30.0, 30.0);
        plan.catalog = vec![cat("mulch", "mulch-bed")];
        plan.circles = vec![
            Circle {
                material_ref: Some("mulch".to_string()),
                ..Circle::new(Box::new(Coord::new(5.0, 5.0)), 0.0, 3.0)
            },
            // No material → produces no clause, doesn't panic.
            Circle::new(Box::new(Coord::new(1.0, 1.0)), 0.0, 2.0),
        ];
        let p = scene_prompt(&plan);
        assert!(p.contains("a mulch bed area in the back-left"), "{p}");
    }

    #[test]
    fn objects_group_by_category_with_a_position_for_singletons() {
        let mut plan = Plan::new(30.0, 30.0);
        plan.catalog = vec![cat("tub", "hot-tub"), cat("bush", "bush")];
        plan.objects = vec![
            Object::new("tub".to_string(), 5.0, 5.0), // lone → position
            Object::new("bush".to_string(), 10.0, 10.0),
            Object::new("bush".to_string(), 12.0, 12.0), // two → count
        ];
        let p = scene_prompt(&plan);
        assert!(p.contains("a hot tub in the back-left"), "singleton: {p}");
        assert!(p.contains("2 bush items"), "grouped count: {p}");
    }

    #[test]
    fn object_counts_exclude_virtual_and_existing() {
        let mut plan = Plan::new(30.0, 30.0);
        plan.catalog = vec![cat("tub", "hot-tub")];
        let mut ghost = Object::new("tub".to_string(), 5.0, 5.0);
        ghost.is_virtual = true;
        let mut owned = Object::new("tub".to_string(), 6.0, 6.0);
        owned.status = ItemStatus::existing;
        plan.objects = vec![Object::new("tub".to_string(), 5.0, 5.0), ghost, owned];
        let p = scene_prompt(&plan);
        // Only the one planned real tub counts → singular with a position.
        assert!(p.contains("a hot tub in"), "one real tub: {p}");
        assert!(!p.contains("items"), "not pluralized: {p}");
    }

    #[test]
    fn references_dedupe_by_id_label_by_category_and_skip_imageless() {
        let mut plan = Plan::new(30.0, 30.0);
        let mut paver = cat("paver", "paver");
        paver.image = Some("data:image/png;base64,AAAA".to_string());
        let mut tub = cat("tub", "hot-tub");
        tub.image = Some("data:image/png;base64,BBBB".to_string());
        let sand = cat("sand", "aggregate"); // no image → skipped
        plan.catalog = vec![paver, tub, sand];
        plan.shapes = vec![rect("paver", None, 5.0, 5.0), rect("paver", None, 9.0, 9.0)];
        plan.objects = vec![
            Object::new("tub".to_string(), 5.0, 5.0),
            Object::new("sand".to_string(), 6.0, 6.0),
        ];
        let refs = reference_images(&plan);
        assert_eq!(refs.len(), 2, "paver (deduped) + tub; sand has no image");
        assert_eq!(refs[0].label, "paver");
        assert_eq!(refs[0].image, "data:image/png;base64,AAAA");
        assert_eq!(refs[1].label, "hot tub");
    }

    #[test]
    fn references_are_capped() {
        let mut plan = Plan::new(30.0, 30.0);
        let mut catalog = Vec::new();
        let mut objects = Vec::new();
        for i in 0..(MAX_REFERENCES + 3) {
            let id = format!("obj{i}");
            let mut c = cat(&id, "furniture");
            c.image = Some(format!("data:image/png;base64,{i}"));
            catalog.push(c);
            objects.push(Object::new(id, 5.0, 5.0));
        }
        plan.catalog = catalog;
        plan.objects = objects;
        assert_eq!(reference_images(&plan).len(), MAX_REFERENCES);
    }

    #[test]
    fn position_thirds_split_at_the_exact_boundary() {
        // Exactly on a third of a 30 ft span (10 and 20): the strict `<` puts the
        // boundary into the *higher* bucket — 1/3 is middle (not left), 2/3 is
        // right (not middle) — pinning both comparisons.
        assert_eq!(describe_position(10.0, 15.0, 30.0, 30.0), "the middle");
        assert_eq!(describe_position(20.0, 15.0, 30.0, 30.0), "the right");
        assert_eq!(describe_position(15.0, 10.0, 30.0, 30.0), "the middle");
        assert_eq!(describe_position(15.0, 20.0, 30.0, 30.0), "the front");
    }

    #[test]
    fn an_area_is_placed_by_its_centroid_not_a_scaled_one() {
        // A 2×2 area near the back-left corner (centroid (5,5)). A corrupted
        // centroid (e.g. multiply instead of divide the corner sum) would push it
        // to the opposite corner, so the position pins the division.
        let mut plan = Plan::new(30.0, 30.0);
        plan.catalog = vec![cat("paver", "paver")];
        plan.shapes = vec![rect("paver", None, 5.0, 5.0)];
        assert!(
            scene_prompt(&plan).contains("a paver area in the back-left"),
            "centroid (5,5) → back-left"
        );
    }

    #[test]
    fn references_skip_virtual_and_existing_objects() {
        let mut plan = Plan::new(30.0, 30.0);
        let mut tub = cat("tub", "hot-tub");
        tub.image = Some("data:tub".to_string());
        let mut grill = cat("grill", "grill");
        grill.image = Some("data:grill".to_string());
        plan.catalog = vec![tub, grill];
        let mut ghost = Object::new("tub".to_string(), 5.0, 5.0);
        ghost.is_virtual = true; // planned but a what-if ghost
        let mut owned = Object::new("grill".to_string(), 6.0, 6.0);
        owned.status = ItemStatus::existing; // real but already owned
        plan.objects = vec![ghost, owned];
        assert!(
            reference_images(&plan).is_empty(),
            "only planned, non-virtual objects seed references (both conditions)"
        );
    }
}

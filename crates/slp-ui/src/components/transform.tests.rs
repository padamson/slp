//! Unit tests for the feet↔pixel `Transform`, incl. the inverse used to turn a
//! pointer click (SVG user-space px) back into world feet (H1.1).

use super::Transform;

fn t() -> Transform {
    Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 30.0,
    }
}

#[test]
fn inverse_round_trips_forward() {
    let t = t();
    for (x, y) in [(0.0, 0.0), (10.0, 5.0), (70.0, 30.0), (3.5, 21.25)] {
        assert!((t.wx(t.sx(x)) - x).abs() < 1e-9, "wx∘sx is identity");
        assert!((t.wy(t.sy(y)) - y).abs() < 1e-9, "wy∘sy is identity");
    }
}

#[test]
fn inverse_maps_known_screen_points_to_feet() {
    let t = t();
    // The pad-offset origin maps to (0,0) feet; +y in feet is up the screen.
    assert!((t.wx(40.0) - 0.0).abs() < 1e-9);
    assert!((t.wy(40.0 + 30.0 * 12.0) - 0.0).abs() < 1e-9);
    assert!((t.wx(40.0 + 12.0) - 1.0).abs() < 1e-9);
    assert!((t.wy(40.0) - 30.0).abs() < 1e-9);
}

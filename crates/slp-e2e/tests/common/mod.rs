//! Shared helpers for the slp-app browser e2e tests. (A `tests/common/` module is
//! compiled into each test binary, not as its own test target.)
// Each test binary uses a different subset of these helpers.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use axum::Router;
use playwright_rs::protocol::click::KeyboardModifier;
use playwright_rs::{ClickOptions, Locator, Page, Position, expect};
use tower_http::services::ServeDir;

/// Default yard: 70 ft wide × 30 ft deep, grid flush to the canvas.
pub const YARD_W: f64 = 70.0;
pub const YARD_D: f64 = 30.0;

/// A 1×1 transparent PNG data-URI — the self-contained stand-in material photo
/// for image tests (no fixture file, no network).
pub const TRANSPARENT_PNG_1X1: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

/// Poll a locator's attribute until it satisfies `pred` or times out.
pub async fn wait_attr(
    loc: &Locator,
    attr: &str,
    mut pred: impl FnMut(&str) -> bool,
) -> Result<String> {
    let start = Instant::now();
    loop {
        if let Some(v) = loc.get_attribute(attr).await?
            && pred(&v)
        {
            return Ok(v);
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("attribute '{attr}' never satisfied the predicate"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Poll a locator's numeric attribute until it satisfies `pred` or times out.
pub async fn wait_attr_f64(loc: &Locator, attr: &str, pred: impl Fn(f64) -> bool) -> Result<f64> {
    let mut latest = None;
    wait_attr(loc, attr, |s| {
        latest = s.parse().ok();
        latest.is_some_and(&pred)
    })
    .await?;
    latest.context("parsed attribute value")
}

/// Path to the Trunk-built `slp-app` dist directory.
pub fn dist_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../slp-app/dist")
}

/// Serve `dist` on an ephemeral local port; returns the address and the server
/// task handle (dropped when the test ends).
pub async fn serve(dist: &PathBuf) -> Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    let app = Router::new().fallback_service(ServeDir::new(dist));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind app server")?;
    let addr = listener.local_addr().context("local addr")?;
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve app");
    });
    Ok((addr, handle))
}

/// The yard's rendered pixels-per-foot (the grid spans the full canvas width).
/// Polls the yard's box: under CI's parallel-test contention the WASM app's
/// layout can lag the page load, so racing a single `bounding_box()` call
/// intermittently sees a not-yet-laid-out (or zero-width) yard.
pub async fn measure_ppf(yard: &Locator) -> Result<f64> {
    let start = Instant::now();
    loop {
        if let Some(bbox) = yard.bounding_box().await.context("measure the yard")?
            && bbox.width > 0.0
        {
            return Ok(bbox.width / YARD_W);
        }
        if start.elapsed() >= Duration::from_secs(10) {
            return Err(anyhow!("yard never laid out with a non-zero bounding box"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Click the yard at world feet `(fx, fy)` — origin south-west, north is up.
/// `ppf` is the rendered pixels-per-foot.
pub async fn click_ft(yard: &Locator, ppf: f64, fx: f64, fy: f64) -> Result<()> {
    click_ft_with(yard, ppf, fx, fy, &[]).await
}

/// Click the yard at world feet `(fx, fy)` with keyboard modifiers held (e.g.
/// `&[KeyboardModifier::Shift]` for a sticky placement, `&[KeyboardModifier::Alt]`
/// for a virtual one).
pub async fn click_ft_with(
    yard: &Locator,
    ppf: f64,
    fx: f64,
    fy: f64,
    modifiers: &[KeyboardModifier],
) -> Result<()> {
    // `force`: the placement/preview overlay redraws under the cursor while
    // playwright hovers to the point, so the default "stable" check never
    // settles — force dispatches the click at the exact position regardless.
    let opts = ClickOptions::builder()
        .position(Position {
            x: fx * ppf,
            y: (YARD_D - fy) * ppf,
        })
        .force(true)
        .modifiers(modifiers.to_vec())
        .build();
    yard.click(Some(opts))
        .await
        .context("click the yard at feet")?;
    Ok(())
}

/// Click the yard at world feet `(fx, fy)` via low-level `Mouse` dispatch
/// (not a `Locator::click`), so a modifier key held with
/// `page.keyboard().down(...)` beforehand is correctly reflected on the
/// resulting event. `Locator::click`'s own `.modifiers(...)` option is
/// *transient* — Playwright presses the key, clicks, then explicitly restores
/// (releases) it afterward, per its documented semantics — so it can't
/// represent a key genuinely held across several clicks; this can.
pub async fn mouse_click_ft(page: &Page, yard: &Locator, ppf: f64, fx: f64, fy: f64) -> Result<()> {
    let bbox = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;
    let x = bbox.x + fx * ppf;
    let y = bbox.y + (YARD_D - fy) * ppf;
    page.mouse()
        .click(x as i32, y as i32, None)
        .await
        .context("mouse click at feet")?;
    Ok(())
}

/// Arm an object for placement by clicking its palette tile. `id` is the
/// catalog id (e.g. `lounge-chair`, `fire-pit`).
pub async fn arm_object(page: &Page, id: &str) -> Result<()> {
    page.locator(&format!("[data-testid='palette-{id}']"))
        .await
        .click(None)
        .await
        .with_context(|| format!("arm the {id} tile"))?;
    Ok(())
}

/// Place the `lounge-chair` (the default first catalog item) at world feet
/// `(fx, fy)`, then wait for the one-shot tool to disarm — so a following click
/// selects rather than places.
pub async fn place(page: &Page, yard: &Locator, ppf: f64, fx: f64, fy: f64) -> Result<()> {
    place_object(page, yard, ppf, "lounge-chair", fx, fy).await
}

/// Place catalog item `id` at world feet `(fx, fy)`: arm its palette tile,
/// click the canvas, and wait for the one-shot tool to disarm.
pub async fn place_object(
    page: &Page,
    yard: &Locator,
    ppf: f64,
    id: &str,
    fx: f64,
    fy: f64,
) -> Result<()> {
    arm_object(page, id).await?;
    click_ft(yard, ppf, fx, fy).await?;
    expect(page.locator("[data-testid='hint']").await)
        .to_have_text("Pick a tool to draw.")
        .await
        .context("object tool disarms after placing")?;
    Ok(())
}

/// Draw a small central deck (well away from every yard corner). This seeds the
/// furniture catalog and auto-selects the first item; the estimate panel then
/// appears, so callers should re-measure `ppf` afterwards.
pub async fn draw_central_deck(page: &Page, yard: &Locator, ppf: f64) -> Result<()> {
    page.locator("[data-testid='draw-deck']")
        .await
        .click(None)
        .await
        .context("arm the deck tool")?;
    let deck = [(28.0, 12.0), (42.0, 12.0), (42.0, 18.0), (28.0, 18.0)];
    for (fx, fy) in deck {
        click_ft(yard, ppf, fx, fy).await?;
    }
    click_ft(yard, ppf, deck[0].0, deck[0].1).await?; // snap-close
    expect(page.locator("[data-testid='yard'] .deck polygon").await)
        .to_have_count(1)
        .await
        .context("the deck is drawn (and the catalog is seeded)")?;
    Ok(())
}

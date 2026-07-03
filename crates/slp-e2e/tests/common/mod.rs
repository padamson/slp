//! Shared helpers for the slp-app browser e2e tests. (A `tests/common/` module is
//! compiled into each test binary, not as its own test target.)
// Each test binary uses a different subset of these helpers.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use axum::Router;
use playwright_rs::{ClickOptions, Locator, Page, Position, expect};
use tower_http::services::ServeDir;

/// Default yard: 70 ft wide × 30 ft deep, grid flush to the canvas.
pub const YARD_W: f64 = 70.0;
pub const YARD_D: f64 = 30.0;

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
pub async fn measure_ppf(yard: &Locator) -> Result<f64> {
    let bbox = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;
    Ok(bbox.width / YARD_W)
}

/// Click the yard at world feet `(fx, fy)` — origin south-west, north is up.
/// `ppf` is the rendered pixels-per-foot.
pub async fn click_ft(yard: &Locator, ppf: f64, fx: f64, fy: f64) -> Result<()> {
    // `force`: the placement/preview overlay redraws under the cursor while
    // playwright hovers to the point, so the default "stable" check never
    // settles — force dispatches the click at the exact position regardless.
    let opts = ClickOptions::builder()
        .position(Position {
            x: fx * ppf,
            y: (YARD_D - fy) * ppf,
        })
        .force(true)
        .build();
    yard.click(Some(opts)).await.context("click the yard at feet")?;
    Ok(())
}

/// Arm the furniture tool (one-shot: it disarms after a placement).
pub async fn arm_furniture(page: &Page) -> Result<()> {
    page.locator("[data-testid='place-furniture']")
        .await
        .click(None)
        .await
        .context("arm the furniture tool")?;
    Ok(())
}

/// Place a furniture item at world feet `(fx, fy)`, then wait for the one-shot
/// tool to disarm — so a following click selects rather than places.
pub async fn place(page: &Page, yard: &Locator, ppf: f64, fx: f64, fy: f64) -> Result<()> {
    arm_furniture(page).await?;
    click_ft(yard, ppf, fx, fy).await?;
    expect(page.locator("[data-testid='hint']").await)
        .to_have_text("Pick a tool to draw.")
        .await
        .context("furniture tool disarms after placing")?;
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

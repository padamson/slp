//! F3.4 e2e: draw a standalone circle (click the center, click again to set
//! the radius), confirm it renders (with its area/diameter) and persists
//! across a reload, then select it and drag its resize handle to change the
//! radius live.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use common::{YARD_D, click_ft, dist_dir, measure_ppf, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::{BoundingBox, Locator, expect};

/// Poll a label's `textContent` until it matches `expected` or a short
/// timeout elapses. `<text>` is an SVG element — `to_have_text`'s `innerText`
/// read throws on it, so this reads `textContent` directly instead.
async fn expect_label(label: &Locator, expected: &str) -> Result<()> {
    let start = Instant::now();
    loop {
        let actual = label.text_content().await?.unwrap_or_default();
        if actual.trim() == expected {
            return Ok(());
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!(
                "expected circle label '{expected}', got '{actual}'"
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn draws_persists_and_resizes_a_circle() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!(
            "skipping: {} not built (run `trunk build`).",
            dist.display()
        );
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = common::new_page(&browser).await?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;

    // Arm the circle tool: click the center, then click again 5 ft east to
    // set the radius.
    page.locator("[data-testid='draw-circle']")
        .await
        .click(None)
        .await
        .context("arm the circle tool")?;
    click_ft(&yard, ppf, 20.0, 15.0).await?;
    click_ft(&yard, ppf, 25.0, 15.0).await?;

    expect(page.locator("[data-testid='yard'] .circle-area").await)
        .to_have_count(1)
        .await
        .context("the circle is drawn")?;
    let label = page.locator("[data-testid='yard'] .circle-label").await;
    // radius 5 ft -> area = π·25 ≈ 79 ft², diameter 10 ft.
    expect_label(&label, "79 ft² · ⌀10 ft")
        .await
        .context("the area + diameter label renders")?;

    // Reload — the circle is restored from localStorage.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] .circle-area").await)
        .to_have_count(1)
        .await
        .context("the circle persists across a reload")?;

    // Click inside its body (off the label) to select it — no armed tool,
    // mirroring how clicking a drawn area or a tree selects it.
    click_ft(&yard, ppf, 22.0, 17.0).await?;
    expect(
        page.locator("[data-testid='yard'] .circle-area--selected")
            .await,
    )
    .to_have_count(1)
    .await
    .context("the circle is selected")?;
    expect(page.locator("[data-testid='circle-resize-handle']").await)
        .to_have_count(1)
        .await
        .context("its resize handle appears")?;

    // Drag the resize handle (starts at world (25,15)) out to (30,15) ->
    // radius 10 ft -> area π·100 ≈ 314 ft², diameter 20 ft.
    let BoundingBox { x, y, .. } = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;
    let screen = |fx: f64, fy: f64| (x + fx * ppf, y + (YARD_D - fy) * ppf);
    let mouse = page.mouse();
    let handle = page.locator("[data-testid='circle-resize-handle']").await;
    handle
        .hover(None)
        .await
        .context("hover the resize handle")?;
    mouse.down(None).await.context("press the resize handle")?;
    let (mx, my) = screen(30.0, 15.0);
    mouse
        .move_to(mx as i32, my as i32, None)
        .await
        .context("drag it out")?;
    mouse.up(None).await.context("release")?;
    expect_label(&label, "314 ft² · ⌀20 ft")
        .await
        .context("the radius grew live")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

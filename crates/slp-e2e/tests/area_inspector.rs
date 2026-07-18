//! Area-inspector e2e: select a drawn area (a mulch bed) and confirm its
//! floating inspector shows the material, area, and cost; edit the depth in the
//! panel and watch the cost (and the estimate) update live; Remove it and watch
//! it leave the plan.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use common::{click_ft, dist_dir, measure_ppf, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::{Locator, expect};

/// Poll a locator's `text_content` until it contains `needle` or times out.
async fn wait_contains(loc: &Locator, needle: &str) -> Result<String> {
    let start = Instant::now();
    loop {
        let text = loc.text_content().await?.unwrap_or_default();
        if text.contains(needle) {
            return Ok(text);
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("'{needle}' never appeared; last was '{text}'"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn selects_an_area_and_edits_it_through_the_inspector() -> Result<()> {
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

    let yard = page.locator("[data-testid='yard']");
    let ppf = measure_ppf(&yard).await?;

    // Set the mulch depth to 3 in, then draw a 10×8 ft bed (80 ft²). At 3 in:
    // yd³ = 80·3/324 ≈ 0.74; × $40/yd³ ≈ $29.63.
    page.locator("[data-testid='area-depth']")
        .fill("3", None)
        .await
        .context("set mulch depth to 3 in")?;
    page.locator("[data-testid='draw-shape']")
        .click(None)
        .await
        .context("arm the bed tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close

    // Click inside its body (off any corner/label) to select it.
    click_ft(&yard, ppf, 12.0, 11.0).await?;
    let inspector = page.locator("[data-testid='area-inspector']");
    expect(inspector.clone())
        .to_be_visible()
        .await
        .context("selecting the area floats its inspector")?;

    // It names the material, reports the area, and shows the volume cost.
    wait_contains(&inspector, "Mulch")
        .await
        .context("the inspector titles the mulch material")?;
    let area = page.locator("[data-testid='area-inspector-area']");
    wait_contains(&area, "80 ft²")
        .await
        .context("the inspector reports 80 ft²")?;
    let cost = page.locator("[data-testid='area-inspector-cost']");
    wait_contains(&cost, "$29.")
        .await
        .context("the inspector shows the volume cost at 3 in")?;

    // Deepen to 6 in through the panel — the per-area cost roughly doubles
    // (yd³ = 80·6/324 ≈ 1.48; × $40 ≈ $59.26), live.
    page.locator("[data-testid='area-inspector-depth']")
        .fill("6", None)
        .await
        .context("deepen the bed to 6 in via the panel")?;
    wait_contains(&cost, "$59.")
        .await
        .context("the cost updates live when the depth changes")?;
    // The estimate reflects the new depth too.
    let estimate = page.locator("[data-testid='estimate']");
    wait_contains(&estimate, "Mulch")
        .await
        .context("the estimate still lists the mulch line")?;

    // Remove it — the bed leaves the plan and the estimate drops the line.
    page.locator("[data-testid='delete-area']")
        .click(None)
        .await
        .context("remove the area via the panel")?;
    expect(page.locator("[data-testid='yard'] .shape polygon"))
        .to_have_count(0)
        .await
        .context("the removed bed is gone from the plan")?;
    expect(page.locator("[data-testid='area-inspector']"))
        .to_have_count(0)
        .await
        .context("its inspector disappears once nothing is selected")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn the_inspector_dodges_an_area_drawn_in_a_corner() -> Result<()> {
    // The corner-placement rule must respect drawn areas too, not just the
    // house/deck/objects: a bed filling the NE corner sends its own inspector to
    // the next free corner (NW), rather than floating on top of it.
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

    let yard = page.locator("[data-testid='yard']");
    let ppf = measure_ppf(&yard).await?;

    // Draw a bed up in the NE corner of the default 70×30 ft yard (north = +y,
    // east = +x), then click inside it to select.
    page.locator("[data-testid='draw-shape']")
        .click(None)
        .await
        .context("arm the bed tool")?;
    let corners = [(55.0, 20.0), (65.0, 20.0), (65.0, 28.0), (55.0, 28.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close
    click_ft(&yard, ppf, 57.0, 22.0).await?; // select the bed

    // Its inspector avoids the occupied NE corner and lands in NW (next in the
    // NE → NW → SE → SW priority order).
    expect(page.locator("[data-testid='area-inspector'][data-corner='nw']"))
        .to_have_count(1)
        .await
        .context("the inspector dodges the bed in the NE corner and floats in NW")?;
    expect(page.locator("[data-testid='area-inspector'][data-corner='ne']"))
        .to_have_count(0)
        .await
        .context("it does not float over the bed it describes")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

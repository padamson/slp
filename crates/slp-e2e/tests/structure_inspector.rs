//! Structure-inspector e2e: selecting the house or a deck level floats the same
//! inspector the drawn areas use, in "structure" mode — a build-status control
//! (existing/planned) instead of material/cost, elevation for a deck level but
//! not the grade-level house. This completes the "metadata panels for all areas"
//! request (house, deck, pavers, mulch, …).
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{click_ft, dist_dir, draw_central_deck, measure_ppf, serve};
use playwright_rs::expect;
use playwright_rs::protocol::Playwright;

#[tokio::test]
async fn the_house_inspector_shows_status_and_hides_elevation() -> Result<()> {
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

    // Draw a house (kept clear of the corners), then click its body to select.
    page.locator("[data-testid='draw-house']")
        .click(None)
        .await
        .context("arm the house tool")?;
    let corners = [(20.0, 8.0), (50.0, 8.0), (50.0, 22.0), (20.0, 22.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close
    click_ft(&yard, ppf, 30.0, 12.0).await?; // select the house

    let inspector = page.locator("[data-testid='area-inspector']");
    expect(inspector.clone())
        .to_be_visible()
        .await
        .context("selecting the house floats its inspector")?;
    // Structure mode: a build-status control, and no elevation field (grade).
    expect(page.locator("[data-testid='area-status']"))
        .to_have_count(1)
        .await
        .context("the house shows an existing/planned status control")?;
    expect(page.locator("[data-testid='area-inspector-elevation']"))
        .to_have_count(0)
        .await
        .context("the grade-level house hides the elevation field")?;

    // Flip it to Planned — the button becomes active (proves the live wiring).
    page.locator("[data-testid='area-status-planned']")
        .click(None)
        .await
        .context("mark the house planned")?;
    expect(page.locator("[data-testid='area-status-planned'].active"))
        .to_have_count(1)
        .await
        .context("the planned status sticks")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn a_deck_level_inspector_edits_status_and_elevation() -> Result<()> {
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
    draw_central_deck(&page, &yard, ppf).await?;
    // The estimate panel appeared (catalog seeded) → re-measure the yard.
    let ppf = measure_ppf(&yard).await?;

    // Click the deck level's body (off-center) to select it.
    click_ft(&yard, ppf, 32.0, 14.0).await?;
    let inspector = page.locator("[data-testid='area-inspector']");
    expect(inspector.clone())
        .to_be_visible()
        .await
        .context("selecting a deck level floats its inspector")?;

    // Structure mode with an elevation field, and no cost row.
    expect(page.locator("[data-testid='area-status']"))
        .to_have_count(1)
        .await
        .context("the deck level shows a status control")?;
    expect(page.locator("[data-testid='area-inspector-elevation']"))
        .to_have_count(1)
        .await
        .context("a deck level exposes its elevation")?;
    expect(page.locator("[data-testid='area-inspector-cost']"))
        .to_have_count(0)
        .await
        .context("a structure has no material cost row")?;

    // Raise the level to 3 ft and flip it to planned — both edits are live.
    page.locator("[data-testid='area-inspector-elevation']")
        .fill("3", None)
        .await
        .context("raise the deck level to 3 ft")?;
    page.locator("[data-testid='area-status-planned']")
        .click(None)
        .await
        .context("mark the level planned")?;
    expect(page.locator("[data-testid='area-status-planned'].active"))
        .to_have_count(1)
        .await
        .context("the planned status sticks")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

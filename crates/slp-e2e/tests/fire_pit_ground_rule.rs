//! e2e: a fire pit's placement-validity rule — it may sit on the yard, a
//! paver, or the deck, but not the house.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{click_ft, dist_dir, draw_central_deck, measure_ppf, place_object, serve};
use playwright_rs::expect;
use playwright_rs::protocol::Playwright;

#[tokio::test]
async fn a_fire_pit_is_fine_on_the_deck_but_flags_red_on_the_house() -> Result<()> {
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
    draw_central_deck(&page, &yard, ppf).await?; // deck spans x:[28,42], y:[12,18]
    let ppf = measure_ppf(&yard).await?;

    // A fire pit on the deck is fine — a fire pit doesn't need to be on a
    // deck, but it isn't forbidden there either.
    place_object(&page, &yard, ppf, "fire-pit", 35.0, 15.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--overflows"))
        .to_have_count(0)
        .await
        .context("a fire pit on the deck is fine")?;
    // It fills silver, not the shared furniture brown.
    expect(page.locator("[data-testid='yard'] circle[fill='#b8b8bc']"))
        .to_have_count(1)
        .await
        .context("the fire pit fills silver")?;

    // Draw a small house and place a second fire pit inside it.
    page.locator("[data-testid='draw-house']")
        .click(None)
        .await
        .context("arm the house tool")?;
    let house = [(50.0, 20.0), (60.0, 20.0), (60.0, 27.0), (50.0, 27.0)];
    for (fx, fy) in house {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, house[0].0, house[0].1).await?; // snap-close
    expect(page.locator("[data-testid='yard'] .house-corner"))
        .to_have_count(4)
        .await
        .context("the house is drawn")?;

    place_object(&page, &yard, ppf, "fire-pit", 55.0, 23.5).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--overflows"))
        .to_have_count(1)
        .await
        .context("a fire pit on the house is flagged")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

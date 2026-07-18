//! D2.1 e2e: a fire pit's safety clearance ring — quiet and dashed when
//! nothing intrudes, red the instant another object's footprint, a house
//! wall, or a deck edge enters the keep-clear zone. Nothing is allowed inside
//! the stay-out zone, full stop — including the edge of the deck the fire
//! pit itself is standing on.
//!
//! The starter fire pit is ⌀3 ft (radius 1.5 ft) with a default clearance of
//! 1.5 ft (its own radius), so the ring's total radius is 3.0 ft — 2x the
//! fire pit's radius.
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
async fn the_ring_turns_red_when_anything_enters_the_stay_out_zone() -> Result<()> {
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
    // The central deck (28,12)-(42,18) only seeds the catalog here — this
    // first fire pit is placed well clear of it and everything else, for an
    // unambiguous "nothing intrudes" baseline.
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;

    place_object(&page, &yard, ppf, "fire-pit", 10.0, 5.0).await?;
    let ring = page.locator("[data-testid='clearance-ring']");
    expect(ring.clone())
        .to_have_count(1)
        .await
        .context("the fire pit's clearance ring renders")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--intrudes"))
        .to_have_count(0)
        .await
        .context("nothing intrudes yet")?;

    // A chair well outside the 3 ft ring doesn't intrude.
    place_object(&page, &yard, ppf, "lounge-chair", 60.0, 5.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--intrudes"))
        .to_have_count(0)
        .await
        .context("a distant object doesn't intrude")?;

    // A chair 1.5 ft from the fire pit's center is well inside the 3 ft ring.
    place_object(&page, &yard, ppf, "lounge-chair", 11.5, 5.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--intrudes"))
        .to_have_count(1)
        .await
        .context("the nearby chair trips the clearance check")?;

    // A deck edge counts too — nothing is allowed inside the stay-out zone,
    // including the deck the fire pit stands on. Place a second fire pit 1 ft
    // inside the deck's near edge (y=12): well within its own 3 ft ring.
    place_object(&page, &yard, ppf, "fire-pit", 35.0, 13.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--intrudes"))
        .to_have_count(2)
        .await
        .context("a nearby deck edge trips this fire pit's clearance too")?;

    // A house wall counts as a structure edge too. Draw a small house well
    // away from everything placed so far, then place a fresh fire pit right
    // beside its wall.
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

    // The house's left wall is at x=50; a fire pit at (48,23.5) is 2 ft from
    // it — inside the ring's 3 ft radius.
    place_object(&page, &yard, ppf, "fire-pit", 48.0, 23.5).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--intrudes"))
        .to_have_count(3)
        .await
        .context("the house wall trips this fire pit's clearance too")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

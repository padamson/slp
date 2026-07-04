//! F2.1 e2e: placement modifiers held at the canvas click — Shift keeps the
//! object tool armed for a "sticky" run (ending the instant Shift comes up),
//! and Option/Alt places the object as a virtual what-if ghost. The two
//! compose (Shift+Option places a row of ghosts).
//!
//! Shift-holding tests use `page.keyboard().down("Shift")` + `mouse_click_ft`
//! (not `click_ft_with(&[KeyboardModifier::Shift])`): `Locator::click`'s
//! `.modifiers()` option is transient — Playwright presses the key, clicks,
//! then explicitly restores (releases) it afterward — so it fires our
//! keyup-driven disarm listener after every single click, which isn't what a
//! genuinely *held* key does. `mouse_click_ft` dispatches via `page.mouse()`,
//! which correctly reflects a key held with `keyboard().down()` beforehand.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{
    arm_object, click_ft_with, dist_dir, draw_central_deck, measure_ppf, mouse_click_ft, serve,
};
use playwright_rs::protocol::Playwright;
use playwright_rs::protocol::click::KeyboardModifier;
use playwright_rs::expect;

#[tokio::test]
async fn shift_click_places_a_row_without_disarming() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;

    arm_object(&page, "lounge-chair").await?;
    page.keyboard().down("Shift").await.context("hold Shift")?;
    for fx in [31.0, 35.0, 39.0] {
        mouse_click_ft(&page, &yard, ppf, fx, 15.0).await?;
        expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
            .to_have_count(1)
            .await
            .context("Shift keeps the tile armed after a placement")?;
    }
    expect(page.locator("[data-testid='yard'] .furniture-item").await)
        .to_have_count(3)
        .await
        .context("three chairs placed in the sticky run")?;
    page.keyboard().up("Shift").await.context("release Shift")?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
        .to_have_count(0)
        .await
        .context("releasing Shift ends the run")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn releasing_shift_ends_the_sticky_run() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;

    arm_object(&page, "lounge-chair").await?;
    page.keyboard().down("Shift").await.context("hold Shift")?;
    mouse_click_ft(&page, &yard, ppf, 32.0, 15.0).await?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
        .to_have_count(1)
        .await
        .context("still armed mid-run")?;

    page.keyboard().up("Shift").await.context("release Shift")?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
        .to_have_count(0)
        .await
        .context("releasing Shift disarms the tool")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn option_click_places_a_virtual_ghost() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;

    // Option/Alt is a one-shot modifier (no held-key persistence needed, and
    // nothing listens for its release), so `click_ft_with`'s transient
    // per-click `.modifiers()` is fine here.
    arm_object(&page, "lounge-chair").await?;
    click_ft_with(&yard, ppf, 35.0, 15.0, &[KeyboardModifier::Alt]).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--virtual").await)
        .to_have_count(1)
        .await
        .context("Option/Alt places a virtual ghost")?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
        .to_have_count(0)
        .await
        .context("Option alone (no Shift) still disarms after placing")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn shift_option_places_a_row_of_ghosts() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;

    // Hold both Shift and Alt for the whole run: mouse_click_ft has no
    // per-click modifiers of its own, so both must be genuinely held keys.
    arm_object(&page, "lounge-chair").await?;
    page.keyboard().down("Shift").await.context("hold Shift")?;
    page.keyboard().down("Alt").await.context("hold Alt")?;
    for fx in [31.0, 35.0] {
        mouse_click_ft(&page, &yard, ppf, fx, 15.0).await?;
    }
    expect(page.locator("[data-testid='yard'] .furniture-item--virtual").await)
        .to_have_count(2)
        .await
        .context("Shift+Option places a row of ghosts")?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
        .to_have_count(1)
        .await
        .context("still armed — Shift was held throughout")?;
    page.keyboard().up("Alt").await.context("release Alt")?;
    page.keyboard().up("Shift").await.context("release Shift")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn escape_cancels_the_armed_tile_without_placing() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;
    draw_central_deck(&page, &yard, ppf).await?;

    arm_object(&page, "lounge-chair").await?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
        .to_have_count(1)
        .await
        .context("armed before Escape")?;

    page.keyboard().press("Escape", None).await.context("press Escape")?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
        .to_have_count(0)
        .await
        .context("Escape cancels the armed tile")?;
    expect(page.locator("[data-testid='yard'] .furniture-item").await)
        .to_have_count(0)
        .await
        .context("nothing was placed")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

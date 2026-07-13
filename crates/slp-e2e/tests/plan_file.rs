//! G1 e2e: the plan is a real file. Build a distinctive plan, **Save** it to a
//! `.slp.json` download, wipe all in-memory + `localStorage` state, then
//! **Open** the saved file and confirm the plan comes back intact.
//!
//! This exercises the universal download + `<input type="file">` path (G1.0/
//! G1.1). The File System Access API path (G1.2/G1.3 — in-place Save, startup
//! reopen) uses native pickers Playwright can't drive and is covered by
//! feature-detected code + dokime, not here.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{dist_dir, measure_ppf, place, serve};
use playwright_rs::expect;
use playwright_rs::protocol::Playwright;

#[tokio::test]
async fn saves_the_plan_to_a_file_and_loads_it_back() -> Result<()> {
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
    let page = browser.new_page().await.context("new page")?;
    // This test covers the portable download / <input> fallback, so disable the
    // File System Access API (headless Chromium provides it) — its own path is
    // covered in plan_file_fsa.rs.
    page.add_init_script(
        "window.showSaveFilePicker = undefined; window.showOpenFilePicker = undefined;",
    )
    .await
    .context("disable the File System Access API for the fallback path")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;

    // Make the plan distinctive: a non-default yard width + one placed chair.
    let width = page.locator("[data-testid='yard-width']").await;
    width.fill("42.5", None).await.context("set yard width")?;
    place(&page, &yard, ppf, 35.0, 15.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item").await)
        .to_have_count(1)
        .await
        .context("the chair is placed")?;

    // Save → capture the download. The waiter must be armed before the click.
    let waiter = page
        .expect_download(None)
        .await
        .context("arm the download waiter")?;
    page.locator("[data-testid='save-plan']")
        .await
        .click(None)
        .await
        .context("click Save")?;
    let download = waiter.wait().await.context("await the download")?;
    assert!(
        download.suggested_filename().ends_with(".slp.json"),
        "the download is a .slp.json file, got: {}",
        download.suggested_filename()
    );
    let path = std::env::temp_dir().join("slp-e2e-plan-roundtrip.slp.json");
    download
        .save_as(&path)
        .await
        .context("save the download to a temp file")?;

    // Wipe everything (in-memory + the localStorage autosave), then reload:
    // the app comes up on a fresh default plan, proving the next load is the
    // file's doing, not stale state.
    page.evaluate_value("(() => { localStorage.clear(); return 'cleared'; })()")
        .await
        .context("clear localStorage")?;
    page.reload(None).await.context("reload the app")?;
    let width = page.locator("[data-testid='yard-width']").await;
    assert_ne!(
        width.input_value(None).await?,
        "42.5",
        "state was cleared before the import"
    );
    expect(page.locator("[data-testid='yard'] .furniture-item").await)
        .to_have_count(0)
        .await
        .context("no placed objects after the wipe")?;

    // Open the saved file → the distinctive plan returns.
    page.locator("[data-testid='plan-file-input']")
        .await
        .set_input_files(&path, None)
        .await
        .context("open the saved plan file")?;
    expect(page.locator("[data-testid='yard-width']").await)
        .to_have_value("42.5")
        .await
        .context("the yard width is restored from the file")?;
    expect(page.locator("[data-testid='yard'] .furniture-item").await)
        .to_have_count(1)
        .await
        .context("the placed chair is restored from the file")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn a_malformed_file_shows_an_error_and_keeps_the_current_plan() -> Result<()> {
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
    let page = browser.new_page().await.context("new page")?;
    // This test covers the portable download / <input> fallback, so disable the
    // File System Access API (headless Chromium provides it) — its own path is
    // covered in plan_file_fsa.rs.
    page.add_init_script(
        "window.showSaveFilePicker = undefined; window.showOpenFilePicker = undefined;",
    )
    .await
    .context("disable the File System Access API for the fallback path")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    // Set a distinctive width, then try to import garbage.
    page.locator("[data-testid='yard-width']")
        .await
        .fill("33.5", None)
        .await
        .context("set yard width")?;

    let bad = std::env::temp_dir().join("slp-e2e-not-a-plan.slp.json");
    std::fs::write(&bad, b"{ this is not valid json").context("write a bad file")?;
    page.locator("[data-testid='plan-file-input']")
        .await
        .set_input_files(&bad, None)
        .await
        .context("try to open the malformed file")?;

    // An error is shown, and the current plan is untouched.
    expect(page.locator("[data-testid='load-error']").await)
        .to_be_visible()
        .await
        .context("a load error is surfaced")?;
    expect(page.locator("[data-testid='yard-width']").await)
        .to_have_value("33.5")
        .await
        .context("the current plan is left untouched")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn save_as_is_disabled_with_a_note_when_the_fs_access_api_is_absent() -> Result<()> {
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
    let page = browser.new_page().await.context("new page")?;
    // Simulate a non-Chromium browser (no File System Access API).
    page.add_init_script(
        "window.showSaveFilePicker = undefined; window.showOpenFilePicker = undefined;",
    )
    .await
    .context("disable the File System Access API")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    // Save As reads disabled (with the asterisk label) and the Chromium-only
    // footnote explains why; Save (the download) stays available.
    let save_as = page.locator("[data-testid='save-plan-as']").await;
    assert!(
        save_as.get_attribute("disabled").await?.is_some(),
        "Save As is disabled without the File System Access API"
    );
    assert!(
        save_as
            .text_content()
            .await?
            .unwrap_or_default()
            .contains('*'),
        "Save As carries the asterisk"
    );
    expect(page.locator("[data-testid='fsa-note']").await)
        .to_be_visible()
        .await
        .context("the Chromium-only footnote is shown")?;
    assert!(
        page.locator("[data-testid='save-plan']")
            .await
            .get_attribute("disabled")
            .await?
            .is_none(),
        "Save (the download) is still available"
    );

    browser.close().await.context("close browser")?;
    Ok(())
}

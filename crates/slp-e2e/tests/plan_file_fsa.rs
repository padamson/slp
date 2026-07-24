//! G1.2/G1.3 e2e: the File System Access API path — named Save As, in-place
//! Save, Open, and reopen-the-last-file-on-startup (silent when permission
//! stands, one-click "Reopen" otherwise).
//!
//! The native pickers can't be clicked, so we install playwright-rs's opt-in
//! `page.fake_file_system()` before the app boots: it fakes
//! `showSaveFilePicker`/`showOpenFilePicker` and makes a persisted handle
//! survive IndexedDB, so the app's real `window.slpFs` + `fs_access` code runs
//! unchanged against it.
//!
//! The fake keeps openable-file *content* and the permission state in page JS,
//! which a full reload clears (it survives IndexedDB, not `localStorage`). The
//! two startup-reopen tests reload and then have the app read the file, so they
//! re-establish that state with `fs.seed_on_navigation(...)`, which runs before
//! the app mounts on the next navigation.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{dist_dir, serve};
use playwright_rs::expect;
use playwright_rs::protocol::{Page, Playwright};

/// The name the app's Save As suggests (and thus the fake handle's name).
const SAVED_NAME: &str = "landscape-plan.slp.json";

/// A serialized plan with a distinctive yard width, for seeding an Open target.
fn plan_json(width: f64) -> String {
    format!(r#"{{"yard_width": {width}, "yard_depth": 30.0}}"#)
}

async fn boot(page: &Page, addr: &std::net::SocketAddr) -> Result<()> {
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;
    Ok(())
}

#[tokio::test]
async fn save_as_writes_a_named_file_save_writes_in_place_and_open_loads() -> Result<()> {
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
    let fs = page
        .fake_file_system()
        .await
        .context("install the fake File System Access API")?;
    // Count save pickers so an in-place Save that wrongly prompts is caught (the
    // fake returns a handle silently, so a stray picker wouldn't hang). Wraps
    // the fake's `showSaveFilePicker`, so it must be registered after it.
    page.add_init_script(
        "(() => { let n = 0; const orig = window.showSaveFilePicker; \
         window.showSaveFilePicker = async (o) => { n++; return orig(o); }; \
         window.__saveCalls = () => n; })()",
    )
    .await
    .context("install the save-picker counter")?;
    boot(&page, &addr).await?;

    // Save As → a named file (default stem, since the plan is unnamed).
    page.locator("[data-testid='yard-width']")
        .fill("42.5", None)
        .await
        .context("set width to 42.5")?;
    page.locator("[data-testid='save-plan-as']")
        .click(None)
        .await
        .context("Save As")?;
    expect(page.locator("[data-testid='current-file']"))
        .to_have_text(SAVED_NAME)
        .await
        .context("the current file name shows")?;
    let saved = fs
        .last_saved_bytes()
        .await
        .context("read the written file")?
        .map(String::from_utf8)
        .transpose()
        .context("saved bytes are utf-8")?
        .unwrap_or_default();
    assert!(saved.contains("42.5"), "Save As wrote the plan: {saved}");

    // In-place Save reuses the handle — no second picker, same file updated.
    page.locator("[data-testid='yard-width']")
        .fill("50", None)
        .await
        .context("change width to 50")?;
    page.locator("[data-testid='save-plan']")
        .click(None)
        .await
        .context("Save (in place)")?;
    // Poll the written file until it reflects the new width.
    for _ in 0..50 {
        let now = fs
            .last_saved_bytes()
            .await
            .ok()
            .flatten()
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_default();
        if now.contains("50") && !now.contains("42.5") {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let calls = page
        .evaluate_value("String(window.__saveCalls())")
        .await
        .context("read save-picker call count")?;
    assert_eq!(
        calls, "1",
        "Save reused the handle (only Save As showed a picker)"
    );
    assert_eq!(
        fs.last_saved_name().await.context("last saved name")?,
        Some(SAVED_NAME.to_string()),
        "the in-place Save wrote to the same file"
    );

    // Open a *different* seeded file → its plan loads.
    fs.set_open_file("other.slp.json", plan_json(77.0).as_bytes())
        .await
        .context("seed an Open target")?;
    page.locator("[data-testid='open-plan']")
        .click(None)
        .await
        .context("Open")?;
    expect(page.locator("[data-testid='yard-width']"))
        .to_have_value("77")
        .await
        .context("Open loaded the seeded plan")?;
    expect(page.locator("[data-testid='current-file']"))
        .to_have_text("other.slp.json")
        .await
        .context("the opened file becomes current")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn reopens_the_last_file_silently_on_startup() -> Result<()> {
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
    let fs = page
        .fake_file_system()
        .await
        .context("install the fake File System Access API")?;
    boot(&page, &addr).await?;

    // Save As the plan at width 42.5 (file + remembered handle), then change the
    // width to 99 WITHOUT saving — the localStorage autosave now disagrees with
    // the file. A reload that lands on 42.5 proves the *file* was reopened, not
    // the autosave.
    page.locator("[data-testid='yard-width']")
        .fill("42.5", None)
        .await?;
    page.locator("[data-testid='save-plan-as']")
        .click(None)
        .await
        .context("Save As")?;
    expect(page.locator("[data-testid='current-file']"))
        .to_have_text(SAVED_NAME)
        .await?;
    page.locator("[data-testid='yard-width']")
        .fill("99", None)
        .await
        .context("diverge the autosave to 99")?;

    // The handle survives IndexedDB, but the fake's file content lives in page
    // JS (cleared by reload) — re-seed it (permission still granted) so the
    // startup reopen reads it before the app mounts.
    fs.seed_on_navigation(SAVED_NAME, plan_json(42.5).as_bytes(), "granted")
        .await
        .context("re-seed the file content for the reload")?;
    page.reload(None).await.context("reload")?;

    expect(page.locator("[data-testid='yard-width']"))
        .to_have_value("42.5")
        .await
        .context("startup silently reopened the file (overriding the 99 autosave)")?;
    expect(page.locator("[data-testid='current-file']"))
        .to_have_text(SAVED_NAME)
        .await
        .context("the reopened file is current")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn offers_a_one_click_reopen_when_permission_lapsed() -> Result<()> {
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
    let fs = page
        .fake_file_system()
        .await
        .context("install the fake File System Access API")?;
    boot(&page, &addr).await?;

    page.locator("[data-testid='yard-width']")
        .fill("42.5", None)
        .await?;
    page.locator("[data-testid='save-plan-as']")
        .click(None)
        .await
        .context("Save As")?;
    expect(page.locator("[data-testid='current-file']"))
        .to_have_text(SAVED_NAME)
        .await?;
    page.locator("[data-testid='yard-width']")
        .fill("99", None)
        .await
        .context("diverge the autosave to 99")?;

    // On reload, re-seed the file content and lapse the read permission — the
    // fake's permission resets to granted otherwise, so a silent load would hide
    // the Reopen affordance we're asserting.
    fs.seed_on_navigation(SAVED_NAME, plan_json(42.5).as_bytes(), "prompt")
        .await
        .context("re-seed content + lapsed permission for the reload")?;
    page.reload(None).await.context("reload")?;

    // No silent load: the plan stays at the autosave's 99, and a Reopen chip
    // appears instead.
    expect(page.locator("[data-testid='reopen-file']"))
        .to_be_visible()
        .await
        .context("a Reopen affordance appears")?;
    expect(page.locator("[data-testid='yard-width']"))
        .to_have_value("99")
        .await
        .context("the file was NOT loaded without a gesture")?;

    // One click grants permission and loads the file.
    page.locator("[data-testid='reopen-file']")
        .click(None)
        .await
        .context("click Reopen")?;
    expect(page.locator("[data-testid='yard-width']"))
        .to_have_value("42.5")
        .await
        .context("Reopen loaded the file after the gesture")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

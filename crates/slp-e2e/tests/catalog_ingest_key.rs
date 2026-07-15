//! Phase B1 e2e: the screenshot-ingestion API key. Entering it in the catalog
//! inspector enables the feature, the key **persists across a reload** (it's
//! localStorage app-config), and — the safety property — it **never lands in
//! the plan autosave**, so a shared/exported `.slp.json` can't leak the secret.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{TRANSPARENT_PNG_1X1, dist_dir, serve};
use playwright_rs::expect;
use playwright_rs::protocol::{Page, Playwright};

const KEY: &str = "sk-ant-e2e-secret-0123456789";

async fn open_catalog(page: &Page) -> Result<()> {
    page.locator("[data-testid='edit-catalog']")
        .await
        .click(None)
        .await
        .context("open the catalog inspector")?;
    Ok(())
}

#[tokio::test]
async fn the_api_key_gates_the_feature_persists_and_stays_out_of_the_plan() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
        return Ok(());
    }
    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = common::new_page(&browser).await?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    open_catalog(&page).await?;

    // Gated off until a key is entered.
    expect(page.locator("[data-testid='ingest-status']").await)
        .to_have_text("Add your Anthropic API key to enable screenshot ingestion.")
        .await
        .context("the feature is gated off without a key")?;

    // Enter the key → the gate flips to enabled.
    page.locator("[data-testid='ingest-api-key']")
        .await
        .fill(KEY, None)
        .await
        .context("enter the API key")?;
    expect(page.locator("[data-testid='ingest-status']").await)
        .to_have_text("Screenshot ingestion enabled.")
        .await
        .context("a key enables the feature")?;

    // Safety property: the key lives under its own localStorage entry, and the
    // plan autosave (`slp:plan`) does NOT contain it — so exporting/sharing the
    // plan can't leak the secret.
    let stored_key = page
        .evaluate_value("localStorage.getItem('slp.anthropicKey')")
        .await
        .context("read the stored key")?;
    assert_eq!(stored_key, KEY, "the key is saved as app config");
    let plan_autosave = page
        .evaluate_value("localStorage.getItem('slp:plan') || ''")
        .await
        .context("read the plan autosave")?;
    assert!(
        !plan_autosave.contains(KEY),
        "the API key must never appear in the plan: {plan_autosave}"
    );

    // Reload → the key persists (it's localStorage), the feature stays enabled.
    page.reload(None).await.context("reload the app")?;
    open_catalog(&page).await?;
    expect(page.locator("[data-testid='ingest-status']").await)
        .to_have_text("Screenshot ingestion enabled.")
        .await
        .context("the key persisted across the reload")?;
    expect(page.locator("[data-testid='ingest-api-key']").await)
        .to_have_value(KEY)
        .await
        .context("the key field is repopulated from storage")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn pasting_a_screenshot_previews_it_and_clear_removes_it() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
        return Ok(());
    }
    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = common::new_page(&browser).await?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    open_catalog(&page).await?;
    // The paste zone is gated on the key.
    page.locator("[data-testid='ingest-api-key']")
        .await
        .fill(KEY, None)
        .await
        .context("enter the API key")?;

    // Playwright can't populate the OS clipboard, so dispatch a real
    // `ClipboardEvent` carrying an image `DataTransfer` at the paste zone — the
    // app's actual `on:paste` → FileReader path runs against it.
    let b64 = TRANSPARENT_PNG_1X1
        .split_once(',')
        .map(|(_, b)| b)
        .unwrap_or_default();
    let dispatched = page
        .evaluate_value(&format!(
            "(() => {{
               const el = document.querySelector(\"[data-testid='ingest-paste']\");
               if (!el) return 'no-zone';
               const bytes = Uint8Array.from(atob('{b64}'), c => c.charCodeAt(0));
               const file = new File([bytes], 'shot.png', {{ type: 'image/png' }});
               const dt = new DataTransfer();
               dt.items.add(file);
               el.dispatchEvent(new ClipboardEvent('paste', {{ clipboardData: dt, bubbles: true }}));
               return 'ok';
             }})()"
        ))
        .await
        .context("dispatch a synthetic paste")?;
    assert_eq!(dispatched, "ok", "the paste zone received the event");

    // The pasted image previews (read to a data URI).
    let preview = page.locator("[data-testid='ingest-screenshot']").await;
    expect(preview.clone())
        .to_have_count(1)
        .await
        .context("the pasted screenshot previews")?;
    let src = preview
        .get_attribute("src")
        .await?
        .unwrap_or_default();
    assert!(
        src.starts_with("data:image/"),
        "the preview is a data URI, got: {src}"
    );

    // Clear removes it.
    page.locator("[data-testid='ingest-clear']")
        .await
        .click(None)
        .await
        .context("clear the screenshot")?;
    expect(page.locator("[data-testid='ingest-screenshot']").await)
        .to_have_count(0)
        .await
        .context("clearing removes the preview")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

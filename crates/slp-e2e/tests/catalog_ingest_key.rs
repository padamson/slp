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
use common::{click_ft, dist_dir, measure_ppf, serve};
use playwright_rs::expect;
use playwright_rs::protocol::{DragToOptions, Page, Playwright, Position};

const KEY: &str = "sk-ant-e2e-secret-0123456789";

async fn open_catalog(page: &Page) -> Result<()> {
    page.locator("[data-testid='edit-catalog']")
        .click(None)
        .await
        .context("open the catalog inspector")?;
    Ok(())
}

/// Dispatch a real `ClipboardEvent` carrying an image `DataTransfer` at the
/// paste zone (Playwright can't populate the OS clipboard), so the app's actual
/// `on:paste` → FileReader path runs. The image is a canvas-generated 300×200
/// PNG rather than a 1×1 stub, so the crop stage renders with real pixels (a
/// drag needs geometry to move across).
async fn paste_screenshot(page: &Page) -> Result<()> {
    paste_screenshot_color(page, "#88aacc").await
}

/// Paste a synthetic screenshot filled with `fill` (a CSS color), so multiple
/// pastes can be told apart by their data URI.
async fn paste_screenshot_color(page: &Page, fill: &str) -> Result<()> {
    let r = page
        .evaluate_value(&format!(
            r#"(() => {{
                 const el = document.querySelector("[data-testid='ingest-paste']");
                 if (!el) return 'no-zone';
                 const canvas = document.createElement('canvas');
                 canvas.width = 300; canvas.height = 200;
                 const ctx = canvas.getContext('2d');
                 ctx.fillStyle = '{fill}'; ctx.fillRect(0, 0, 300, 200);
                 const b64 = canvas.toDataURL('image/png').split(',')[1];
                 const bytes = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
                 const file = new File([bytes], 'shot.png', {{ type: 'image/png' }});
                 const dt = new DataTransfer();
                 dt.items.add(file);
                 el.dispatchEvent(new ClipboardEvent('paste', {{ clipboardData: dt, bubbles: true }}));
                 return 'ok';
               }})()"#,
        ))
        .await
        .context("dispatch a synthetic paste")?;
    if r != "ok" {
        anyhow::bail!("paste dispatch failed: {r}");
    }
    Ok(())
}

#[tokio::test]
async fn the_api_key_gates_the_feature_persists_and_stays_out_of_the_plan() -> Result<()> {
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

    open_catalog(&page).await?;

    // Gated off until a key is entered.
    expect(page.locator("[data-testid='ingest-status']"))
        .to_have_text("Add your Anthropic API key to enable screenshot ingestion.")
        .await
        .context("the feature is gated off without a key")?;

    // Enter the key → the gate flips to enabled.
    page.locator("[data-testid='ingest-api-key']")
        .fill(KEY, None)
        .await
        .context("enter the API key")?;
    expect(page.locator("[data-testid='ingest-status']"))
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
    expect(page.locator("[data-testid='ingest-status']"))
        .to_have_text("Screenshot ingestion enabled.")
        .await
        .context("the key persisted across the reload")?;
    expect(page.locator("[data-testid='ingest-api-key']"))
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

    open_catalog(&page).await?;
    // The paste zone is gated on the key.
    page.locator("[data-testid='ingest-api-key']")
        .fill(KEY, None)
        .await
        .context("enter the API key")?;

    paste_screenshot(&page).await?;

    // The pasted image previews (read to a data URI).
    let preview = page.locator("[data-testid='ingest-screenshot']");
    expect(preview.clone())
        .to_have_count(1)
        .await
        .context("the pasted screenshot previews")?;
    let src = preview.get_attribute("src").await?.unwrap_or_default();
    assert!(
        src.starts_with("data:image/"),
        "the preview is a data URI, got: {src}"
    );

    // Clear removes it.
    page.locator("[data-testid='ingest-clear']")
        .click(None)
        .await
        .context("clear the screenshot")?;
    expect(page.locator("[data-testid='ingest-screenshot']"))
        .to_have_count(0)
        .await
        .context("clearing removes the preview")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

/// A product page is often several screenshots (colors, sizes, laying
/// patterns), so repeated pastes append to a list, each with its own remove,
/// and "Clear all" empties it (M4.6).
#[tokio::test]
async fn pasting_multiple_screenshots_lists_them_with_per_image_remove() -> Result<()> {
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

    open_catalog(&page).await?;
    page.locator("[data-testid='ingest-api-key']")
        .fill(KEY, None)
        .await
        .context("enter the API key")?;

    // Two ⌘V pastes → two thumbnails.
    paste_screenshot(&page).await?;
    paste_screenshot(&page).await?;
    expect(page.locator("[data-testid='ingest-screenshot']"))
        .to_have_count(2)
        .await
        .context("two pastes list two thumbnails")?;

    // Per-image remove drops just that one.
    page.locator("[data-testid='ingest-remove-0']")
        .click(None)
        .await
        .context("remove the first screenshot")?;
    expect(page.locator("[data-testid='ingest-screenshot']"))
        .to_have_count(1)
        .await
        .context("per-image remove leaves one")?;

    // Clear all empties the list.
    page.locator("[data-testid='ingest-clear']")
        .click(None)
        .await
        .context("clear all screenshots")?;
    expect(page.locator("[data-testid='ingest-screenshot']"))
        .to_have_count(0)
        .await
        .context("clear-all empties the list")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

/// When a color's bounding box names screenshot 1, its swatch is cropped out of
/// screenshot 1 — not screenshot 0 (M4.6 indexed crops).
#[tokio::test]
async fn a_swatch_crop_resolves_against_the_screenshot_its_box_names() -> Result<()> {
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

    // Stub the bridge: `extract` records the images it received and returns a
    // color whose box is on image index 1; `crop` records which image it cropped.
    page.evaluate_value(
        r#"(() => {
             window.__uris = null; window.__cropUri = null;
             window.slpVision = {
               extract: async (key, model, uris) => {
                 window.__uris = uris;
                 return JSON.stringify({
                   name: "Two-Shot Slab", category: "slab",
                   price_unit: "per_square_foot", unit_price: null,
                   colors: [{name:"Grey",available:true,bbox:{image:1,x:0.1,y:0.2,width:0.1,height:0.1}}],
                   textures: [], sizes: [], notes: null
                 });
               },
               crop: async (dataUri) => { window.__cropUri = dataUri; return "data:image/png;base64,CROPPED"; },
             };
             return 'ok';
           })()"#,
    )
    .await
    .context("stub the vision bridge")?;

    open_catalog(&page).await?;
    page.locator("[data-testid='ingest-api-key']")
        .fill(KEY, None)
        .await
        .context("enter the API key")?;
    // Two distinguishable screenshots.
    paste_screenshot_color(&page, "#112233").await?;
    paste_screenshot_color(&page, "#ccbbaa").await?;
    page.locator("[data-testid='ingest-extract']")
        .click(None)
        .await
        .context("run extraction")?;
    expect(page.locator("[data-testid='ingest-draft']"))
        .to_be_visible()
        .await
        .context("the draft appears")?;

    // The crop ran against screenshot 1 (the box's `image` index), not 0.
    let r = page
        .evaluate_value(
            "(() => (window.__cropUri && window.__cropUri === window.__uris[1]) ? 'match' \
             : (window.__cropUri === window.__uris[0] ? 'wrong-image' : 'no-crop'))()",
        )
        .await
        .context("read which screenshot was cropped")?;
    assert_eq!(
        r, "match",
        "the swatch was cropped from the indexed screenshot"
    );

    // Opening the crop editor for that color shows the screenshot its box names
    // (screenshot 1), not a blank stage.
    page.locator("[data-testid='ingest-color-swatch-0']")
        .click(None)
        .await
        .context("open the crop editor")?;
    expect(page.locator("[data-testid='crop-editor']"))
        .to_have_count(1)
        .await
        .context("the crop editor opens")?;
    let img_src = page
        .locator("[data-testid='crop-stage'] img.crop-image")
        .get_attribute("src")
        .await?
        .unwrap_or_default();
    assert!(
        img_src == uris_1(&page).await?,
        "the crop editor shows the indexed screenshot, got a {}-char src",
        img_src.len()
    );

    browser.close().await.context("close browser")?;
    Ok(())
}

/// Read `window.__uris[1]` (the second pasted screenshot's data URI).
async fn uris_1(page: &Page) -> Result<String> {
    Ok(page.evaluate_value("(() => window.__uris[1])()").await?)
}

/// The REAL crop bridge (`window.slpVision.crop`, not the stub the other tests
/// install) trims near-white border margins off a crop: product pages wrap
/// color chips in white cards, and an untrimmed sliver renders as white grid
/// lines when the swatch tiles a drawn area.
///
/// Uses the typed `Page::evaluate` (structured data out via serde) — one
/// parameterized probe instead of three string-packed `evaluate_value` blocks.
#[tokio::test]
async fn the_crop_bridge_trims_white_margins() -> Result<()> {
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

    /// What the crop bridge returned for a synthetic canvas: the cropped
    /// image's pixel dimensions.
    #[derive(serde::Deserialize)]
    struct Cropped {
        width: u32,
        height: u32,
    }
    /// The synthetic canvas to feed it: a `w`×`h` white sheet with an optional
    /// `[x, y, w, h]` textured block painted on it.
    #[derive(serde::Serialize)]
    struct Probe {
        w: u32,
        h: u32,
        fill: &'static str,
        block: Option<[u32; 4]>,
    }
    let crop = |probe: Probe| {
        let page = page.clone();
        async move {
            let out: Cropped = page
                .evaluate(
                    r#"async (p) => {
                         const src = document.createElement('canvas');
                         src.width = p.w; src.height = p.h;
                         const ctx = src.getContext('2d');
                         ctx.fillStyle = p.fill; ctx.fillRect(0, 0, p.w, p.h);
                         if (p.block) {
                           const [x, y, w, h] = p.block;
                           ctx.fillStyle = '#8899aa'; ctx.fillRect(x, y, w, h);
                         }
                         const out = await window.slpVision.crop(src.toDataURL('image/png'), 0, 0, 1, 1);
                         const img = new Image();
                         await new Promise((res, rej) => { img.onload = res; img.onerror = rej; img.src = out; });
                         return { width: img.naturalWidth, height: img.naturalHeight };
                       }"#,
                    Some(&probe),
                )
                .await?;
            Ok::<_, anyhow::Error>(out)
        }
    };

    // A 100×80 white image with a 60×50 textured block at (10, 12): the full
    // crop comes back trimmed to just the block.
    let out = crop(Probe {
        w: 100,
        h: 80,
        fill: "#ffffff",
        block: Some([10, 12, 60, 50]),
    })
    .await
    .context("crop a white-margined image")?;
    assert_eq!((out.width, out.height), (60, 50), "margins trimmed");

    // An all-dark image is untouched (nothing near-white to trim).
    let out = crop(Probe {
        w: 40,
        h: 30,
        fill: "#445566",
        block: None,
    })
    .await
    .context("crop a margin-free image")?;
    assert_eq!((out.width, out.height), (40, 30), "returned unchanged");

    // An all-white crop survives the trim cap (a light material can't be
    // eaten to nothing).
    let out = crop(Probe {
        w: 50,
        h: 50,
        fill: "#ffffff",
        block: None,
    })
    .await
    .context("crop an all-white image")?;
    assert!(
        out.width >= 20 && out.height >= 20,
        "the trim cap kept at least 40% per axis: {}x{}",
        out.width,
        out.height
    );

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn adjusting_a_swatch_crop_re_crops_it() -> Result<()> {
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

    // A counting crop stub so the re-crop returns a *different* URI than the one
    // produced during extraction — proving the adjustment re-cropped.
    page.evaluate_value(
        r#"(() => {
             let n = 0;
             window.slpVision = {
               extract: async () => JSON.stringify({
                 name: "Blu 60", category: "slab", price_unit: "per_square_foot",
                 colors: [{name:"Shale Grey",available:true,bbox:{x:0.1,y:0.1,width:0.1,height:0.1}}],
                 sizes: [{name:"60 MM",available:true,width_ft:1,depth_ft:1}]
               }),
               crop: async () => { n += 1; return "data:image/png;base64,CROP" + n; },
             };
             return "ok";
           })()"#,
    )
    .await
    .context("stub the vision bridge")?;

    open_catalog(&page).await?;
    page.locator("[data-testid='ingest-api-key']")
        .fill(KEY, None)
        .await?;
    paste_screenshot(&page).await?;
    page.locator("[data-testid='ingest-extract']")
        .click(None)
        .await
        .context("extract")?;

    // The extracted swatch is the first crop (CROP1). Click it to adjust.
    let swatch = page.locator("[data-testid='ingest-color-swatch-0']");
    expect(swatch.clone())
        .to_have_count(1)
        .await
        .context("the swatch shows")?;
    swatch.click(None).await.context("open the crop editor")?;
    expect(page.locator("[data-testid='crop-editor']"))
        .to_have_count(1)
        .await
        .context("the crop editor opens")?;

    // Drag the crop box deeper into the stage — a real held-button drag via
    // `Locator::drag_to`, engaging the box's pointer-capture handlers. The box's
    // `left` is bound straight to the `x` signal, so its computed `style.left`
    // reflects the drag. (The pixel→percent geometry itself is unit-tested
    // natively in slp-ui; this proves the gesture wiring end to end.)
    let read_left = || async {
        page.evaluate_value("document.querySelector(\"[data-testid='crop-box']\").style.left")
            .await
    };
    let initial_left = read_left().await.context("read the initial box position")?;
    let cbox = page.locator("[data-testid='crop-box']");
    let stage = page.locator("[data-testid='crop-stage']");
    cbox.drag_to(
        &stage,
        Some(
            DragToOptions::builder()
                .target_position(Position { x: 180.0, y: 120.0 })
                .build(),
        ),
    )
    .await
    .context("drag the crop box")?;
    let moved_left = read_left().await.context("read the dragged box position")?;
    assert_ne!(
        moved_left, initial_left,
        "dragging moved the crop box (was {initial_left}, now {moved_left})"
    );

    // Tighten the crop via a numeric input, then re-crop.
    page.locator("[data-testid='crop-w']")
        .fill("30", None)
        .await
        .context("widen the crop")?;
    page.locator("[data-testid='crop-apply']")
        .click(None)
        .await
        .context("use the new crop")?;
    expect(page.locator("[data-testid='crop-editor']"))
        .to_have_count(0)
        .await
        .context("the editor closes after applying")?;
    // The swatch now shows the re-cropped image (CROP2, not CROP1).
    let src = page
        .locator("[data-testid='ingest-color-swatch-0'] img")
        .get_attribute("src")
        .await?
        .unwrap_or_default();
    assert_eq!(
        src, "data:image/png;base64,CROP2",
        "the swatch was re-cropped"
    );

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn extracting_and_curating_a_screenshot_adds_catalog_items() -> Result<()> {
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

    // Stub the vision bridge with a canned extraction (no real API call, no
    // key needed) — this overrides the shell's `window.slpVision` after load.
    page.evaluate_value(
        r#"(() => {
             window.slpVision = {
               extract: async () => JSON.stringify({
                 name: "Blu 60 Slate Slabs", category: "slab",
                 price_unit: "per_square_foot", unit_price: null,
                 colors: [
                   {name:"Shale Grey",available:true,bbox:{x:0.1,y:0.2,width:0.1,height:0.1}},
                   {name:"Onyx Black",available:false}
                 ],
                 textures: [{name:"Slate",available:true}],
                 sizes: [{name:"60 MM",available:true,width_ft:1.083,depth_ft:1.083,thickness_in:2.375}],
                 notes: "No price listed."
               }),
               // Stub the swatch crop (real crop needs a canvas we don't exercise here).
               crop: async () => "data:image/png;base64,CROPPED",
             };
             return 'ok';
           })()"#,
    )
    .await
    .context("stub the vision bridge")?;

    open_catalog(&page).await?;
    page.locator("[data-testid='ingest-api-key']")
        .fill(KEY, None)
        .await
        .context("enter the API key")?;
    paste_screenshot(&page).await?;

    // The extract action appears once a screenshot is pasted.
    let extract = page.locator("[data-testid='ingest-extract']");
    expect(extract.clone())
        .to_have_count(1)
        .await
        .context("the extract button appears")?;
    extract.click(None).await.context("run extraction")?;

    // The draft product renders from the canned extraction.
    let draft = page.locator("[data-testid='ingest-draft']");
    expect(draft.clone())
        .to_have_count(1)
        .await
        .context("the draft appears")?;
    expect(draft.clone())
        .to_contain_text("Blu 60 Slate Slabs")
        .await
        .context("the product name")?;
    expect(draft.clone())
        .to_contain_text("Onyx Black")
        .await
        .context("the variant matrix (with the unavailable color)")?;
    // The color's swatch was cropped from the screenshot and previews.
    expect(page.locator("[data-testid='ingest-color-swatch-0']"))
        .to_have_count(1)
        .await
        .context("the cropped swatch thumbnail shows")?;

    // Approve curation: the available color × size combo (Shale Grey × 60 MM)
    // becomes a catalog item; the draft closes.
    let approve = page.locator("[data-testid='ingest-approve']");
    expect(approve.clone())
        .to_have_text("Add 1 to catalog")
        .await
        .context("the count reflects the ticked combos")?;
    approve.click(None).await.context("approve")?;
    expect(page.locator("[data-testid='catalog-row-blu-60-slate-slabs-shale-grey-60-mm']"))
        .to_have_count(1)
        .await
        .context("the curated item is in the catalog")?;
    expect(page.locator("[data-testid='ingest-draft']"))
        .to_have_count(0)
        .await
        .context("the draft closes after approving")?;

    // The ingestion payoff: the new material's category is now armable in the
    // Area tool's (catalog-driven) picker, so it can price + tile a drawn area.
    expect(page.locator("[data-testid='area-mat-cat-slab']"))
        .to_have_count(1)
        .await
        .context("the ingested material's category appears in the Area picker")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

/// End to end: a product extracted **with laying patterns** lists them as
/// ticked checkboxes with diagram thumbnails in curation; the approved item
/// carries them; a drawn area of that material can pick one — the inspector
/// shows the diagram and the estimate line notes the layout.
#[tokio::test]
async fn laying_patterns_ride_curation_and_a_drawn_area_picks_one() -> Result<()> {
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

    // A canned extraction with a priced slab (so the estimate lines appear)
    // and two laying patterns, Herringbone carrying a diagram bbox.
    page.evaluate_value(
        r#"(() => {
             window.slpVision = {
               extract: async () => JSON.stringify({
                 name: "Blu 60 Slate Slabs", category: "slab",
                 price_unit: "per_square_foot", unit_price: 9.0,
                 colors: [{name:"Shale Grey",available:true,bbox:{x:0.1,y:0.2,width:0.1,height:0.1}}],
                 textures: [],
                 sizes: [{name:"60 MM",available:true,width_ft:1.083,depth_ft:1.083,thickness_in:2.375}],
                 patterns: [
                   {name:"Herringbone",bbox:{image:0,x:0.5,y:0.5,width:0.2,height:0.2}},
                   {name:"Linear"}
                 ],
                 notes: null
               }),
               crop: async () => "data:image/png;base64,CROPPED",
             };
             return 'ok';
           })()"#,
    )
    .await
    .context("stub the vision bridge")?;

    open_catalog(&page).await?;
    page.locator("[data-testid='ingest-api-key']")
        .fill(KEY, None)
        .await
        .context("enter the API key")?;
    paste_screenshot(&page).await?;
    page.locator("[data-testid='ingest-extract']")
        .click(None)
        .await
        .context("run extraction")?;

    // Curation lists the patterns, ticked, Herringbone with its cropped diagram.
    expect(page.locator("[data-testid='ingest-pattern-0']"))
        .to_be_checked()
        .await
        .context("herringbone starts ticked")?;
    expect(page.locator("[data-testid='ingest-pattern-1']"))
        .to_be_checked()
        .await
        .context("linear starts ticked")?;
    expect(page.locator("[data-testid='ingest-pattern-diagram-0']"))
        .to_have_count(1)
        .await
        .context("the cropped diagram thumbnail shows")?;
    page.locator("[data-testid='ingest-approve']")
        .click(None)
        .await
        .context("approve")?;
    page.locator("[data-testid='catalog-close']")
        .click(None)
        .await
        .context("close the catalog panel")?;

    // Draw a slab area (10×8) with the ingested material and select it.
    let yard = page.locator("[data-testid='yard']");
    let ppf = measure_ppf(&yard).await?;
    page.locator("[data-testid='area-mat-cat-slab']")
        .click(None)
        .await
        .context("arm the ingested slab material")?;
    page.locator("[data-testid='draw-shape']")
        .click(None)
        .await
        .context("arm the area tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close
    click_ft(&yard, ppf, 12.0, 11.0).await?; // select the area

    // The inspector offers the material's patterns; pick Herringbone → its
    // diagram shows and the estimate line notes the layout.
    let select = page.locator("[data-testid='area-pattern']");
    expect(select.clone())
        .to_have_count(1)
        .await
        .context("the pattern select appears for a patterned material")?;
    select
        .select_option("Herringbone", None)
        .await
        .context("choose herringbone")?;
    expect(page.locator("[data-testid='area-pattern-diagram']"))
        .to_have_count(1)
        .await
        .context("the chosen pattern's diagram shows in the inspector")?;
    expect(page.locator("[data-testid='estimate-pattern']"))
        .to_have_text("(Herringbone)")
        .await
        .context("the estimate line notes the chosen layout")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

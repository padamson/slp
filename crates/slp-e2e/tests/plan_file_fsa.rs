//! G1.2/G1.3 e2e: the File System Access API path — named Save As, in-place
//! Save, Open, and reopen-the-last-file-on-startup (silent when permission
//! stands, one-click "Reopen" otherwise).
//!
//! The native pickers can't be clicked, so we install a deterministic fake
//! `showSaveFilePicker`/`showOpenFilePicker` + a minimal fake IndexedDB via
//! `add_init_script` before the app boots (the standard Playwright FSA
//! pattern). The fake backs files + the remembered handle name in
//! `localStorage` so they survive a reload, and returns *live* handles (methods
//! intact) — the app's real `window.slpFs` + `fs_access` code runs unchanged
//! against it.
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

/// A fake File System Access API + IndexedDB, installed at document start.
const FAKE_FS: &str = r#"
(() => {
  const FK = '__fake_files', HK = '__fake_handle', PK = '__fake_perm', OK = '__fake_open';
  const files = () => JSON.parse(localStorage.getItem(FK) || '{}');
  const content = (n) => { const c = files()[n]; return c == null ? null : c; };
  const write = (n, c) => { const f = files(); f[n] = c; localStorage.setItem(FK, JSON.stringify(f)); };

  const makeHandle = (name) => ({
    __fakeHandle: true, kind: 'file', name,
    async createWritable() {
      let buf = '';
      return { async write(t) { buf += t; }, async close() { write(name, buf); } };
    },
    async getFile() { const c = content(name); return { text: async () => (c == null ? '' : c) }; },
    async queryPermission() { return localStorage.getItem(PK) || 'granted'; },
    async requestPermission() { localStorage.setItem(PK, 'granted'); return 'granted'; },
  });

  window.__saveCalls = 0;
  window.showSaveFilePicker = async (opts) => {
    window.__saveCalls++;
    return makeHandle((opts && opts.suggestedName) || 'untitled.slp.json');
  };
  window.showOpenFilePicker = async () => [makeHandle(localStorage.getItem(OK) || 'opened.slp.json')];

  const fire = (req, setup) => Promise.resolve().then(() => {
    if (setup) setup(req);
    if (req.onsuccess) req.onsuccess({ target: req });
  });
  // window.indexedDB is a read-only accessor — must be replaced via
  // defineProperty, not assignment (which silently no-ops and leaves the real
  // IndexedDB, whose structuredClone rejects a method-bearing fake handle).
  Object.defineProperty(window, 'indexedDB', {
    configurable: true,
    value: {
      open() {
        const req = {};
        const store = {
          put(v) { localStorage.setItem(HK, v.name); const r = {}; fire(r); return r; },
          get() { const r = {}; fire(r, (x) => { const n = localStorage.getItem(HK); x.result = n ? makeHandle(n) : undefined; }); return r; },
        };
        const tx = { objectStore: () => store };
        Object.defineProperty(tx, 'oncomplete', { set(f) { Promise.resolve().then(() => f && f()); } });
        const db = { createObjectStore() { return store; }, close() {}, transaction: () => tx };
        fire(req, (x) => { x.result = db; });
        return req;
      },
    },
  });
})();
"#;

/// A serialized plan with a distinctive yard width, for seeding an Open target.
fn plan_json(width: f64) -> String {
    format!(r#"{{"yard_width": {width}, "yard_depth": 30.0}}"#)
}

async fn boot(page: &Page, addr: &std::net::SocketAddr) -> Result<()> {
    page.add_init_script(FAKE_FS)
        .await
        .context("install the fake File System Access API")?;
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
        .to_have_text("landscape-plan.slp.json")
        .await
        .context("the current file name shows")?;
    let saved = page
        .evaluate_value(
            "JSON.parse(localStorage.getItem('__fake_files'))['landscape-plan.slp.json']",
        )
        .await
        .context("read the written file")?;
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
        let now = page
            .evaluate_value(
                "JSON.parse(localStorage.getItem('__fake_files'))['landscape-plan.slp.json']",
            )
            .await
            .unwrap_or_default();
        if now.contains("50") && !now.contains("42.5") {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let calls = page
        .evaluate_value("String(window.__saveCalls)")
        .await
        .context("read save-picker call count")?;
    assert_eq!(
        calls, "1",
        "Save reused the handle (only Save As showed a picker)"
    );

    // Open a *different* seeded file → its plan loads.
    page.evaluate_value(&format!(
        "(() => {{ const f = JSON.parse(localStorage.getItem('__fake_files')||'{{}}'); \
         f['other.slp.json'] = {plan}; localStorage.setItem('__fake_files', JSON.stringify(f)); \
         localStorage.setItem('__fake_open','other.slp.json'); return 'ok'; }})()",
        plan = serde_json_str(&plan_json(77.0)),
    ))
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
        .to_have_text("landscape-plan.slp.json")
        .await?;
    page.locator("[data-testid='yard-width']")
        .fill("99", None)
        .await
        .context("diverge the autosave to 99")?;

    page.reload(None).await.context("reload")?;

    expect(page.locator("[data-testid='yard-width']"))
        .to_have_value("42.5")
        .await
        .context("startup silently reopened the file (overriding the 99 autosave)")?;
    expect(page.locator("[data-testid='current-file']"))
        .to_have_text("landscape-plan.slp.json")
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
    boot(&page, &addr).await?;

    page.locator("[data-testid='yard-width']")
        .fill("42.5", None)
        .await?;
    page.locator("[data-testid='save-plan-as']")
        .click(None)
        .await
        .context("Save As")?;
    expect(page.locator("[data-testid='current-file']"))
        .to_have_text("landscape-plan.slp.json")
        .await?;
    // Permission lapses; diverge the autosave so a silent load would be visible.
    page.evaluate_value("(() => { localStorage.setItem('__fake_perm','prompt'); return 'ok'; })()")
        .await
        .context("lapse the read permission")?;
    page.locator("[data-testid='yard-width']")
        .fill("99", None)
        .await?;

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

/// Minimal JSON string literal encoder for embedding one JSON doc inside a JS
/// expression (escapes `"` and `\`).
fn serde_json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

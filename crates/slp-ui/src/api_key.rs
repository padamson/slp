//! The user's Anthropic API key for screenshot ingestion (Phase B). Stored in
//! `localStorage` as **app/browser config — deliberately not part of the
//! `Plan`** — so an exported/shared `.slp.json` never carries a billable
//! secret. The vision "extract from screenshot" feature gates on whether a key
//! is present.
//!
//! Everything here is `csr`-gated: `localStorage` exists only in the browser
//! build. Under `ssr`/native tests the entry points are inert (no key), so the
//! crate still compiles and renders.

/// `localStorage` key for the API key — namespaced, and distinct from the
/// plan's own storage key so the two never collide.
pub const API_KEY_STORAGE: &str = "slp.anthropicKey";

/// The stored API key, or `None` when unset/empty. On the browser this reads
/// `localStorage`; if that's empty it falls back to the **dev-only** build-time
/// seed (see [`dev_seed`]), persisting it so later reads are consistent. Always
/// `None` off the browser.
#[must_use]
pub fn api_key() -> Option<String> {
    #[cfg(feature = "csr")]
    {
        if let Some(k) = storage()
            .and_then(|s| s.get_item(API_KEY_STORAGE).ok().flatten())
            .filter(|k| !k.is_empty())
        {
            return Some(k);
        }
        // Empty store: seed from the dev build-time key once, if present.
        if let Some(seed) = dev_seed() {
            set_api_key(seed);
            return Some(seed.to_string());
        }
        None
    }
    #[cfg(not(feature = "csr"))]
    {
        None
    }
}

/// Persist `key` (trimmed) as the API key; an empty/blank value clears it.
/// No-op off the browser.
pub fn set_api_key(key: &str) {
    #[cfg(feature = "csr")]
    {
        if let Some(s) = storage() {
            let trimmed = key.trim();
            if trimmed.is_empty() {
                let _ = s.remove_item(API_KEY_STORAGE);
            } else {
                let _ = s.set_item(API_KEY_STORAGE, trimmed);
            }
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = key;
    }
}

/// A compile-time key baked in for **local dev only** — a `trunk serve` build
/// with `SLP_ANTHROPIC_KEY` set (e.g. sourced from a gitignored `.env`) so the
/// key doesn't have to be re-typed in a fresh browser profile. `None` in any
/// build that didn't set the env var.
///
/// This MUST never be set in the hosted / CI build: `option_env!` bakes the
/// value into the WASM artifact, so a set key would ship publicly. The hosted
/// build simply leaves `SLP_ANTHROPIC_KEY` unset, and this returns `None`.
#[cfg(feature = "csr")]
fn dev_seed() -> Option<&'static str> {
    option_env!("SLP_ANTHROPIC_KEY").filter(|k| !k.is_empty())
}

#[cfg(feature = "csr")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

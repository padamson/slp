//! Thin async bridge to the app shell's File System Access API helper
//! (`window.slpFs`, defined in `slp-app/index.html`) for named Save / Save As,
//! in-place Save, and reopen-the-last-file-on-startup (G1.2/G1.3).
//!
//! Everything degrades to `unsupported` / `None` when the shell doesn't provide
//! `slpFs` — dokime/ssr, a browser without the File System Access API, or a
//! consumer that didn't include the bridge — so the caller falls back to the
//! portable download / `<input type="file">` path. `slpFs`'s methods return a
//! JSON string (or `null`), parsed here into typed results.

/// A file the FSA layer opened/reopened: its display name and its text (the
/// plan JSON, to be validated by [`crate::plan_file::parse_plan`]).
pub struct Loaded {
    pub name: String,
    pub plan: String,
}

/// The outcome of a startup reopen attempt.
pub enum Reopen {
    /// The last file loaded silently (read permission still stood).
    Loaded(Loaded),
    /// A file is remembered but reading it needs a user gesture — carries its
    /// name, for the "Reopen &lt;name&gt;" affordance.
    NeedsGesture(String),
}

/// Parse `{name, plan}` from a `slpFs` result string.
fn parse_loaded(s: &str) -> Option<Loaded> {
    let v: serde_json::Value = serde_json::from_str(s).ok()?;
    Some(Loaded {
        name: v.get("name")?.as_str()?.to_string(),
        plan: v.get("plan")?.as_str()?.to_string(),
    })
}

/// Parse just the `name` from a `slpFs` result string (Save / Save As).
fn parse_name(s: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(s).ok()?;
    Some(v.get("name")?.as_str()?.to_string())
}

/// Parse a `reopen()` result (`{name, plan?, silent}`) into a [`Reopen`].
fn parse_reopen(s: &str) -> Option<Reopen> {
    let v: serde_json::Value = serde_json::from_str(s).ok()?;
    let name = v.get("name")?.as_str()?.to_string();
    let silent = v
        .get("silent")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    match v.get("plan").and_then(serde_json::Value::as_str) {
        Some(plan) if silent => Some(Reopen::Loaded(Loaded {
            name,
            plan: plan.to_string(),
        })),
        _ => Some(Reopen::NeedsGesture(name)),
    }
}

#[cfg(feature = "csr")]
mod imp {
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    /// `window.slpFs`, if the shell defined it.
    fn slpfs() -> Option<JsValue> {
        let win = web_sys::window()?;
        let fs = js_sys::Reflect::get(&win, &JsValue::from_str("slpFs")).ok()?;
        (!fs.is_undefined() && !fs.is_null()).then_some(fs)
    }

    /// Call a synchronous `slpFs` method returning its `JsValue`.
    fn call_sync(method: &str) -> Option<JsValue> {
        let fs = slpfs()?;
        let f = js_sys::Reflect::get(&fs, &JsValue::from_str(method)).ok()?;
        f.dyn_ref::<js_sys::Function>()?.call0(&fs).ok()
    }

    /// Call an async `slpFs` method, await its promise, return the result string
    /// (or `None` when the method returned `null`/threw or `slpFs` is absent).
    async fn call_str(method: &str, args: &[JsValue]) -> Option<String> {
        let fs = slpfs()?;
        let f = js_sys::Reflect::get(&fs, &JsValue::from_str(method)).ok()?;
        let func = f.dyn_ref::<js_sys::Function>()?;
        let promise = match args {
            [] => func.call0(&fs),
            [a] => func.call1(&fs, a),
            [a, b] => func.call2(&fs, a, b),
            _ => return None,
        }
        .ok()?;
        let promise: js_sys::Promise = promise.dyn_into().ok()?;
        JsFuture::from(promise).await.ok()?.as_string()
    }

    pub fn supported() -> bool {
        call_sync("supported")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub async fn save(text: &str) -> Option<String> {
        call_str("save", &[JsValue::from_str(text)]).await
    }

    pub async fn save_as(text: &str, suggested: &str) -> Option<String> {
        call_str(
            "saveAs",
            &[JsValue::from_str(text), JsValue::from_str(suggested)],
        )
        .await
    }

    pub async fn open() -> Option<String> {
        call_str("open", &[]).await
    }

    pub async fn reopen() -> Option<String> {
        call_str("reopen", &[]).await
    }

    pub async fn reopen_grant() -> Option<String> {
        call_str("reopenGrant", &[]).await
    }
}

#[cfg(not(feature = "csr"))]
mod imp {
    pub fn supported() -> bool {
        false
    }
    pub async fn save(_text: &str) -> Option<String> {
        None
    }
    pub async fn save_as(_text: &str, _suggested: &str) -> Option<String> {
        None
    }
    pub async fn open() -> Option<String> {
        None
    }
    pub async fn reopen() -> Option<String> {
        None
    }
    pub async fn reopen_grant() -> Option<String> {
        None
    }
}

/// Whether the File System Access bridge is available (a named-file Save/Open
/// with a handle SLP can remember); `false` falls back to download/`<input>`.
pub fn supported() -> bool {
    imp::supported()
}

/// Write to the current file with no dialog; `None` when there's no current
/// file yet (the caller should [`save_as`] instead) or the write failed.
/// Returns the file name written.
pub async fn save(text: &str) -> Option<String> {
    parse_name(&imp::save(text).await?)
}

/// Prompt for a name/location, write, and adopt the file as current; `None` if
/// the user cancelled. Returns the chosen file name.
pub async fn save_as(text: &str, suggested: &str) -> Option<String> {
    parse_name(&imp::save_as(text, suggested).await?)
}

/// Pick a file to open and adopt it as current; `None` if cancelled.
pub async fn open() -> Option<Loaded> {
    parse_loaded(&imp::open().await?)
}

/// On startup: the last file loaded silently, or a name that needs a gesture,
/// or `None` when no file is remembered.
pub async fn reopen() -> Option<Reopen> {
    parse_reopen(&imp::reopen().await?)
}

/// The "Reopen &lt;name&gt;" gesture: request permission, then read; `None` if
/// denied.
pub async fn reopen_grant() -> Option<Loaded> {
    parse_loaded(&imp::reopen_grant().await?)
}

/// Convenience for the Save button: write in place, or Save As when there's no
/// current file. Returns the file name written, or `None` if cancelled.
pub async fn save_or_save_as(text: &str, suggested: &str) -> Option<String> {
    if let Some(name) = save(text).await {
        return Some(name);
    }
    save_as(text, suggested).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_loaded_file() {
        let l = parse_loaded(r#"{"name":"yard.slp.json","plan":"{\"yard_width\":40}"}"#).unwrap();
        assert_eq!(l.name, "yard.slp.json");
        assert_eq!(l.plan, r#"{"yard_width":40}"#);
    }

    #[test]
    fn parses_a_saved_name() {
        assert_eq!(
            parse_name(r#"{"name":"a.slp.json"}"#).as_deref(),
            Some("a.slp.json")
        );
    }

    #[test]
    fn a_silent_reopen_carries_the_plan() {
        let r = parse_reopen(r#"{"name":"a.slp.json","plan":"{}","silent":true}"#).unwrap();
        assert!(matches!(r, Reopen::Loaded(l) if l.name == "a.slp.json" && l.plan == "{}"));
    }

    #[test]
    fn a_non_silent_reopen_needs_a_gesture() {
        let r = parse_reopen(r#"{"name":"a.slp.json","silent":false}"#).unwrap();
        assert!(matches!(r, Reopen::NeedsGesture(n) if n == "a.slp.json"));
    }

    #[test]
    fn a_reopen_with_a_plan_but_not_silent_still_needs_a_gesture() {
        // Defensive: silent=false wins even if a plan slipped in.
        let r = parse_reopen(r#"{"name":"a.slp.json","plan":"{}","silent":false}"#).unwrap();
        assert!(matches!(r, Reopen::NeedsGesture(_)));
    }
}

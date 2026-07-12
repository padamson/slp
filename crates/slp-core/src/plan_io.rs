//! Plan-file naming: turn a `Plan`'s name into a safe download filename. The
//! serialization itself is plain `serde_json` over `Plan` (the file *is* the
//! plan) and lives in the UI's browser I/O; only the filename-derivation logic
//! is headless enough to unit/mutation-test here.

use crate::generated::slp::Plan;

/// The file extension every saved plan carries.
pub const PLAN_EXT: &str = ".slp.json";

/// The filename used when a plan has no usable name.
pub const DEFAULT_PLAN_STEM: &str = "landscape-plan";

/// A safe download filename for `plan`: its `name` slugified (lowercased,
/// non-alphanumerics collapsed to single dashes, trimmed), falling back to
/// [`DEFAULT_PLAN_STEM`] when the name is absent or slugifies to nothing —
/// always suffixed with [`PLAN_EXT`].
#[must_use]
pub fn plan_filename(plan: &Plan) -> String {
    let stem = plan
        .name
        .as_deref()
        .map(slugify)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_PLAN_STEM.to_string());
    format!("{stem}{PLAN_EXT}")
}

/// Lowercase `name`, map each run of non-`[a-z0-9]` characters to a single
/// dash, and trim leading/trailing dashes — a filesystem-safe stem.
fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut pending_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: Option<&str>) -> Plan {
        Plan {
            name: name.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn a_named_plan_slugifies_into_the_filename() {
        assert_eq!(
            plan_filename(&named(Some("Back Yard"))),
            "back-yard.slp.json"
        );
    }

    #[test]
    fn punctuation_and_runs_collapse_to_single_dashes() {
        assert_eq!(
            plan_filename(&named(Some("Paul & Wife's  Yard!!"))),
            "paul-wife-s-yard.slp.json"
        );
    }

    #[test]
    fn leading_and_trailing_non_alphanumerics_are_trimmed() {
        assert_eq!(
            plan_filename(&named(Some("  *North 40* "))),
            "north-40.slp.json"
        );
    }

    #[test]
    fn an_absent_name_uses_the_default_stem() {
        assert_eq!(plan_filename(&named(None)), "landscape-plan.slp.json");
    }

    #[test]
    fn a_name_that_slugifies_to_nothing_uses_the_default_stem() {
        // All punctuation → empty slug → the default, never a bare ".slp.json".
        assert_eq!(
            plan_filename(&named(Some("!!! ---"))),
            "landscape-plan.slp.json"
        );
    }

    #[test]
    fn digits_and_letters_are_kept_lowercased() {
        assert_eq!(plan_filename(&named(Some("Plan A2"))), "plan-a2.slp.json");
    }
}

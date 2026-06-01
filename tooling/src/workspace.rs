//! Workspace discovery and structured-file helpers.
//!
//! Per the contract, `mw` owns the shape of `workspace.yaml` and edits it with
//! anchored single-line operations rather than a full YAML parser (the
//! unmaintained `serde_yaml` crate is deliberately avoided). JSON files use
//! `serde_json`; YAML stays token/line-based.

use std::path::{Path, PathBuf};

/// Marker file that identifies a meta-workspace root.
pub const WORKSPACE_FILE: &str = "workspace.yaml";

/// Locate the workspace root by walking up from `start` until `workspace.yaml`
/// is found. Returns the directory containing it.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(d) = dir {
        if d.join(WORKSPACE_FILE).is_file() {
            return Some(d.to_path_buf());
        }
        dir = d.parent();
    }
    None
}

/// Locate the workspace root from the current directory.
pub fn find_root_from_cwd() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    find_root(&cwd).ok_or_else(|| {
        anyhow::anyhow!("not inside a meta-workspace (no {WORKSPACE_FILE} found above {cwd:?})")
    })
}

/// Read a top-level-ish scalar value for a `key:` line.
///
/// This is intentionally simple: it returns the trimmed value after the first
/// `key:` match, ignoring quotes and inline comments. It is meant for the
/// well-known keys that `mw` owns, not arbitrary user YAML.
///
/// ```
/// use meta_workspace::workspace::read_scalar;
/// let yaml = "memory:\n  profile: none # none | mempalace | prism | full\n";
/// assert_eq!(read_scalar(yaml, "profile").as_deref(), Some("none"));
/// assert_eq!(read_scalar(yaml, "missing"), None);
/// ```
pub fn read_scalar(yaml: &str, key: &str) -> Option<String> {
    let needle = format!("{key}:");
    for line in yaml.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(&needle) {
            let mut value = rest.trim();
            // strip a trailing inline comment that is not inside quotes
            if !value.starts_with('"')
                && !value.starts_with('\'')
                && let Some(idx) = value.find(" #")
            {
                value = value[..idx].trim();
            }
            let value = value.trim_matches(|c| c == '"' || c == '\'');
            if value.is_empty() {
                return None;
            }
            return Some(value.to_string());
        }
    }
    None
}

/// Replace the value of an anchored `indent + key:` line, preserving indentation
/// and any trailing inline comment. Returns the rewritten document, or an error
/// if the key line is not found.
///
/// ```
/// use meta_workspace::workspace::set_scalar;
/// let yaml = "memory:\n  profile: none # comment\n";
/// let out = set_scalar(yaml, "profile", "prism").unwrap();
/// assert!(out.contains("  profile: prism # comment"));
/// ```
pub fn set_scalar(yaml: &str, key: &str, value: &str) -> anyhow::Result<String> {
    let needle = format!("{key}:");
    let mut out = String::with_capacity(yaml.len() + value.len());
    let mut replaced = false;
    for line in yaml.lines() {
        let trimmed = line.trim_start();
        if !replaced && trimmed.starts_with(&needle) {
            let indent = &line[..line.len() - trimmed.len()];
            let comment = trimmed
                .find(" #")
                .map(|i| trimmed[i..].to_string())
                .unwrap_or_default();
            out.push_str(indent);
            out.push_str(&needle);
            out.push(' ');
            out.push_str(value);
            out.push_str(&comment);
            out.push('\n');
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        anyhow::bail!("key `{key}` not found in workspace.yaml");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "workspace:\n  schemaVersion: 1\n  name: meta-workspace\nmemory:\n  profile: none # none | mempalace | prism | full\n";

    #[test]
    fn reads_scalar() {
        assert_eq!(
            read_scalar(SAMPLE, "name").as_deref(),
            Some("meta-workspace")
        );
        assert_eq!(read_scalar(SAMPLE, "profile").as_deref(), Some("none"));
        assert_eq!(read_scalar(SAMPLE, "schemaVersion").as_deref(), Some("1"));
        assert_eq!(read_scalar(SAMPLE, "missing"), None);
    }

    #[test]
    fn reads_scalar_strips_quotes_and_inline_comment() {
        let y = "a: \"hello world\"\nb: bare # trailing comment\nc:\n";
        assert_eq!(read_scalar(y, "a").as_deref(), Some("hello world"));
        assert_eq!(read_scalar(y, "b").as_deref(), Some("bare"));
        // empty value reads as None
        assert_eq!(read_scalar(y, "c"), None);
    }

    #[test]
    fn set_scalar_is_idempotent_roundtrip() {
        let once = set_scalar(SAMPLE, "profile", "prism").unwrap();
        let twice = set_scalar(&once, "profile", "prism").unwrap();
        assert_eq!(once, twice);
        assert_eq!(read_scalar(&once, "profile").as_deref(), Some("prism"));
    }

    #[test]
    fn sets_scalar_preserving_comment_and_indent() {
        let out = set_scalar(SAMPLE, "profile", "mempalace").unwrap();
        assert!(out.contains("  profile: mempalace # none | mempalace | prism | full"));
    }

    #[test]
    fn set_missing_key_errors() {
        assert!(set_scalar(SAMPLE, "nope", "x").is_err());
    }
}

//! `mw policy check` — the cross-harness enforcement engine.
//!
//! Reads a single event as JSON on stdin and returns a decision as JSON on
//! stdout. The canonical event shape follows Claude Code's hook payload so
//! Claude needs zero translation; other harnesses translate in their thin
//! adapter (Pi extension, Codex config subset, Gemini instructions).
//!
//! Decision is one of: allow | deny{reason} | modify{input} | warn{message}.
//! The policy file is `.agents/policies.yaml`; parsing stays intentionally
//! line-based so the CLI does not depend on a YAML runtime.

use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::cli::{PolicyAction, PolicyArgs};
use crate::workspace;

/// Subset of the Claude Code hook payload that `mw` reasons about. Unknown
/// fields are ignored so the engine tolerates richer payloads.
#[derive(Debug, Default, Deserialize)]
struct Event {
    #[serde(default)]
    hook_event_name: String,
    #[serde(default)]
    tool_name: String,
    #[serde(default)]
    tool_input: serde_json::Value,
}

/// Harness-neutral decision returned on stdout.
#[derive(Debug, Serialize)]
#[serde(tag = "decision", rename_all = "lowercase")]
enum Decision {
    Allow,
    Deny {
        reason: String,
    },
    // Part of the wire protocol; not yet emitted by the built-in brain.
    #[allow(dead_code)]
    Warn {
        message: String,
    },
}

pub fn run(args: PolicyArgs) -> anyhow::Result<()> {
    match args.action {
        PolicyAction::Check => check(),
    }
}

fn check() -> anyhow::Result<()> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw)?;

    // A malformed or empty event must not crash a tool call: default to allow.
    let event: Event = if raw.trim().is_empty() {
        Event::default()
    } else {
        serde_json::from_str(&raw).unwrap_or_default()
    };

    let policy = load_policy();
    let decision = evaluate(&event, &policy);
    println!("{}", serde_json::to_string(&decision)?);

    // Exit code mirrors the decision so harnesses without JSON parsing (e.g. a
    // simple Codex/Gemini shell shim) can still react: 0 allow/warn, 1 deny.
    if matches!(decision, Decision::Deny { .. }) {
        std::process::exit(1);
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct Policy {
    protect_paths_enabled: bool,
    deny_write: Vec<String>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            protect_paths_enabled: true,
            deny_write: vec![".env".into(), ".env.*".into(), "secrets/".into()],
        }
    }
}

fn load_policy() -> Policy {
    let Some(root) = workspace::find_root_from_cwd().ok() else {
        return Policy::default();
    };
    let path = root.join(".agents/policies.yaml");
    let Ok(content) = std::fs::read_to_string(path) else {
        return Policy::default();
    };
    parse_policy(&content).unwrap_or_default()
}

fn parse_policy(content: &str) -> Option<Policy> {
    let mut policy = Policy::default();
    let mut in_protect_paths = false;
    let mut in_deny_write = false;
    let mut deny_write: Vec<String> = Vec::new();

    for raw in content.lines() {
        let line = raw.split('#').next().unwrap_or("").trim_end();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !raw.starts_with(' ') && trimmed.ends_with(':') {
            in_protect_paths = trimmed == "protect_paths:";
            in_deny_write = false;
            continue;
        }
        if !in_protect_paths {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("enabled:") {
            policy.protect_paths_enabled = value.trim() != "false";
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("deny_write:") {
            in_deny_write = true;
            let value = value.trim();
            if value.starts_with('[') && value.ends_with(']') {
                deny_write.extend(
                    value
                        .trim_matches(['[', ']'])
                        .split(',')
                        .map(clean_scalar)
                        .filter(|s| !s.is_empty()),
                );
            }
            continue;
        }
        if in_deny_write {
            if let Some(item) = trimmed.strip_prefix('-') {
                let item = clean_scalar(item.trim());
                if !item.is_empty() {
                    deny_write.push(item);
                }
            } else if !raw.starts_with("    ") {
                in_deny_write = false;
            }
        }
    }

    if !deny_write.is_empty() {
        policy.deny_write = deny_write;
    }
    Some(policy)
}

fn clean_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn evaluate(event: &Event, policy: &Policy) -> Decision {
    if policy.protect_paths_enabled && is_write_tool(&event.tool_name) {
        if let Some(path) = target_path(&event.tool_input) {
            if is_protected(&path, &policy.deny_write) {
                return Decision::Deny {
                    reason: format!("writing to protected path is not allowed: {path}"),
                };
            }
        }
    }
    let _ = &event.hook_event_name;
    Decision::Allow
}

fn is_write_tool(tool: &str) -> bool {
    matches!(tool, "Write" | "Edit" | "MultiEdit" | "NotebookEdit")
}

fn target_path(input: &serde_json::Value) -> Option<String> {
    input
        .get("file_path")
        .or_else(|| input.get("path"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn is_protected(path: &str, patterns: &[String]) -> bool {
    let normalized = path.replace('\\', "/");
    let name = normalized.rsplit('/').next().unwrap_or(&normalized);
    patterns
        .iter()
        .any(|pattern| matches_pattern(&normalized, name, pattern))
}

fn matches_pattern(path: &str, name: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('/') {
        return path.split('/').any(|part| part == prefix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return name.starts_with(prefix) && name != ".env.example";
    }
    name == pattern || path == pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(tool: &str, path: &str) -> Event {
        Event {
            tool_name: tool.into(),
            tool_input: serde_json::json!({ "file_path": path }),
            ..Default::default()
        }
    }

    #[test]
    fn denies_env_write() {
        let policy = Policy::default();
        assert!(matches!(
            evaluate(&ev("Write", "/x/.env"), &policy),
            Decision::Deny { .. }
        ));
        assert!(matches!(
            evaluate(&ev("Edit", "secrets/key"), &policy),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn allows_normal_write() {
        let policy = Policy::default();
        assert!(matches!(
            evaluate(&ev("Write", "src/main.rs"), &policy),
            Decision::Allow
        ));
        assert!(matches!(
            evaluate(&ev("Write", "/x/.env.example"), &policy),
            Decision::Allow
        ));
    }

    #[test]
    fn allows_read_of_protected() {
        let policy = Policy::default();
        assert!(matches!(
            evaluate(&ev("Read", "/x/.env"), &policy),
            Decision::Allow
        ));
    }

    #[test]
    fn parses_policy_deny_write_list() {
        let policy = parse_policy(
            "protect_paths:\n  enabled: true\n  deny_write:\n    - .env\n    - private/\n",
        )
        .unwrap();
        assert_eq!(policy.deny_write, vec![".env", "private/"]);
        assert!(matches!(
            evaluate(&ev("Write", "private/key"), &policy),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn disabled_protect_paths_allows_write() {
        let policy =
            parse_policy("protect_paths:\n  enabled: false\n  deny_write:\n    - .env\n").unwrap();
        assert!(matches!(
            evaluate(&ev("Write", "/x/.env"), &policy),
            Decision::Allow
        ));
    }
}

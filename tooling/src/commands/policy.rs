//! `mw policy check` — the cross-harness enforcement engine.
//!
//! Reads a single event as JSON on stdin and returns a decision as JSON on
//! stdout. The canonical event shape follows Claude Code's hook payload so
//! Claude needs zero translation; other harnesses translate in their thin
//! adapter (Pi extension, Codex config subset, Gemini instructions).
//!
//! Decision is one of: allow | deny{reason} | modify{input} | warn{message}.
//! Phase 2 ships the protocol and a conservative default-allow brain; the real
//! rules are read from .agents/policies.yaml in a later phase.

use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::cli::{PolicyAction, PolicyArgs};

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

    let decision = evaluate(&event);
    println!("{}", serde_json::to_string(&decision)?);

    // Exit code mirrors the decision so harnesses without JSON parsing (e.g. a
    // simple Codex/Gemini shell shim) can still react: 0 allow/warn, 1 deny.
    if matches!(decision, Decision::Deny { .. }) {
        std::process::exit(1);
    }
    Ok(())
}

/// Conservative built-in brain. Phase 3 replaces these literals with rules
/// loaded from .agents/policies.yaml (protect paths, enforce worktree,
/// draft-only PR, session warm-up).
fn evaluate(event: &Event) -> Decision {
    // protect paths: deny writes to .env / secrets/ on write-like tools.
    if is_write_tool(&event.tool_name) {
        if let Some(path) = target_path(&event.tool_input) {
            if is_protected(&path) {
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

fn is_protected(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    name == ".env"
        || name.starts_with(".env.") && name != ".env.example"
        || path.contains("secrets/")
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
        assert!(matches!(
            evaluate(&ev("Write", "/x/.env")),
            Decision::Deny { .. }
        ));
        assert!(matches!(
            evaluate(&ev("Edit", "secrets/key")),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn allows_normal_write() {
        assert!(matches!(
            evaluate(&ev("Write", "src/main.rs")),
            Decision::Allow
        ));
        assert!(matches!(
            evaluate(&ev("Write", "/x/.env.example")),
            Decision::Allow
        ));
    }

    #[test]
    fn allows_read_of_protected() {
        assert!(matches!(evaluate(&ev("Read", "/x/.env")), Decision::Allow));
    }
}

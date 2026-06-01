//! `mw policy check` — the cross-harness enforcement engine.
//!
//! Reads a single event as JSON on stdin and returns a decision as JSON on
//! stdout. The canonical event shape follows Claude Code's hook payload so
//! Claude needs zero translation; other harnesses translate in their thin
//! adapter (Pi extension, Codex config subset, Gemini instructions).
//!
//! Decision is one of: allow | deny{reason} | warn{message}.
//! The policy file is `.agents/policies.yaml`; parsing stays intentionally
//! line-based so the CLI does not depend on a YAML runtime.

use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::cli::{PolicyAction, PolicyArgs};
use crate::workspace;

/// Path of the policy file, relative to the workspace root. Both the runtime
/// engine ([`load_policy`]) and the `mw doctor` diagnostic ([`check_policy_file`])
/// resolve the file through this single constant so they never diverge.
pub const POLICY_FILE: &str = ".agents/policies.yaml";

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
    /// Out-of-band user approval, set from the environment by [`check`] — never
    /// deserialized from the payload. Approval must come from a channel the
    /// agent constructing this event cannot forge (see
    /// [`user_approved_out_of_band`]).
    #[serde(skip)]
    user_approved: bool,
}

/// Harness-neutral decision returned on stdout.
#[derive(Debug, Serialize)]
#[serde(tag = "decision", rename_all = "lowercase")]
enum Decision {
    Allow,
    Deny { reason: String },
    Warn { message: String },
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
    let mut event: Event = if raw.trim().is_empty() {
        Event::default()
    } else {
        serde_json::from_str(&raw).unwrap_or_default()
    };
    // The approval signal is read from the environment, NOT the payload: the
    // process that builds the event (the agent) cannot set the parent
    // environment of this `mw policy check` subprocess, so it cannot self-approve.
    event.user_approved = user_approved_out_of_band();

    let decision = decision_for_load(load_policy(), &event);
    println!("{}", serde_json::to_string(&decision)?);

    // Exit code mirrors the decision so harnesses without JSON parsing (e.g. a
    // simple Codex/Gemini shell shim) can still react: 0 allow/warn, 1 deny.
    if matches!(decision, Decision::Deny { .. }) {
        std::process::exit(1);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PolicyEffect {
    Warn,
    Deny,
}

#[derive(Debug, Clone)]
struct Policy {
    protect_paths_enabled: bool,
    deny_write: Vec<String>,
    enforce_worktree_enabled: bool,
    enforce_worktree_action: PolicyEffect,
    draft_only_pr_enabled: bool,
    draft_only_pr_action: PolicyEffect,
    require_explicit_user_approval: bool,
    repos_roots: Vec<String>,
    worktrees_roots: Vec<String>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            protect_paths_enabled: true,
            deny_write: vec![".env".into(), ".env.*".into(), "secrets/".into()],
            // Missing policy files keep the historical safe default: protect
            // obvious secret files, but do not infer workflow/PR gates.
            enforce_worktree_enabled: false,
            enforce_worktree_action: PolicyEffect::Warn,
            draft_only_pr_enabled: false,
            draft_only_pr_action: PolicyEffect::Deny,
            require_explicit_user_approval: true,
            repos_roots: vec!["../repos".into(), "repos".into()],
            worktrees_roots: vec!["../worktrees".into(), "worktrees".into()],
        }
    }
}

/// Outcome of loading the runtime policy.
///
/// A *malformed* policy file is deliberately distinct from a *missing* one. A
/// missing file is a legitimate "apply safe defaults" signal; a malformed file
/// means the author intended protection the engine cannot read, so it fails
/// closed (see [`decision_for_load`]) instead of silently applying weaker
/// defaults.
enum PolicyLoad {
    /// Policy resolved — parsed from a valid file, or safe defaults when no
    /// workspace root or no policy file exists.
    Loaded(Policy),
    /// The policy file is present but could not be loaded — it failed to parse,
    /// or could not be read (e.g. permission denied). Carries the reason.
    Invalid(String),
}

fn load_policy() -> PolicyLoad {
    let Some(root) = workspace::find_root_from_cwd().ok() else {
        return PolicyLoad::Loaded(Policy::default());
    };

    let path = root.join(POLICY_FILE);
    // Only a genuinely *absent* file degrades to safe defaults. A file that
    // exists but cannot be loaded — unparseable, or unreadable (e.g. permission
    // denied) — fails closed (see `decision_for_load`) so a broken security
    // policy can never silently grant more than its author intended. Matching a
    // bare `Err(_)` here would conflate "missing" with "exists but unreadable"
    // and reopen exactly that hole. `mw doctor` surfaces the same conditions
    // ahead of time (see `check_policy_file`).
    let mut policy = match std::fs::read_to_string(&path) {
        Ok(content) => match parse_policy(&content) {
            Ok(policy) => policy,
            Err(err) => return PolicyLoad::Invalid(err),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Policy::default(),
        Err(e) => return PolicyLoad::Invalid(format!("cannot read {POLICY_FILE}: {e}")),
    };

    if let Ok(yaml) = std::fs::read_to_string(root.join(workspace::WORKSPACE_FILE)) {
        if let Some(repos) = workspace::read_scalar(&yaml, "repos") {
            push_path_candidate(&mut policy.repos_roots, &repos);
            push_existing_absolute_candidate(&mut policy.repos_roots, root.join(&repos));
        }
        if let Some(worktrees) = workspace::read_scalar(&yaml, "worktrees") {
            push_path_candidate(&mut policy.worktrees_roots, &worktrees);
            push_existing_absolute_candidate(&mut policy.worktrees_roots, root.join(&worktrees));
        }
    }

    PolicyLoad::Loaded(policy)
}

fn push_path_candidate(candidates: &mut Vec<String>, path: &str) {
    let normalized = normalize_path(path);
    if !normalized.is_empty() && !candidates.iter().any(|p| p == &normalized) {
        candidates.push(normalized);
    }
}

fn push_existing_absolute_candidate(candidates: &mut Vec<String>, path: std::path::PathBuf) {
    if let Ok(abs) = path.canonicalize() {
        push_path_candidate(candidates, &abs.to_string_lossy());
    }
}

fn parse_policy(content: &str) -> Result<Policy, String> {
    let mut policy = Policy::default();
    let mut section = String::new();
    let mut in_deny_write = false;
    let mut deny_write: Vec<String> = Vec::new();

    for (idx, raw) in content.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim_end();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Reject structurally broken lines so a malformed policy file is
        // surfaced (by `mw doctor`) instead of silently degrading. YAML forbids
        // tab indentation, and every meaningful line in this format is one of:
        // a top-level section header, a `key: value` pair, or a `- item` entry.
        if raw.starts_with('\t') {
            return Err(format!(
                "line {}: tab indentation is not valid YAML",
                idx + 1
            ));
        }
        let is_section_header = !raw.starts_with(' ') && trimmed.ends_with(':');
        let is_list_item = trimmed.starts_with('-');
        let is_key_value = trimmed.contains(':');
        if !is_section_header && !is_list_item && !is_key_value {
            return Err(format!(
                "line {}: expected `key: value`, a `- item`, or a `section:` header, found `{trimmed}`",
                idx + 1
            ));
        }

        if is_section_header {
            section = trimmed.trim_end_matches(':').to_string();
            in_deny_write = false;
            continue;
        }

        match section.as_str() {
            "protect_paths" => {
                if let Some(value) = trimmed.strip_prefix("enabled:") {
                    policy.protect_paths_enabled = parse_bool(value, true);
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
            "enforce_worktree" => {
                if let Some(value) = trimmed.strip_prefix("enabled:") {
                    policy.enforce_worktree_enabled = parse_bool(value, true);
                } else if let Some(value) = trimmed.strip_prefix("action:") {
                    policy.enforce_worktree_action = parse_effect(value, PolicyEffect::Warn);
                }
            }
            "draft_only_pr" => {
                if let Some(value) = trimmed.strip_prefix("enabled:") {
                    policy.draft_only_pr_enabled = parse_bool(value, true);
                } else if let Some(value) = trimmed.strip_prefix("action:") {
                    policy.draft_only_pr_action = parse_effect(value, PolicyEffect::Deny);
                } else if let Some(value) = trimmed.strip_prefix("require_explicit_user_approval:")
                {
                    policy.require_explicit_user_approval = parse_bool(value, true);
                }
            }
            _ => {}
        }
    }

    if !deny_write.is_empty() {
        policy.deny_write = deny_write;
    }
    Ok(policy)
}

/// Result of validating the policy file for `mw doctor`.
///
/// Absence is valid — the engine applies safe defaults — so it is a distinct,
/// non-failing state rather than an error.
#[derive(Debug, PartialEq, Eq)]
pub enum PolicyFileStatus {
    /// No policy file at `.agents/policies.yaml`; defaults apply.
    Missing,
    /// The file exists and parses cleanly.
    Ok,
    /// The file is present but could not be loaded — it failed to parse, or
    /// could not be read (e.g. permission denied). Carries the reason.
    Malformed(String),
}

/// Validate the workspace policy file the same way the runtime engine loads it.
///
/// This powers the `mw doctor` diagnostic that surfaces a malformed
/// `.agents/policies.yaml` so the operator can fix it before the runtime engine
/// fails closed on it (see [`decision_for_load`]). It reuses [`POLICY_FILE`] and
/// [`parse_policy`] so the check can never diverge from the real loader.
pub fn check_policy_file(root: &std::path::Path) -> PolicyFileStatus {
    let path = root.join(POLICY_FILE);
    match std::fs::read_to_string(&path) {
        Ok(content) => match parse_policy(&content) {
            Ok(_) => PolicyFileStatus::Ok,
            Err(e) => PolicyFileStatus::Malformed(e),
        },
        // Mirror `load_policy`: only a genuinely absent file is `Missing`. A
        // present-but-unreadable file (e.g. permission denied) is reported so
        // the operator is not told "absent; defaults apply" about a policy that
        // actually exists but is being silently ignored.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => PolicyFileStatus::Missing,
        Err(e) => PolicyFileStatus::Malformed(format!("cannot read policy file: {e}")),
    }
}

fn parse_bool(value: &str, default: bool) -> bool {
    match clean_scalar(value).as_str() {
        "true" | "yes" | "on" => true,
        "false" | "no" | "off" => false,
        _ => default,
    }
}

fn parse_effect(value: &str, default: PolicyEffect) -> PolicyEffect {
    match clean_scalar(value).as_str() {
        "deny" | "block" => PolicyEffect::Deny,
        "warn" => PolicyEffect::Warn,
        _ => default,
    }
}

fn clean_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

/// Map a [`PolicyLoad`] to the decision for the current event.
///
/// A valid (or absent) policy is evaluated normally. A policy file that exists
/// but cannot be loaded (unparseable or unreadable) fails closed: every gated
/// action is denied — with a reason naming the file and the cause — until it is
/// fixed. This is the runtime counterpart to the `mw doctor` diagnostic
/// ([`check_policy_file`]).
fn decision_for_load(load: PolicyLoad, event: &Event) -> Decision {
    match load {
        PolicyLoad::Loaded(policy) => evaluate(event, &policy),
        PolicyLoad::Invalid(err) => Decision::Deny {
            reason: format!(
                "policy file {POLICY_FILE} could not be loaded ({err}); refusing to apply \
                 weaker defaults — fix it (run `mw doctor` to validate) and retry"
            ),
        },
    }
}

fn evaluate(event: &Event, policy: &Policy) -> Decision {
    let _ = &event.hook_event_name;

    if policy.protect_paths_enabled
        && is_write_tool(&event.tool_name)
        && let Some(path) = target_path(&event.tool_input)
        && is_protected(&path, &policy.deny_write)
    {
        return Decision::Deny {
            reason: format!("writing to protected path is not allowed: {path}"),
        };
    }

    if policy.enforce_worktree_enabled
        && is_write_tool(&event.tool_name)
        && let Some(path) = target_path(&event.tool_input)
        && is_clean_checkout_path(&path, policy)
    {
        return decision_for_effect(
            policy.enforce_worktree_action,
            format!("edit appears to target a clean checkout instead of a worktree: {path}"),
        );
    }

    if policy.draft_only_pr_enabled
        && is_pr_publish_event(&event.tool_name, &event.tool_input)
        && policy.require_explicit_user_approval
        && !event.user_approved
    {
        return decision_for_effect(
            policy.draft_only_pr_action,
            "PR comments, approvals, or review submissions require explicit user approval \
             (set MW_USER_APPROVED=1 in the environment to authorize)"
                .into(),
        );
    }

    Decision::Allow
}

fn decision_for_effect(effect: PolicyEffect, message: String) -> Decision {
    match effect {
        PolicyEffect::Warn => Decision::Warn { message },
        PolicyEffect::Deny => Decision::Deny { reason: message },
    }
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
    let normalized = normalize_path(path);
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

fn is_clean_checkout_path(path: &str, policy: &Policy) -> bool {
    is_under_any_root(path, &policy.repos_roots)
        && !is_under_any_root(path, &policy.worktrees_roots)
}

fn is_under_any_root(path: &str, roots: &[String]) -> bool {
    let path = normalize_path(path);
    roots.iter().any(|root| is_under_root(&path, root))
}

fn is_under_root(path: &str, root: &str) -> bool {
    // Exact match or a true path-prefix only. A `path.contains("/{root}/")`
    // arm was removed: it false-matched any path with the root name as an
    // interior segment (e.g. `/a/repos/b` for root `repos`). Absolute paths
    // configured via relative roots are handled by `load_policy`, which also
    // pushes the resolved absolute root as a candidate.
    let root = normalize_path(root).trim_end_matches('/').to_string();
    path == root || path.starts_with(&format!("{root}/"))
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_string()
}

fn is_pr_publish_event(tool_name: &str, input: &serde_json::Value) -> bool {
    let tool = tool_name.to_ascii_lowercase();
    if contains_pr_publish_terms(&tool) {
        return true;
    }

    for key in ["command", "cmd", "args", "input"] {
        if let Some(value) = input.get(key).and_then(|v| v.as_str())
            && is_pr_publish_command(value)
        {
            return true;
        }
    }
    false
}

fn contains_pr_publish_terms(value: &str) -> bool {
    let pr_context = value.contains("pull_request") || value.contains("pull-request");
    let publish_action = value.contains("comment")
        || value.contains("review")
        || value.contains("approve")
        || value.contains("submit")
        || value.contains("post");
    pr_context && publish_action
}

fn is_pr_publish_command(command: &str) -> bool {
    let command = command.to_ascii_lowercase();
    command.contains("gh pr comment")
        || command.contains("gh pr review")
        || (command.contains("gh api")
            && command.contains("/pulls/")
            && (command.contains("/comments") || command.contains("/reviews")))
}

/// True when the user has authorized PR publishing out-of-band, via the
/// `MW_USER_APPROVED` environment variable.
///
/// Approval deliberately comes from the environment rather than the event
/// payload. The agent that constructs a tool call (and thus the `tool_input`
/// JSON) cannot set the environment of the separate `mw policy check`
/// subprocess, so it cannot grant itself approval. The user (or harness, on the
/// user's behalf) exports `MW_USER_APPROVED=1` to authorize. This is a coarse,
/// session-scoped signal — that is an accepted limitation, documented in
/// `.agents/policies.yaml` and `SECURITY.md`.
fn user_approved_out_of_band() -> bool {
    std::env::var("MW_USER_APPROVED")
        .ok()
        .is_some_and(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
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

    fn policy_with_workflow_gates() -> Policy {
        parse_policy(
            "protect_paths:\n  enabled: true\n  deny_write:\n    - .env\n    - secrets/\nenforce_worktree:\n  enabled: true\n  action: warn\ndraft_only_pr:\n  enabled: true\n  action: deny\n  require_explicit_user_approval: true\n",
        )
        .unwrap()
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

    #[test]
    fn parses_workflow_policy_sections() {
        let policy = policy_with_workflow_gates();
        assert!(policy.enforce_worktree_enabled);
        assert_eq!(policy.enforce_worktree_action, PolicyEffect::Warn);
        assert!(policy.draft_only_pr_enabled);
        assert_eq!(policy.draft_only_pr_action, PolicyEffect::Deny);
        assert!(policy.require_explicit_user_approval);
    }

    #[test]
    fn warns_when_editing_clean_checkout() {
        let policy = policy_with_workflow_gates();
        assert!(matches!(
            evaluate(&ev("Edit", "../repos/api/src/lib.rs"), &policy),
            Decision::Warn { .. }
        ));
        assert!(matches!(
            evaluate(&ev("Edit", "../worktrees/api-fix/src/lib.rs"), &policy),
            Decision::Allow
        ));
    }

    #[test]
    fn can_deny_clean_checkout_edits() {
        let policy = parse_policy(
            "enforce_worktree:\n  enabled: true\n  action: deny\ndraft_only_pr:\n  enabled: false\n",
        )
        .unwrap();
        assert!(matches!(
            evaluate(&ev("Write", "../repos/api/src/lib.rs"), &policy),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn denies_pr_publish_without_explicit_user_approval() {
        let policy = policy_with_workflow_gates();
        let event = Event {
            tool_name: "Bash".into(),
            tool_input: serde_json::json!({ "command": "gh pr comment 12 --body 'ready'" }),
            ..Default::default()
        };
        assert!(matches!(evaluate(&event, &policy), Decision::Deny { .. }));
    }

    #[test]
    fn allows_pr_publish_with_out_of_band_approval() {
        let policy = policy_with_workflow_gates();
        // Approval comes from the out-of-band signal (env var, surfaced as
        // `user_approved`), never from an agent-supplied payload field.
        let event = Event {
            tool_name: "Bash".into(),
            tool_input: serde_json::json!({ "command": "gh pr review 12 --approve" }),
            user_approved: true,
            ..Default::default()
        };
        assert!(matches!(evaluate(&event, &policy), Decision::Allow));
    }

    #[test]
    fn payload_field_cannot_self_approve_pr_publish() {
        // Regression guard for the old loophole: setting approval-looking keys
        // in the agent-controlled tool_input must NOT grant approval.
        let policy = policy_with_workflow_gates();
        let event = Event {
            tool_name: "Bash".into(),
            tool_input: serde_json::json!({
                "command": "gh pr review 12 --approve",
                "explicit_user_approval": true,
                "user_approved": true,
                "mw_policy": { "user_approved": true }
            }),
            // user_approved stays false: no out-of-band approval was given.
            ..Default::default()
        };
        assert!(matches!(evaluate(&event, &policy), Decision::Deny { .. }));
    }

    #[test]
    fn approval_not_required_allows_pr_publish_without_approval() {
        // With `require_explicit_user_approval: false` the gate is opt-out:
        // the draft-only feature is enabled but approval is not demanded, so a
        // PR-publish event is allowed even though no out-of-band approval was
        // given. (Regression guard: the flag must loosen the gate, not deny
        // unconditionally.)
        let policy = parse_policy(
            "draft_only_pr:\n  enabled: true\n  action: deny\n  require_explicit_user_approval: false\n",
        )
        .unwrap();
        assert!(!policy.require_explicit_user_approval);
        let event = Event {
            tool_name: "Bash".into(),
            tool_input: serde_json::json!({ "command": "gh pr comment 12 --body 'ready'" }),
            // No approval signal: still allowed because approval is not required.
            ..Default::default()
        };
        assert!(matches!(evaluate(&event, &policy), Decision::Allow));
    }

    #[test]
    fn parse_policy_rejects_malformed_lines() {
        // A stray scalar with no `:` and no `-` is not a valid line in this
        // format and must be reported, not silently ignored.
        assert!(parse_policy("protect_paths:\n  this is broken yaml\n").is_err());
        // Tab indentation is invalid YAML.
        assert!(parse_policy("protect_paths:\n\tenabled: true\n").is_err());
    }

    #[test]
    fn parse_policy_accepts_the_template_shape() {
        let ok = "version: 1\nprotect_paths:\n  enabled: true\n  deny_write:\n    - .env\nenforce_worktree:\n  enabled: true\n  action: warn\n";
        assert!(parse_policy(ok).is_ok());
    }

    #[test]
    fn malformed_policy_file_fails_closed() {
        // A present-but-unparseable policy must DENY every gated action rather
        // than silently degrade to weaker defaults.
        let denied = decision_for_load(
            PolicyLoad::Invalid("line 2: tab indentation is not valid YAML".into()),
            &ev("Write", "src/main.rs"),
        );
        assert!(
            matches!(denied, Decision::Deny { .. }),
            "malformed policy must fail closed, got {denied:?}"
        );
        // A valid load still evaluates normally: an innocuous write is allowed.
        let allowed = decision_for_load(
            PolicyLoad::Loaded(Policy::default()),
            &ev("Write", "src/main.rs"),
        );
        assert!(matches!(allowed, Decision::Allow));
    }

    // A file that exists but is unreadable (permission denied) must NOT be
    // mistaken for an absent file: that would silently drop the gates the
    // author enabled. It must surface as `Malformed` so the engine fails closed.
    #[cfg(unix)]
    #[test]
    fn check_policy_file_treats_unreadable_as_malformed_not_missing() {
        use std::os::unix::fs::PermissionsExt;
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "mw-policy-perm-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join(".agents")).unwrap();
        let path = root.join(POLICY_FILE);
        std::fs::write(&path, "protect_paths:\n  enabled: true\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

        // If the file is still readable, permissions are not enforced (e.g. the
        // test runs as root) and this scenario cannot be exercised — skip.
        let unreadable = std::fs::read_to_string(&path).is_err();
        let status = check_policy_file(&root);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).ok();
        std::fs::remove_dir_all(&root).ok();

        if unreadable {
            assert!(
                matches!(status, PolicyFileStatus::Malformed(_)),
                "unreadable policy must not read as Missing, got {status:?}"
            );
        }
    }

    #[test]
    fn check_policy_file_reports_missing_ok_and_malformed() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "mw-policy-check-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join(".agents")).unwrap();

        // Absent file is a valid, non-failing state.
        assert_eq!(check_policy_file(&root), PolicyFileStatus::Missing);

        // A valid file parses.
        std::fs::write(
            root.join(POLICY_FILE),
            "protect_paths:\n  enabled: true\n  deny_write:\n    - .env\n",
        )
        .unwrap();
        assert_eq!(check_policy_file(&root), PolicyFileStatus::Ok);

        // A malformed file is detected and carries an explanatory error.
        std::fs::write(root.join(POLICY_FILE), "protect_paths:\n  not valid yaml\n").unwrap();
        assert!(matches!(
            check_policy_file(&root),
            PolicyFileStatus::Malformed(_)
        ));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn denies_pr_review_mcp_tool_without_approval() {
        let policy = policy_with_workflow_gates();
        let event = Event {
            tool_name: "mcp__github__add_pull_request_review_comment".into(),
            tool_input: serde_json::json!({ "body": "nit" }),
            ..Default::default()
        };
        assert!(matches!(evaluate(&event, &policy), Decision::Deny { .. }));
    }
}

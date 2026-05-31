//! `mw hook session-start` — optional, always non-blocking session warm-up.
//!
//! Reads the memory profile from workspace.yaml and warms up only if needed.
//! It must never fail the session: any error degrades to exit 0. Replaces
//! scripts/session-start.sh once at parity.

use crate::cli::{HookArgs, HookEvent};
use crate::commands::which;
use crate::workspace;

pub fn run(args: HookArgs) -> anyhow::Result<()> {
    match args.event {
        HookEvent::SessionStart => session_start(),
    }
}

fn session_start() -> anyhow::Result<()> {
    // Best-effort only; never block the session.
    let root = workspace::find_root_from_cwd().ok();
    let profile = root
        .as_ref()
        .and_then(|r| std::fs::read_to_string(r.join(workspace::WORKSPACE_FILE)).ok())
        .and_then(|y| workspace::read_scalar(&y, "profile"))
        .unwrap_or_else(|| "none".to_string());

    match profile.as_str() {
        "none" => { /* common case: nothing to do */ }
        "mempalace" | "full" => mempalace_warmup(root.as_deref()),
        "prism" => {
            // The harness starts the MCP server; no warm-up needed.
            println!("session-start: prism profile (no warm-up needed)");
        }
        other => {
            eprintln!("session-start: unknown profile `{other}`, treating as none");
        }
    }

    // Always succeed.
    Ok(())
}

/// Non-blocking mempalace warm-up: resolve the wing from `.env.local`
/// (`MEMPALACE_WING`) or the workspace directory name, then run
/// `mempalace status` and `mempalace wake-up`. Any failure is tolerated.
fn mempalace_warmup(root: Option<&std::path::Path>) {
    if which("mempalace").is_none() {
        eprintln!("session-start: mempalace CLI not found; skipping warm-up");
        return;
    }
    let wing = root.and_then(env_wing).or_else(|| {
        root.and_then(|r| r.file_name())
            .map(|n| n.to_string_lossy().to_string())
    });
    let wing = wing.unwrap_or_else(|| "default".to_string());

    let status_ok = run_quiet("mempalace", &["status"]);
    let wake_ok = run_quiet("mempalace", &["wake-up", "--wing", &wing]);
    if status_ok && wake_ok {
        eprintln!("session-start: mempalace ready (wing={wing})");
    } else {
        eprintln!("session-start: mempalace warm-up degraded; continuing");
    }
}

/// Read `MEMPALACE_WING=` from `.env.local`, if present.
fn env_wing(root: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(root.join(".env.local")).ok()?;
    for line in content.lines() {
        if let Some(v) = line.trim().strip_prefix("MEMPALACE_WING=") {
            let v = v.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn run_quiet(bin: &str, args: &[&str]) -> bool {
    std::process::Command::new(bin)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

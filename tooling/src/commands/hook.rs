//! `mw hook session-start` — optional, always non-blocking session warm-up.
//!
//! Reads the memory profile from workspace.yaml and warms up only if needed.
//! It must never fail the session: any error degrades to exit 0. Replaces
//! scripts/session-start.sh once at parity.

use crate::cli::{HookArgs, HookEvent};
use crate::workspace;

pub fn run(args: HookArgs) -> anyhow::Result<()> {
    match args.event {
        HookEvent::SessionStart => session_start(),
    }
}

fn session_start() -> anyhow::Result<()> {
    // Best-effort only; never block the session.
    let profile = (|| {
        let root = workspace::find_root_from_cwd().ok()?;
        let yaml = std::fs::read_to_string(root.join(workspace::WORKSPACE_FILE)).ok()?;
        workspace::read_scalar(&yaml, "profile")
    })()
    .unwrap_or_else(|| "none".to_string());

    match profile.as_str() {
        "none" => { /* common case: nothing to do */ }
        "mempalace" | "full" => {
            // TODO(phase 3): best-effort mempalace warm-up.
            println!("session-start: mempalace warm-up (stub)");
        }
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

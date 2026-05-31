//! `mw links` — create or repair agent compatibility symlinks.
//!
//! Reconciles AGENTS.md / CLAUDE.md / GEMINI.md -> .agents/AGENTS.md. Harness
//! enforcement adapters (Claude hooks, Pi extension, Codex config, Gemini
//! instructions) are generated separately once .agents/policies.yaml exists.

use crate::cli::LinksArgs;
use crate::links::{ensure_link, LinkStatus, AGENT_LINKS};
use crate::workspace;

pub fn run(args: LinksArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    println!("links: workspace root = {}", root.display());

    let mut conflicts = 0u32;
    for spec in AGENT_LINKS {
        let status = ensure_link(&root, spec, args.force, args.common.dry_run)?;
        let label = match status {
            LinkStatus::Ok => "ok",
            LinkStatus::Created => "created",
            LinkStatus::Repaired => "repaired",
            LinkStatus::WouldCreate => "would-create",
            LinkStatus::WouldRepair => "would-repair",
            LinkStatus::Conflict => {
                conflicts += 1;
                "CONFLICT (exists, not a correct link; use --force)"
            }
        };
        println!("  {label:<12} {} -> {}", spec.name, spec.target);
    }

    if conflicts > 0 {
        anyhow::bail!("{conflicts} link conflict(s); re-run with --force to replace");
    }
    // TODO(phase 3b): generate harness enforcement adapters that shell out to
    // `mw policy check`, driven by .agents/policies.yaml.
    Ok(())
}

//! `mw links` — create or repair agent compatibility symlinks and adapters.

use crate::cli::LinksArgs;
use crate::workspace;

pub fn run(args: LinksArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    println!("links: workspace root = {}", root.display());
    if args.force {
        println!("links: --force (existing links/adapters will be replaced)");
    }
    if args.common.dry_run {
        println!("[dry-run] no changes will be written");
    }

    // TODO(phase 3): create/repair canonical symlinks
    //   AGENTS.md, CLAUDE.md, GEMINI.md -> .agents/AGENTS.md
    // and generate the harness enforcement adapters that call `mw policy check`:
    //   .claude/settings.json hooks, .pi/extensions/mw-policy.ts,
    //   .codex/config.toml subset, Gemini instructions.
    // Supersedes scripts/install-agent-links.sh + scripts/check-symlinks.sh.
    println!("links: not yet implemented (interim: scripts/install-agent-links.sh)");
    Ok(())
}

//! `mw links` — create or repair agent compatibility symlinks.
//!
//! Reconciles all workspace compatibility links, including top-level agent
//! instruction files and harness skill/command/agent directories.

use crate::cli::LinksArgs;
use crate::links::{ensure_link, LinkStatus, COMPAT_LINKS};
use crate::workspace;

pub fn run(args: LinksArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    println!("links: workspace root = {}", root.display());

    let mut conflicts = 0u32;
    for spec in COMPAT_LINKS {
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
    Ok(())
}

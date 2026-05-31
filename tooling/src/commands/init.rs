//! `mw init` — materialize or repair a workspace from the content template.

use crate::cli::InitArgs;

pub fn run(args: InitArgs) -> anyhow::Result<()> {
    let target = args.path.clone().unwrap_or_else(|| ".".to_string());

    if args.common.dry_run {
        println!("[dry-run] would initialize meta-workspace at {target}");
    } else {
        println!("init: target = {target}");
    }
    if let Some(id) = &args.company_id {
        println!("init: company_id = {id}");
    }
    if let Some(name) = &args.company_name {
        println!("init: company_name = {name}");
    }

    // TODO(phase 3): materialize the content template, stamp company_* into
    // workspace.yaml via token substitution, create parent-sibling dirs
    // (../repos, ../worktrees, ../scratch, ../archives, ../logs), then call
    // links to generate symlinks and harness adapters. Supersedes
    // scripts/bootstrap.sh + scripts/install-agent-links.sh.
    println!("init: not yet implemented (interim: scripts/bootstrap.sh)");
    Ok(())
}

//! `mw init` — materialize or repair a workspace from the embedded template.
//!
//! Writes the embedded `template/` tree (skipping compat-link paths), stamps
//! company fields into `workspace.yaml`, and recreates the agent/harness
//! symlinks. Existing files are preserved, so `init` doubles as a repair.

use std::path::PathBuf;

use crate::cli::InitArgs;
use crate::links::{ensure_link, LinkStatus, COMPAT_LINKS};
use crate::{embed, workspace};

pub fn run(args: InitArgs) -> anyhow::Result<()> {
    let root = PathBuf::from(args.path.clone().unwrap_or_else(|| ".".to_string()));
    if !args.common.dry_run {
        std::fs::create_dir_all(&root)?;
    }
    println!("init: target = {}", root.display());

    // 1. Materialize files, skipping the compat-link paths (recreated below).
    let files = embed::files();
    let skip: Vec<&str> = COMPAT_LINKS.iter().map(|l| l.name).collect();
    let summary = embed::materialize(&files, &root, &skip, args.common.dry_run)?;
    println!(
        "init: {} file(s) created, {} preserved",
        summary.created.len(),
        summary.skipped.len()
    );

    // 2. Stamp company fields into workspace.yaml (if provided and present).
    if !args.common.dry_run && (args.company_id.is_some() || args.company_name.is_some()) {
        stamp_company(&root, &args)?;
    }

    // 3. Recreate compatibility symlinks.
    for spec in COMPAT_LINKS {
        let status = ensure_link(&root, spec, false, args.common.dry_run)?;
        if matches!(status, LinkStatus::Conflict) {
            println!("  link conflict (kept): {} (exists, not a link)", spec.name);
        }
    }

    if args.common.dry_run {
        println!("[dry-run] no changes were written");
    } else {
        println!("init: workspace ready at {}", root.display());
    }
    Ok(())
}

fn stamp_company(root: &std::path::Path, args: &InitArgs) -> anyhow::Result<()> {
    let wf = root.join(workspace::WORKSPACE_FILE);
    let mut yaml = std::fs::read_to_string(&wf)?;
    if let Some(id) = &args.company_id {
        yaml = workspace::set_scalar(&yaml, "company_id", id)?;
    }
    if let Some(name) = &args.company_name {
        yaml = workspace::set_scalar(&yaml, "company_name", name)?;
    }
    std::fs::write(&wf, yaml)?;
    Ok(())
}

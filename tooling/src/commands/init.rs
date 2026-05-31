//! `mw init` — materialize or repair a workspace from the embedded template.
//!
//! Writes the embedded `template/` tree (skipping compat-link paths), stamps
//! company fields into `workspace.yaml`, and recreates the agent/harness
//! symlinks. Existing files are preserved, so `init` doubles as a repair.

use std::path::PathBuf;

use crate::cli::InitArgs;
use crate::links::{ensure_link, LinkStatus, COMPAT_LINKS};
use crate::{embed, scaffold, workspace};

/// Parent-sibling working directories created next to a workspace.
const PARENT_DIRS: &[&str] = &[
    "../repos",
    "../worktrees",
    "../scratch",
    "../archives",
    "../logs",
];

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

    // 2. Stamp company fields and write .env.local (non-dry-run only).
    if !args.common.dry_run {
        stamp_workspace(&root, &args)?;
        ensure_env_local(&root, &args)?;
        create_parent_dirs(&root)?;
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

/// Resolve the workspace slug: explicit company id, else the existing
/// `workspace.name`, else the target directory name.
fn slug(root: &std::path::Path, args: &InitArgs, yaml: &str) -> String {
    if let Some(id) = &args.company_id {
        return id.clone();
    }
    if let Some(name) = workspace::read_scalar(yaml, "name") {
        return name;
    }
    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "meta-workspace".to_string())
}

fn stamp_workspace(root: &std::path::Path, args: &InitArgs) -> anyhow::Result<()> {
    let wf = root.join(workspace::WORKSPACE_FILE);
    let mut yaml = std::fs::read_to_string(&wf)?;
    let s = slug(root, args, &yaml);

    if let Some(id) = &args.company_id {
        yaml = workspace::set_scalar(&yaml, "company_id", id)?;
        yaml = workspace::set_scalar(&yaml, "name", &s)?;
    }
    if let Some(name) = &args.company_name {
        yaml = workspace::set_scalar(&yaml, "company_name", name)?;
    }
    std::fs::write(&wf, yaml)?;

    // company/profile.md Name/Slug (only when something to stamp).
    if args.company_id.is_some() || args.company_name.is_some() {
        let profile_path = root.join("company/profile.md");
        if let Ok(md) = std::fs::read_to_string(&profile_path) {
            let display = args.company_name.clone().unwrap_or_else(|| s.clone());
            std::fs::write(&profile_path, scaffold::stamp_profile(&md, &display, &s))?;
        }
    }
    Ok(())
}

fn ensure_env_local(root: &std::path::Path, args: &InitArgs) -> anyhow::Result<()> {
    let env_path = root.join(".env.local");
    if env_path.exists() {
        return Ok(());
    }
    let yaml = std::fs::read_to_string(root.join(workspace::WORKSPACE_FILE)).unwrap_or_default();
    let s = slug(root, args, &yaml);
    std::fs::write(&env_path, scaffold::render_env_local(&s, "none"))?;
    println!("init: created .env.local (slug = {s})");
    Ok(())
}

fn create_parent_dirs(root: &std::path::Path) -> anyhow::Result<()> {
    for dir in PARENT_DIRS {
        std::fs::create_dir_all(root.join(dir))?;
    }
    Ok(())
}

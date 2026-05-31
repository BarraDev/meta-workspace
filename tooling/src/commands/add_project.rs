//! `mw add-project` — append an entry to projects/registry.yaml.

use crate::cli::AddProjectArgs;
use crate::registry::{self, NewProject};
use crate::workspace;

pub fn run(args: AddProjectArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    let registry_path = root.join("projects/registry.yaml");

    let id = match &args.id {
        Some(id) => id.clone(),
        None => anyhow::bail!("--id is required (non-interactive add-project)"),
    };
    let name = args.name.clone().unwrap_or_else(|| id.clone());

    // Default working-dir roots come from workspace.yaml paths (parent siblings).
    let ws_yaml = std::fs::read_to_string(root.join(workspace::WORKSPACE_FILE))?;
    let repos_dir = workspace::read_scalar(&ws_yaml, "repos").unwrap_or_else(|| "../repos".into());
    let worktrees_dir =
        workspace::read_scalar(&ws_yaml, "worktrees").unwrap_or_else(|| "../worktrees".into());

    let project = NewProject {
        id,
        name,
        repo_url: args.repo_url.clone(),
        default_branch: args.default_branch.clone(),
        repos_dir,
        worktrees_dir,
    };

    let current = std::fs::read_to_string(&registry_path)?;
    let updated = registry::append(&current, &project)?;

    println!("add-project: registry = {}", registry_path.display());
    println!("  id             = {}", project.id);
    println!("  name           = {}", project.name);
    if let Some(url) = &project.repo_url {
        println!("  repo_url       = {url}");
    }
    println!("  default_branch = {}", project.default_branch);

    if args.common.dry_run {
        println!("[dry-run] no changes will be written");
        return Ok(());
    }
    std::fs::write(&registry_path, updated)?;
    println!("add-project: added `{}`", project.id);
    Ok(())
}

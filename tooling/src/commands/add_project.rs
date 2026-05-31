//! `mw add-project` — append an entry to projects/registry.yaml.

use crate::cli::AddProjectArgs;
use crate::workspace;

pub fn run(args: AddProjectArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    let registry = root.join("projects/registry.yaml");

    let id = match &args.id {
        Some(id) => id.clone(),
        None => anyhow::bail!("--id is required (non-interactive add-project)"),
    };

    println!("add-project: registry = {}", registry.display());
    println!("  id             = {id}");
    if let Some(name) = &args.name {
        println!("  name           = {name}");
    }
    if let Some(url) = &args.repo_url {
        println!("  repo_url       = {url}");
    }
    println!("  default_branch = {}", args.default_branch);

    if args.common.dry_run {
        println!("[dry-run] no changes will be written");
    }

    // TODO(phase 3): append-with-duplicate-check into projects/registry.yaml
    // (id uniqueness enforced), preserving the file's existing list shape.
    // Supersedes scripts/new-project.sh.
    println!("add-project: not yet implemented (interim: scripts/new-project.sh)");
    Ok(())
}

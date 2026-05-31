//! `mw migrate` — upgrade an older workspace to the current schemaVersion.

use crate::cli::{MigrateArgs, SCHEMA_VERSION};
use crate::workspace;

pub fn run(args: MigrateArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    let wf = root.join(workspace::WORKSPACE_FILE);
    let yaml = std::fs::read_to_string(&wf)?;

    let target = args.to.unwrap_or(SCHEMA_VERSION);
    if target > SCHEMA_VERSION {
        anyhow::bail!("this binary supports up to schemaVersion {SCHEMA_VERSION}, not {target}");
    }

    let current: u32 = workspace::read_scalar(&yaml, "schemaVersion")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("workspace.yaml has no valid schemaVersion"))?;

    if current == target {
        println!("migrate: already at schemaVersion {target}");
        return Ok(());
    }
    if current > target {
        anyhow::bail!("workspace is at schemaVersion {current}, cannot downgrade to {target}");
    }

    println!("migrate: {current} -> {target}");
    // TODO(phase 3): apply ordered, idempotent step migrations (1->2, 2->3, ...)
    // as new schema versions are introduced. With only version 1 today there is
    // nothing to apply.
    if args.common.dry_run {
        println!("[dry-run] no changes will be written");
    }
    println!("migrate: no migration steps registered for current versions");
    Ok(())
}

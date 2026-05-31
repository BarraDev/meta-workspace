//! `mw memory` — read or set the workspace memory profile.
//!
//! Setting the profile is a single anchored line edit on `workspace.yaml`
//! (`profile: <value>`), preserving indentation and the inline comment.
//! mempalace and prism remain optional; `none` is the common default.

use crate::cli::{MemoryArgs, MemoryProfile};
use crate::workspace;

fn profile_str(p: MemoryProfile) -> &'static str {
    match p {
        MemoryProfile::None => "none",
        MemoryProfile::Mempalace => "mempalace",
        MemoryProfile::Prism => "prism",
        MemoryProfile::Full => "full",
    }
}

pub fn run(args: MemoryArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    let wf = root.join(workspace::WORKSPACE_FILE);
    let yaml = std::fs::read_to_string(&wf)?;

    let Some(profile) = args.profile else {
        let current = workspace::read_scalar(&yaml, "profile").unwrap_or_else(|| "none".into());
        println!("memory: profile = {current}");
        return Ok(());
    };

    let value = profile_str(profile);
    let current = workspace::read_scalar(&yaml, "profile");
    if current.as_deref() == Some(value) {
        println!("memory: profile already = {value}");
        return Ok(());
    }

    let updated = workspace::set_scalar(&yaml, "profile", value)?;
    if args.common.dry_run {
        println!("[dry-run] memory: would set profile = {value}");
        return Ok(());
    }
    std::fs::write(&wf, updated)?;
    println!("memory: profile = {value}");

    // Note: enabling mempalace/prism only flips the profile here. Installing the
    // optional runtimes (python/mempalace CLI, node MCP) stays a separate,
    // explicit step (interim: scripts/install-memory.sh).
    Ok(())
}

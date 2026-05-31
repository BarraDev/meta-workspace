//! `mw memory` — read or set the workspace memory profile.
//!
//! Setting the profile is a single anchored line edit on `workspace.yaml`
//! (`profile: <value>`), preserving indentation and the inline comment.
//! mempalace and prism remain optional; `none` is the common default.

use crate::cli::{MemoryArgs, MemoryProfile};
use crate::{scaffold, workspace};

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
    if current.as_deref() == Some(value) && args.slug.is_none() {
        println!("memory: profile already = {value}");
        return Ok(());
    }

    let updated = workspace::set_scalar(&yaml, "profile", value)?;
    if args.common.dry_run {
        println!("[dry-run] memory: would set profile = {value}");
        if let Some(slug) = &args.slug {
            println!("[dry-run] memory: would set .env.local slug = {slug}");
        }
        return Ok(());
    }
    std::fs::write(&wf, updated)?;
    println!("memory: profile = {value}");

    // Mirror the profile (and optional slug) into .env.local, replacing the
    // interim scripts/install-memory.sh behavior.
    update_env_local(&root, value, args.slug.as_deref())?;

    // Note: enabling mempalace/prism only flips the profile here. Installing the
    // optional runtimes (python/mempalace CLI, node MCP) stays a separate,
    // explicit step.
    Ok(())
}

fn update_env_local(
    root: &std::path::Path,
    profile: &str,
    slug: Option<&str>,
) -> anyhow::Result<()> {
    let env_path = root.join(".env.local");
    let mut content = std::fs::read_to_string(&env_path).unwrap_or_default();
    content = scaffold::upsert_env(&content, "MEMORY_PROFILE", profile);
    if let Some(slug) = slug {
        content = scaffold::upsert_env(&content, "MEMPALACE_WING", slug);
        content = scaffold::upsert_env(&content, "PRISM_PROJECT", slug);
    }
    std::fs::write(&env_path, content)?;
    println!("memory: updated .env.local");
    Ok(())
}

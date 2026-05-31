//! `mw sdd` — controlled cc-sdd install/update (staged by default).

use crate::cli::{SddAction, SddArgs, SddInstallArgs, SddMemoryPolicy, SddMode};
use crate::workspace;

pub fn run(args: SddArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    match args.action {
        SddAction::Install(a) => stage("install", &root, a),
        SddAction::Update(a) => stage("update", &root, a),
        SddAction::Status => status(&root),
    }
}

fn mode_str(m: SddMode) -> &'static str {
    match m {
        SddMode::Staged => "staged",
        SddMode::Direct => "direct",
    }
}

fn policy_str(p: SddMemoryPolicy) -> &'static str {
    match p {
        SddMemoryPolicy::Vendor => "vendor",
        SddMemoryPolicy::Replace => "replace",
    }
}

fn stage(op: &str, root: &std::path::Path, a: SddInstallArgs) -> anyhow::Result<()> {
    println!("sdd {op}: root = {}", root.display());
    println!("  mode          = {}", mode_str(a.mode));
    println!("  memory_policy = {}", policy_str(a.memory_policy));

    if matches!(a.mode, SddMode::Direct) && matches!(a.memory_policy, SddMemoryPolicy::Vendor) {
        // Guard the live CLAUDE.md -> .agents/AGENTS.md symlink.
        println!("  note: direct mode preserves the CLAUDE.md symlink (vendor policy)");
    }

    // TODO(phase 3): run cc-sdd staged in a temp dir by default; write the
    // generated memory doc to .agents/vendor/cc-sdd/CLAUDE.md; only touch the
    // live symlink under --memory-policy=replace; update .sdd/manifest.json
    // (serde_json) and the sdd block in workspace.yaml.
    // Supersedes scripts/install-sdd.sh + scripts/update-sdd.sh.
    println!("sdd {op}: not yet implemented (interim: scripts/install-sdd.sh)");
    Ok(())
}

fn status(root: &std::path::Path) -> anyhow::Result<()> {
    let manifest = root.join(".sdd/manifest.json");
    if !manifest.is_file() {
        println!("sdd status: no .sdd/manifest.json found");
        return Ok(());
    }
    let raw = std::fs::read_to_string(&manifest)?;
    let json: serde_json::Value = serde_json::from_str(&raw)?;
    let enabled = json
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let version = json
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("null");
    let mode = json
        .get("install_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("staged");
    println!("sdd status: enabled={enabled} version={version} install_mode={mode}");
    Ok(())
}

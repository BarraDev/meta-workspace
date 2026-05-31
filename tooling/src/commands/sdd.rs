//! `mw sdd` — controlled cc-sdd install/update (staged by default).
//!
//! Mirrors template/scripts/install-sdd.sh: validate the mode/policy pairing,
//! run cc-sdd's dry run, then apply (staged in a temp dir by default), vendor the
//! generated docs, and record state in `.sdd/manifest.json` (serde_json) plus the
//! `sdd:` block of `workspace.yaml`. Requires node/`npx` only when actually
//! applying; a missing runtime degrades to a clear error.

use std::path::Path;
use std::process::Command;

use crate::cli::{SddAction, SddArgs, SddInstallArgs, SddMemoryPolicy, SddMode, SddTargets};
use crate::sdd::{self, Target};
use crate::workspace;

pub fn run(args: SddArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    match args.action {
        SddAction::Install(a) => apply(&root, "install", a),
        SddAction::Update(a) => apply(&root, "update", a),
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

fn target(t: SddTargets) -> Target {
    match t {
        SddTargets::Claude => Target::Claude,
        SddTargets::Codex => Target::Codex,
        SddTargets::Gemini => Target::Gemini,
        SddTargets::All => Target::All,
    }
}

fn apply(root: &Path, op: &str, a: SddInstallArgs) -> anyhow::Result<()> {
    let mode = mode_str(a.mode);
    let policy = policy_str(a.memory_policy);
    let tgt = target(a.targets);

    // Validate before doing anything external (fails fast, no runtime needed).
    sdd::validate(mode, policy)?;

    println!("sdd {op}: root = {}", root.display());
    println!("  targets       = {}", tgt.label());
    println!("  mode          = {mode}");
    println!("  memory_policy = {policy}");

    // mw-level dry run: describe and stop, spawn nothing.
    if a.common.dry_run {
        println!(
            "[dry-run] would run cc-sdd ({}) and update manifest + workspace.yaml",
            tgt.label()
        );
        return Ok(());
    }

    let npx = which("npx").ok_or_else(|| {
        anyhow::anyhow!("npx is required to run cc-sdd; install Node.js/npm first")
    })?;

    // cc-sdd dry run (always).
    let mut dry = vec![
        "cc-sdd@latest",
        "--dry-run",
        "--kiro-dir",
        ".kiro",
        "--backup",
        "--overwrite",
        "skip",
    ];
    dry.extend(tgt.skill_flags());
    run_cmd(&npx, &dry, root)?;

    if a.dry_run_only {
        println!("sdd {op}: dry run complete; no files changed");
        return Ok(());
    }

    let vendor = root.join(".agents/vendor/cc-sdd");
    std::fs::create_dir_all(&vendor)?;
    std::fs::create_dir_all(root.join(".sdd"))?;
    std::fs::create_dir_all(root.join(".kiro"))?;

    let mut apply_args = vec![
        "cc-sdd@latest",
        "--kiro-dir",
        ".kiro",
        "--backup",
        "--overwrite",
        "force",
        "--yes",
    ];
    apply_args.extend(tgt.skill_flags());

    if matches!(a.mode, SddMode::Staged) {
        let stage = tempdir()?;
        println!(
            "sdd {op}: applying cc-sdd in staging dir {}",
            stage.display()
        );
        run_cmd(&npx, &apply_args, &stage)?;

        for sub in ["claude", "codex", "gemini"] {
            let _ = std::fs::remove_dir_all(vendor.join(sub));
        }
        copy_if_exists(&stage.join(".claude/skills"), &vendor.join("claude/skills"))?;
        copy_if_exists(&stage.join(".codex/skills"), &vendor.join("codex/skills"))?;
        copy_if_exists(&stage.join(".gemini/skills"), &vendor.join("gemini/skills"))?;
        copy_if_exists(&stage.join(".kiro/settings"), &root.join(".kiro/settings"))?;

        let staged_memory = stage.join("CLAUDE.md");
        if staged_memory.is_file() {
            std::fs::copy(&staged_memory, vendor.join("CLAUDE.md"))?;
            println!("sdd {op}: stored generated memory doc -> .agents/vendor/cc-sdd/CLAUDE.md");
        }
        let _ = std::fs::remove_dir_all(&stage);
    } else {
        // direct mode (validated to require replace): cc-sdd writes in place.
        run_cmd(&npx, &apply_args, root)?;
    }

    // Record state: manifest (JSON) + workspace.yaml sdd block (line-based).
    let manifest = sdd::build_manifest(tgt, mode, policy, &sdd::now_rfc3339());
    let json = serde_json::to_string_pretty(&manifest)? + "\n";
    std::fs::write(root.join(".sdd/manifest.json"), json)?;

    let wf = root.join(workspace::WORKSPACE_FILE);
    let yaml = std::fs::read_to_string(&wf)?;
    std::fs::write(&wf, sdd::update_workspace_sdd(&yaml, tgt, mode, policy)?)?;

    println!("sdd {op}: done ({} mode)", mode);
    Ok(())
}

fn status(root: &Path) -> anyhow::Result<()> {
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

// --- helpers ---------------------------------------------------------------

fn which(bin: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn run_cmd(bin: &Path, args: &[&str], cwd: &Path) -> anyhow::Result<()> {
    let status = Command::new(bin).args(args).current_dir(cwd).status()?;
    if !status.success() {
        anyhow::bail!("command failed: {} {:?}", bin.display(), args);
    }
    Ok(())
}

fn tempdir() -> anyhow::Result<std::path::PathBuf> {
    let p = std::env::temp_dir().join(format!(
        "mw-sdd-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&p)?;
    Ok(p)
}

/// Recursively copy `src` into `dest` if `src` exists.
fn copy_if_exists(src: &Path, dest: &Path) -> anyhow::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    copy_dir(src, dest)?;
    println!("  copied {} -> {}", src.display(), dest.display());
    Ok(())
}

fn copy_dir(src: &Path, dest: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

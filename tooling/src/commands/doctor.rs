//! `mw doctor` — validate a workspace against the contract.
//!
//! This command does real, read-only checks now (it does not depend on the
//! interim scripts). It reports errors and warnings and sets the exit code
//! accordingly. Errors always fail; warnings fail only under `--strict`.

use std::path::Path;

use crate::cli::{DoctorArgs, SCHEMA_VERSION};
use crate::commands::policy::{self, POLICY_FILE, PolicyFileStatus};
use crate::links::COMPAT_LINKS;
use crate::workspace;

pub fn run(args: DoctorArgs) -> anyhow::Result<()> {
    let root = workspace::find_root_from_cwd()?;
    println!("doctor: workspace root = {}", root.display());

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // 1. workspace.yaml + schemaVersion
    let wf = root.join(workspace::WORKSPACE_FILE);
    let yaml = std::fs::read_to_string(&wf)?;
    match workspace::read_scalar(&yaml, "schemaVersion") {
        Some(v) => match v.parse::<u32>() {
            Ok(found) if found == SCHEMA_VERSION => {
                println!("  ok   schemaVersion = {found}");
            }
            Ok(found) if found < SCHEMA_VERSION => warnings.push(format!(
                "schemaVersion {found} is older than {SCHEMA_VERSION}; run `mw migrate`"
            )),
            Ok(found) => errors.push(format!(
                "schemaVersion {found} is newer than this binary supports ({SCHEMA_VERSION})"
            )),
            Err(_) => errors.push(format!("schemaVersion `{v}` is not a number")),
        },
        None => errors.push("workspace.yaml is missing schemaVersion".into()),
    }

    // 2. canonical agent support
    require_dir(&root, ".agents", &mut errors);
    require_file(&root, ".agents/AGENTS.md", &mut errors);

    // 3. symlink compatibility layer (all compat links)
    for spec in COMPAT_LINKS {
        check_symlink(&root, spec.name, spec.target, &mut warnings);
    }

    // 4. structured files mw owns
    require_file(&root, "projects/registry.yaml", &mut warnings);
    require_file(&root, "company/profile.md", &mut warnings);

    // 5. policy file parses (a malformed file makes the engine fail closed and
    //    deny gated actions, so surface it here to be fixed first).
    match policy::check_policy_file(&root) {
        PolicyFileStatus::Missing => println!("  ok   {POLICY_FILE} (absent; defaults apply)"),
        PolicyFileStatus::Ok => println!("  ok   {POLICY_FILE} parses"),
        PolicyFileStatus::Malformed(err) => errors.push(format!(
            "{POLICY_FILE} could not be loaded ({err}); the policy engine fails closed and denies gated actions until this is fixed"
        )),
    }

    // 6. parent-sibling working dirs (informational)
    for sibling in [
        "../repos",
        "../worktrees",
        "../scratch",
        "../archives",
        "../logs",
    ] {
        if !root.join(sibling).is_dir() {
            warnings.push(format!(
                "{sibling} does not exist yet (created on first use)"
            ));
        }
    }

    report(&errors, &warnings);

    if !errors.is_empty() {
        std::process::exit(1);
    }
    if args.strict && !warnings.is_empty() {
        std::process::exit(2);
    }
    Ok(())
}

fn require_dir(root: &Path, rel: &str, errors: &mut Vec<String>) {
    if root.join(rel).is_dir() {
        println!("  ok   {rel}/");
    } else {
        errors.push(format!("missing directory: {rel}/"));
    }
}

fn require_file(root: &Path, rel: &str, sink: &mut Vec<String>) {
    if root.join(rel).is_file() {
        println!("  ok   {rel}");
    } else {
        sink.push(format!("missing file: {rel}"));
    }
}

fn check_symlink(root: &Path, link: &str, want: &str, warnings: &mut Vec<String>) {
    let p = root.join(link);
    match std::fs::read_link(&p) {
        Ok(t) if t.to_string_lossy() == want => println!("  ok   {link} -> {want}"),
        Ok(t) => warnings.push(format!("{link} -> {} (expected {want})", t.display())),
        Err(_) => warnings.push(format!("{link} is not a symlink to {want}")),
    }
}

fn report(errors: &[String], warnings: &[String]) {
    for w in warnings {
        println!("  warn {w}");
    }
    for e in errors {
        println!("  ERR  {e}");
    }
    println!(
        "doctor: {} error(s), {} warning(s)",
        errors.len(),
        warnings.len()
    );
}

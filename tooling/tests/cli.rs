//! End-to-end (black-box) tests that drive the compiled `mw` binary.
//!
//! These complement the in-module unit tests: they exercise the real CLI from
//! the outside, against a throwaway fixture workspace, and assert on stdout and
//! exit codes. No extra dependencies — `std` only. Cargo provides the built
//! binary path via `CARGO_BIN_EXE_mw`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_mw");

/// Create a unique, minimal fixture workspace under the target tmp dir and
/// return its root. It mirrors the real content layer enough for `doctor`,
/// `memory`, and `migrate` to pass.
fn fixture() -> PathBuf {
    let unique = format!(
        "mw-it-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let root = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(root.join(".agents")).unwrap();
    std::fs::create_dir_all(root.join("projects")).unwrap();

    std::fs::write(
        root.join("workspace.yaml"),
        "workspace:\n  schemaVersion: 1\n  name: fixture\nmemory:\n  profile: none # none | mempalace | prism | full\n",
    )
    .unwrap();
    std::fs::write(root.join(".agents/AGENTS.md"), "# fixture\n").unwrap();
    std::fs::write(root.join("projects/registry.yaml"), "projects: []\n").unwrap();

    #[cfg(unix)]
    for link in ["AGENTS.md", "CLAUDE.md", "GEMINI.md"] {
        std::os::unix::fs::symlink(".agents/AGENTS.md", root.join(link)).unwrap();
    }
    root
}

fn run(root: &Path, args: &[&str], stdin: &str) -> Output {
    use std::io::Write;
    let mut child = Command::new(BIN)
        .args(args)
        .current_dir(root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn mw");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().expect("wait mw")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn help_lists_commands_but_not_eject() {
    let out = run(&std::env::temp_dir(), &["--help"], "");
    assert!(out.status.success());
    let s = stdout(&out);
    for cmd in [
        "init",
        "doctor",
        "links",
        "add-project",
        "memory",
        "sdd",
        "policy",
        "migrate",
    ] {
        assert!(s.contains(cmd), "help missing `{cmd}`");
    }
    assert!(!s.contains("eject"), "eject must not be exposed");
}

#[test]
fn doctor_passes_on_clean_fixture() {
    let root = fixture();
    let out = run(&root, &["doctor"], "");
    let s = stdout(&out);
    assert!(out.status.success(), "doctor failed: {s}");
    assert!(s.contains("0 error(s)"), "expected zero errors: {s}");
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn doctor_fails_without_workspace() {
    // An empty dir has no workspace.yaml above it.
    let empty = fixture();
    std::fs::remove_file(empty.join("workspace.yaml")).unwrap();
    let out = run(&empty, &["doctor"], "");
    assert!(!out.status.success());
    std::fs::remove_dir_all(&empty).ok();
}

#[test]
fn memory_read_set_revert_roundtrip() {
    let root = fixture();
    assert!(stdout(&run(&root, &["memory"], "")).contains("profile = none"));

    // dry-run must not touch the file
    run(
        &root,
        &["memory", "--profile", "mempalace", "--dry-run"],
        "",
    );
    let yaml = std::fs::read_to_string(root.join("workspace.yaml")).unwrap();
    assert!(yaml.contains("profile: none"), "dry-run mutated file");

    run(&root, &["memory", "--profile", "prism"], "");
    let yaml = std::fs::read_to_string(root.join("workspace.yaml")).unwrap();
    assert!(yaml.contains("profile: prism # none | mempalace | prism | full"));

    run(&root, &["memory", "--profile", "none"], "");
    let yaml = std::fs::read_to_string(root.join("workspace.yaml")).unwrap();
    assert!(yaml.contains("profile: none # none | mempalace | prism | full"));
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn policy_denies_protected_path_with_exit_1() {
    let root = fixture();
    let out = run(
        &root,
        &["policy", "check"],
        r#"{"tool_name":"Write","tool_input":{"file_path":"/x/.env"}}"#,
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).contains("\"decision\":\"deny\""));
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn policy_allows_normal_path_and_empty_input() {
    let root = fixture();
    let out = run(
        &root,
        &["policy", "check"],
        r#"{"tool_name":"Write","tool_input":{"file_path":"src/main.rs"}}"#,
    );
    assert!(out.status.success());
    assert!(stdout(&out).contains("\"decision\":\"allow\""));

    // empty / malformed stdin must default to allow, never crash
    let out = run(&root, &["policy", "check"], "");
    assert!(out.status.success());
    assert!(stdout(&out).contains("\"decision\":\"allow\""));
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn hook_session_start_is_non_blocking() {
    let root = fixture();
    assert!(run(&root, &["hook", "session-start"], "").status.success());
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn add_project_requires_id() {
    let root = fixture();
    let out = run(&root, &["add-project"], "");
    assert!(!out.status.success());
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn add_project_appends_entry_and_rejects_duplicate() {
    let root = fixture();
    let reg = root.join("projects/registry.yaml");

    // dry-run must not write
    run(
        &root,
        &["add-project", "--id", "api", "--name", "API", "--dry-run"],
        "",
    );
    assert!(!std::fs::read_to_string(&reg).unwrap().contains("- id: api"));

    // real add
    let out = run(
        &root,
        &[
            "add-project",
            "--id",
            "api",
            "--name",
            "API",
            "--repo-url",
            "git@github.com:example/api.git",
        ],
        "",
    );
    assert!(out.status.success(), "{}", stdout(&out));
    let yaml = std::fs::read_to_string(&reg).unwrap();
    assert!(yaml.contains("- id: api"));
    assert!(yaml.contains("url: git@github.com:example/api.git"));
    assert!(yaml.contains("main_path: ../repos/api"));

    // duplicate id fails and does not double-write
    let out = run(&root, &["add-project", "--id", "api"], "");
    assert!(!out.status.success());
    let yaml = std::fs::read_to_string(&reg).unwrap();
    assert_eq!(yaml.matches("- id: api").count(), 1);

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn migrate_is_noop_at_current_version() {
    let root = fixture();
    let out = run(&root, &["migrate"], "");
    assert!(out.status.success());
    assert!(stdout(&out).contains("already at schemaVersion 1"));
    std::fs::remove_dir_all(&root).ok();
}

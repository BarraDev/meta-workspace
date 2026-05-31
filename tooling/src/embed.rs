//! Embedded deployable workspace template.
//!
//! The whole `template/` tree is baked into the binary at compile time via
//! `include_dir`, so `mw init` can materialize a workspace with no network and
//! no files on disk. Symlinks are NOT materialized as files: `init` skips the
//! embedded compat-link paths and recreates them as symlinks (see `links`).

use std::path::Path;

use include_dir::{include_dir, Dir};

/// The embedded `template/` directory. The crate manifest lives at the
/// repository root, so `template/` sits directly under `$CARGO_MANIFEST_DIR`.
pub static TEMPLATE: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/template");

/// Flatten the embedded template into `(relative_path, contents)` pairs.
pub fn files() -> Vec<(String, &'static [u8])> {
    let mut out = Vec::new();
    collect(&TEMPLATE, &mut out);
    out
}

fn collect(dir: &Dir<'static>, out: &mut Vec<(String, &'static [u8])>) {
    for f in dir.files() {
        out.push((f.path().to_string_lossy().replace('\\', "/"), f.contents()));
    }
    for d in dir.dirs() {
        collect(d, out);
    }
}

/// Result of materializing the template.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct InitSummary {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
}

/// Write `files` under `root`, creating parent directories. Existing files are
/// left untouched (reported as skipped). Any path equal to, or under, a
/// `skip_prefix` is ignored entirely (those are recreated as symlinks).
pub fn materialize(
    files: &[(String, &[u8])],
    root: &Path,
    skip_prefixes: &[&str],
    dry_run: bool,
) -> anyhow::Result<InitSummary> {
    let mut summary = InitSummary::default();
    for (rel, bytes) in files {
        if is_skipped(rel, skip_prefixes) {
            continue;
        }
        let dest = root.join(rel);
        if dest.exists() {
            summary.skipped.push(rel.clone());
            continue;
        }
        summary.created.push(rel.clone());
        if !dry_run {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, bytes)?;
        }
    }
    Ok(summary)
}

/// True if `rel` equals a skip prefix or lives under one (`prefix/...`).
fn is_skipped(rel: &str, skip_prefixes: &[&str]) -> bool {
    skip_prefixes
        .iter()
        .any(|p| rel == *p || rel.starts_with(&format!("{p}/")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "mw-embed-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn fileset() -> Vec<(String, &'static [u8])> {
        vec![
            ("workspace.yaml".to_string(), b"a: 1\n" as &[u8]),
            ("nested/dir/file.txt".to_string(), b"hello" as &[u8]),
            ("CLAUDE.md".to_string(), b"symlink-content" as &[u8]),
            (".claude/skills/.gitkeep".to_string(), b"" as &[u8]),
        ]
    }

    #[test]
    fn creates_files_and_parent_dirs() {
        let root = tmp();
        let s = materialize(&fileset(), &root, &[], false).unwrap();
        assert!(root.join("workspace.yaml").is_file());
        assert_eq!(
            std::fs::read(root.join("nested/dir/file.txt")).unwrap(),
            b"hello"
        );
        assert!(s.created.contains(&"workspace.yaml".to_string()));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn skips_existing_files() {
        let root = tmp();
        std::fs::write(root.join("workspace.yaml"), b"PRESERVE").unwrap();
        let s = materialize(&fileset(), &root, &[], false).unwrap();
        assert_eq!(
            std::fs::read(root.join("workspace.yaml")).unwrap(),
            b"PRESERVE"
        );
        assert!(s.skipped.contains(&"workspace.yaml".to_string()));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn skips_compat_link_prefixes() {
        let root = tmp();
        let s = materialize(&fileset(), &root, &["CLAUDE.md", ".claude/skills"], false).unwrap();
        assert!(!root.join("CLAUDE.md").exists());
        assert!(!root.join(".claude/skills/.gitkeep").exists());
        assert!(!s.created.iter().any(|p| p == "CLAUDE.md"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn dry_run_writes_nothing() {
        let root = tmp();
        let s = materialize(&fileset(), &root, &[], true).unwrap();
        assert!(!root.join("workspace.yaml").exists());
        assert!(s.created.contains(&"workspace.yaml".to_string()));
        std::fs::remove_dir_all(&root).ok();
    }
}

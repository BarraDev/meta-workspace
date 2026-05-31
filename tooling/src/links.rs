//! Agent compatibility symlinks: `AGENTS.md`, `CLAUDE.md`, `GEMINI.md` all point
//! at the canonical `.agents/AGENTS.md`. (Harness enforcement adapters \u2014 Claude
//! settings.json hooks, the Pi extension, Codex config, Gemini instructions \u2014
//! are generated separately once `.agents/policies.yaml` exists.)

use std::path::Path;

/// A canonical compatibility symlink: `name` -> `target` (target is relative to
/// the workspace root, matching how the link is stored).
pub struct LinkSpec {
    pub name: &'static str,
    pub target: &'static str,
}

/// The agent instruction symlinks every workspace carries (subset of
/// [`COMPAT_LINKS`]).
pub const AGENT_LINKS: &[LinkSpec] = &[
    LinkSpec {
        name: "AGENTS.md",
        target: ".agents/AGENTS.md",
    },
    LinkSpec {
        name: "CLAUDE.md",
        target: ".agents/AGENTS.md",
    },
    LinkSpec {
        name: "GEMINI.md",
        target: ".agents/AGENTS.md",
    },
];

/// Every compatibility symlink a deployed workspace carries: the three agent
/// instruction files plus the harness skill/command/agent links that share the
/// canonical `.agents/` content. Used by `mw init` to recreate symlinks after
/// materializing the template.
pub const COMPAT_LINKS: &[LinkSpec] = &[
    LinkSpec {
        name: "AGENTS.md",
        target: ".agents/AGENTS.md",
    },
    LinkSpec {
        name: "CLAUDE.md",
        target: ".agents/AGENTS.md",
    },
    LinkSpec {
        name: "GEMINI.md",
        target: ".agents/AGENTS.md",
    },
    LinkSpec {
        name: ".claude/agents",
        target: "../.agents/agents",
    },
    LinkSpec {
        name: ".claude/commands",
        target: "../.agents/commands",
    },
    LinkSpec {
        name: ".claude/skills",
        target: "../.agents/skills",
    },
    LinkSpec {
        name: ".pi/agents",
        target: "../.agents/agents",
    },
    LinkSpec {
        name: ".pi/skills",
        target: "../.agents/skills",
    },
];

/// Outcome of reconciling one link.
#[derive(Debug, PartialEq, Eq)]
pub enum LinkStatus {
    /// Already a correct symlink.
    Ok,
    /// Created a missing link.
    Created,
    /// Replaced a wrong/!symlink entry (only with `force`).
    Repaired,
    /// Present but wrong, left as-is because `force` was not set.
    Conflict,
    /// `--dry-run`: would create.
    WouldCreate,
    /// `--dry-run`: would repair.
    WouldRepair,
}

/// Reconcile a single link under `root`.
pub fn ensure_link(
    root: &Path,
    spec: &LinkSpec,
    force: bool,
    dry_run: bool,
) -> anyhow::Result<LinkStatus> {
    let path = root.join(spec.name);

    // Already a correct symlink?
    if let Ok(existing) = std::fs::read_link(&path) {
        if existing.to_string_lossy() == spec.target {
            return Ok(LinkStatus::Ok);
        }
        // wrong symlink target -> needs repair
        if !force {
            return Ok(LinkStatus::Conflict);
        }
        if dry_run {
            return Ok(LinkStatus::WouldRepair);
        }
        std::fs::remove_file(&path)?;
        make_symlink(spec.target, &path)?;
        return Ok(LinkStatus::Repaired);
    }

    // A non-symlink entry (regular file/dir) is a conflict.
    if path.exists() {
        if !force {
            return Ok(LinkStatus::Conflict);
        }
        if dry_run {
            return Ok(LinkStatus::WouldRepair);
        }
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
        make_symlink(spec.target, &path)?;
        return Ok(LinkStatus::Repaired);
    }

    // Missing -> create.
    if dry_run {
        return Ok(LinkStatus::WouldCreate);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    make_symlink(spec.target, &path)?;
    Ok(LinkStatus::Created)
}

#[cfg(unix)]
fn make_symlink(target: &str, link: &Path) -> anyhow::Result<()> {
    std::os::unix::fs::symlink(target, link)?;
    Ok(())
}

#[cfg(windows)]
fn make_symlink(target: &str, link: &Path) -> anyhow::Result<()> {
    use std::os::windows::fs::{symlink_dir, symlink_file};

    // The stored target is relative to the link's own directory (matching the
    // Unix symlink). Resolve it once to decide dir vs file and to drive the
    // copy fallback for file links.
    let resolved = link
        .parent()
        .map(|p| p.join(target))
        .unwrap_or_else(|| std::path::PathBuf::from(target));
    let target_is_dir = resolved.is_dir();

    let result = if target_is_dir {
        symlink_dir(target, link)
    } else {
        symlink_file(target, link)
    };

    match result {
        Ok(()) => Ok(()),
        // Windows symlink creation needs Developer Mode or elevation. For file
        // links (AGENTS/CLAUDE/GEMINI) a copy still yields a working file; for
        // directory links a copy is not a meaningful substitute, so surface a
        // clear, actionable error instead of the old silent `fs::copy` failure.
        Err(err) if target_is_dir => Err(anyhow::anyhow!(
            "could not create directory symlink {} -> {target}: {err}. \
             Enable Windows Developer Mode (or run elevated) so `mw` can create symlinks.",
            link.display()
        )),
        Err(err) => std::fs::copy(&resolved, link)
            .map(|_| ())
            .map_err(|copy_err| {
                anyhow::anyhow!(
                    "could not symlink {} -> {target} ({err}); copy fallback also failed: {copy_err}",
                    link.display()
                )
            }),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;

    fn tmp() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "mw-links-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(p.join(".agents")).unwrap();
        std::fs::write(p.join(".agents/AGENTS.md"), "#\n").unwrap();
        p
    }

    const SPEC: LinkSpec = LinkSpec {
        name: "CLAUDE.md",
        target: ".agents/AGENTS.md",
    };

    #[test]
    fn creates_missing_link() {
        let root = tmp();
        assert_eq!(
            ensure_link(&root, &SPEC, false, false).unwrap(),
            LinkStatus::Created
        );
        assert_eq!(
            std::fs::read_link(root.join("CLAUDE.md"))
                .unwrap()
                .to_string_lossy(),
            ".agents/AGENTS.md"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn ok_when_already_correct() {
        let root = tmp();
        symlink(".agents/AGENTS.md", root.join("CLAUDE.md")).unwrap();
        assert_eq!(
            ensure_link(&root, &SPEC, false, false).unwrap(),
            LinkStatus::Ok
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn conflict_without_force_then_repair_with_force() {
        let root = tmp();
        std::fs::write(root.join("CLAUDE.md"), "real file\n").unwrap();
        assert_eq!(
            ensure_link(&root, &SPEC, false, false).unwrap(),
            LinkStatus::Conflict
        );
        // file is untouched
        assert!(root.join("CLAUDE.md").is_file());
        // with force it is replaced by the symlink
        assert_eq!(
            ensure_link(&root, &SPEC, true, false).unwrap(),
            LinkStatus::Repaired
        );
        assert!(std::fs::read_link(root.join("CLAUDE.md")).is_ok());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn dry_run_does_not_write() {
        let root = tmp();
        assert_eq!(
            ensure_link(&root, &SPEC, false, true).unwrap(),
            LinkStatus::WouldCreate
        );
        assert!(!root.join("CLAUDE.md").exists());
        std::fs::remove_dir_all(&root).ok();
    }
}

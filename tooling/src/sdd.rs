//! Pure decision logic for `mw sdd` — the controlled cc-sdd control plane.
//!
//! The orchestration (spawning `npx cc-sdd`, copying staged output) lives in the
//! command; everything that can be reasoned about without a runtime lives here
//! and is unit-tested: argument validation, cc-sdd flag construction, the
//! `.sdd/manifest.json` shape (serde_json), and the line-based `sdd:` block
//! update in `workspace.yaml`.

use serde::Serialize;

use crate::workspace::set_scalar;

/// Agent target set passed to cc-sdd.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Claude,
    Codex,
    Gemini,
    All,
}

impl Target {
    /// cc-sdd skill flags for this target set.
    pub fn skill_flags(self) -> Vec<&'static str> {
        match self {
            Target::Claude => vec!["--claude-skills"],
            Target::Codex => vec!["--codex-skills"],
            Target::Gemini => vec!["--gemini-skills"],
            Target::All => vec!["--claude-skills", "--codex-skills", "--gemini-skills"],
        }
    }

    /// Agent ids recorded as installed.
    pub fn installed_agents(self) -> Vec<&'static str> {
        match self {
            Target::Claude => vec!["claude"],
            Target::Codex => vec!["codex"],
            Target::Gemini => vec!["gemini"],
            Target::All => vec!["claude", "codex", "gemini"],
        }
    }

    /// The `selected_target` label.
    pub fn label(self) -> &'static str {
        match self {
            Target::Claude => "claude",
            Target::Codex => "codex",
            Target::Gemini => "gemini",
            Target::All => "all",
        }
    }
}

/// Validate the mode/policy combination, mirroring install-sdd.sh:
/// staged requires vendor; direct requires replace.
pub fn validate(mode: &str, policy: &str) -> anyhow::Result<()> {
    match mode {
        "staged" | "direct" => {}
        other => anyhow::bail!("unsupported install mode: {other}"),
    }
    match policy {
        "vendor" | "replace" => {}
        other => anyhow::bail!("unsupported memory policy: {other}"),
    }
    match (mode, policy) {
        ("staged", "replace") => {
            anyhow::bail!("--memory-policy=replace only applies to --mode=direct")
        }
        ("direct", "vendor") => anyhow::bail!(
            "direct mode requires --memory-policy=replace so CLAUDE.md changes are explicit"
        ),
        _ => Ok(()),
    }
}

/// The `.sdd/manifest.json` document.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Manifest {
    pub enabled: bool,
    pub provider: String,
    pub version: String,
    pub installed_at: String,
    pub selected_target: String,
    pub installed_agents: Vec<String>,
    pub install_mode: String,
    pub memory_policy: String,
    pub generated_memory_document: String,
    pub kiro_dir: String,
}

/// Build the manifest for a completed install.
pub fn build_manifest(target: Target, mode: &str, policy: &str, installed_at: &str) -> Manifest {
    let generated_memory_document = if mode == "staged" {
        ".agents/vendor/cc-sdd/CLAUDE.md"
    } else {
        "CLAUDE.md"
    };
    Manifest {
        enabled: true,
        provider: "cc-sdd".to_string(),
        version: "latest".to_string(),
        installed_at: installed_at.to_string(),
        selected_target: target.label().to_string(),
        installed_agents: target
            .installed_agents()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        install_mode: mode.to_string(),
        memory_policy: policy.to_string(),
        generated_memory_document: generated_memory_document.to_string(),
        kiro_dir: ".kiro".to_string(),
    }
}

/// Update the `sdd:` block of `workspace.yaml` (line-based): enable it and set
/// install_mode, memory_policy, and installed_agents.
pub fn update_workspace_sdd(
    yaml: &str,
    target: Target,
    mode: &str,
    policy: &str,
) -> anyhow::Result<String> {
    // sdd.enabled is the first `enabled:` key in workspace.yaml (the memory
    // block's enabled keys are deeper and come later), so first-match is safe.
    let agents = target.installed_agents().join(", ");
    let mut out = set_scalar(yaml, "enabled", "true")?;
    out = set_scalar(&out, "install_mode", mode)?;
    out = set_scalar(&out, "memory_policy", policy)?;
    out = set_scalar(&out, "installed_agents", &format!("[{agents}]"))?;
    Ok(out)
}

/// Format a Unix timestamp (seconds) as an RFC3339 UTC string, dependency-free.
pub fn epoch_to_rfc3339(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    // Howard Hinnant's civil_from_days.
    let mut z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    let _ = &mut z;
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Current time as an RFC3339 UTC string.
pub fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    epoch_to_rfc3339(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_enforces_mode_policy_pairing() {
        assert!(validate("staged", "vendor").is_ok());
        assert!(validate("direct", "replace").is_ok());
        // staged must not replace the live memory doc
        assert!(validate("staged", "replace").is_err());
        // direct must be explicit about replacing
        assert!(validate("direct", "vendor").is_err());
        // unknown values rejected
        assert!(validate("bogus", "vendor").is_err());
    }

    #[test]
    fn target_flags_and_agents() {
        assert_eq!(Target::Claude.skill_flags(), ["--claude-skills"]);
        assert_eq!(
            Target::All.skill_flags(),
            ["--claude-skills", "--codex-skills", "--gemini-skills"]
        );
        assert_eq!(
            Target::All.installed_agents(),
            ["claude", "codex", "gemini"]
        );
        assert_eq!(Target::Codex.installed_agents(), ["codex"]);
    }

    #[test]
    fn manifest_reflects_mode_for_memory_doc() {
        let staged = build_manifest(Target::Claude, "staged", "vendor", "2026-01-01T00:00:00Z");
        assert!(staged.enabled);
        assert_eq!(staged.provider, "cc-sdd");
        assert_eq!(staged.selected_target, "claude");
        assert_eq!(
            staged.generated_memory_document,
            ".agents/vendor/cc-sdd/CLAUDE.md"
        );

        let direct = build_manifest(Target::All, "direct", "replace", "2026-01-01T00:00:00Z");
        assert_eq!(direct.generated_memory_document, "CLAUDE.md");
        assert_eq!(direct.installed_agents, ["claude", "codex", "gemini"]);

        // serializes to valid JSON
        let json = serde_json::to_string(&staged).unwrap();
        assert!(json.contains("\"provider\":\"cc-sdd\""));
    }

    const WS: &str = "sdd:\n  enabled: false\n  provider: cc-sdd\n  install_mode: staged # staged | direct\n  memory_policy: vendor # vendor | replace\n  installed_agents: []\nmemory:\n  mempalace:\n    enabled: false\n";

    #[test]
    fn updates_only_the_sdd_block() {
        let out = update_workspace_sdd(WS, Target::All, "direct", "replace").unwrap();
        assert!(out.contains("  enabled: true"));
        assert!(out.contains("  install_mode: direct # staged | direct"));
        assert!(out.contains("  memory_policy: replace # vendor | replace"));
        assert!(out.contains("  installed_agents: [claude, codex, gemini]"));
        // the memory block's enabled stays false (sdd.enabled comes first)
        assert!(out.contains("    enabled: false"));
    }

    #[test]
    fn epoch_formats_to_rfc3339() {
        assert_eq!(epoch_to_rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(epoch_to_rfc3339(86_400), "1970-01-02T00:00:00Z");
        assert_eq!(epoch_to_rfc3339(1_700_000_000), "2023-11-14T22:13:20Z");
    }
}

//! Pure scaffolding helpers shared by `init` and `memory`: `.env.local` upserts
//! and `company/profile.md` stamping. Kept line-based and dependency-free so the
//! commands stay testable without touching the filesystem.

/// Insert or replace a `KEY=value` line in a dotenv-style document. Preserves
/// other lines and order; appends if the key is absent.
pub fn upsert_env(content: &str, key: &str, value: &str) -> String {
    let prefix = format!("{key}=");
    let mut replaced = false;
    let mut out: Vec<String> = content
        .lines()
        .map(|line| {
            if line.trim_start().starts_with(&prefix) {
                replaced = true;
                format!("{key}={value}")
            } else {
                line.to_string()
            }
        })
        .collect();
    if !replaced {
        out.push(format!("{key}={value}"));
    }
    let mut result = out.join("\n");
    result.push('\n');
    result
}

/// Render a fresh `.env.local` for a workspace slug and memory profile.
pub fn render_env_local(slug: &str, profile: &str) -> String {
    format!("MEMPALACE_WING={slug}\nPRISM_PROJECT={slug}\nMEMORY_PROFILE={profile}\n")
}

/// Stamp `- Name:` and `- Slug:` lines in `company/profile.md`.
pub fn stamp_profile(md: &str, name: &str, slug: &str) -> String {
    md.lines()
        .map(|line| {
            let t = line.trim_start();
            if t.starts_with("- Name:") {
                format!("- Name: {name}")
            } else if t.starts_with("- Slug:") {
                format!("- Slug: {slug}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + if md.ends_with('\n') { "\n" } else { "" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_replaces_existing_key_in_place() {
        let env = "MEMPALACE_WING=old\nPRISM_PROJECT=old\nMEMORY_PROFILE=none\n";
        let out = upsert_env(env, "MEMORY_PROFILE", "mempalace");
        assert!(out.contains("MEMORY_PROFILE=mempalace"));
        assert!(out.contains("MEMPALACE_WING=old"));
        // exactly one MEMORY_PROFILE line
        assert_eq!(out.matches("MEMORY_PROFILE=").count(), 1);
    }

    #[test]
    fn upsert_appends_missing_key() {
        let out = upsert_env("A=1\n", "B", "2");
        assert!(out.contains("A=1"));
        assert!(out.contains("B=2"));
    }

    #[test]
    fn render_env_local_has_all_keys() {
        let out = render_env_local("acme", "mempalace");
        assert!(out.contains("MEMPALACE_WING=acme"));
        assert!(out.contains("PRISM_PROJECT=acme"));
        assert!(out.contains("MEMORY_PROFILE=mempalace"));
    }

    #[test]
    fn stamp_profile_sets_name_and_slug() {
        let md = "# Company Profile\n\n## Company\n\n- Name:\n- Slug:\n- Website:\n";
        let out = stamp_profile(md, "Acme Inc", "acme");
        assert!(out.contains("- Name: Acme Inc"));
        assert!(out.contains("- Slug: acme"));
        // unrelated lines preserved
        assert!(out.contains("- Website:"));
    }
}

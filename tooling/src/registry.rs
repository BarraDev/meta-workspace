//! Append-with-duplicate-check editing for `projects/registry.yaml`.
//!
//! Per the contract, `mw` owns the shape of this file and edits it with
//! line-based operations rather than a YAML parser. The registry is a single
//! top-level `projects:` list; entries are appended in block form and duplicate
//! ids are rejected.

/// A new project to append to the registry.
#[derive(Debug, Clone)]
pub struct NewProject {
    pub id: String,
    pub name: String,
    pub repo_url: Option<String>,
    pub default_branch: String,
    pub repos_dir: String,
    pub worktrees_dir: String,
}

/// Returns true if the registry already contains a project with `id`.
pub fn has_id(yaml: &str, id: &str) -> bool {
    let needle = format!("- id: {id}");
    yaml.lines().any(|l| {
        l.trim_start().starts_with(&needle) && {
            // ensure exact id match, not a prefix (e.g. `api` vs `api-v2`)
            let rest = l.trim_start().trim_start_matches("- id:").trim();
            let rest = rest.split_once(" #").map(|(v, _)| v).unwrap_or(rest);
            rest.trim_matches(|c| c == '"' || c == '\'') == id
        }
    })
}

/// Render a single block-list entry (indented under `projects:`), without a
/// trailing newline.
pub fn render_entry(p: &NewProject) -> String {
    let mut s = String::new();
    s.push_str(&format!("  - id: {}\n", p.id));
    s.push_str(&format!("    name: {}\n", p.name));
    s.push_str("    repository:\n");
    if let Some(url) = &p.repo_url {
        s.push_str(&format!("      url: {url}\n"));
    }
    s.push_str(&format!("      main_path: {}/{}\n", p.repos_dir, p.id));
    s.push_str(&format!("      default_branch: {}\n", p.default_branch));
    s.push_str("    worktrees:\n");
    s.push_str(&format!("      root: {}\n", p.worktrees_dir));
    s.push_str(&format!("      naming: {}-{{task-or-branch}}", p.id));
    s
}

/// Append `p` to the `projects:` list, returning the rewritten document.
/// Errors if the id already exists or no `projects:` key is found.
pub fn append(yaml: &str, p: &NewProject) -> anyhow::Result<String> {
    if has_id(yaml, &p.id) {
        anyhow::bail!("project id `{}` already exists in registry", p.id);
    }

    let lines: Vec<&str> = yaml.lines().collect();
    let key_idx = lines
        .iter()
        .position(|l| l.trim_start().starts_with("projects:"))
        .ok_or_else(|| anyhow::anyhow!("no `projects:` key found in registry"))?;

    // Normalize an inline empty list (`projects: []`) to a block key.
    let mut out_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    let inline_empty = lines[key_idx].trim_end().ends_with("[]");
    if inline_empty {
        out_lines[key_idx] = "projects:".to_string();
    }

    // Find where the projects block ends: the first blank line or column-0 line
    // after the key. The new entry is inserted just before that boundary so any
    // trailing comments stay put.
    let mut insert_at = out_lines.len();
    for (i, line) in out_lines.iter().enumerate().skip(key_idx + 1) {
        let is_blank = line.trim().is_empty();
        let is_dedented = !line.starts_with(char::is_whitespace);
        if is_blank || is_dedented {
            insert_at = i;
            break;
        }
    }

    let entry = render_entry(p);
    out_lines.insert(insert_at, entry);

    let mut result = out_lines.join("\n");
    if yaml.ends_with('\n') {
        result.push('\n');
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_project() -> NewProject {
        NewProject {
            id: "api".into(),
            name: "API".into(),
            repo_url: Some("git@github.com:example/api.git".into()),
            default_branch: "main".into(),
            repos_dir: "../repos".into(),
            worktrees_dir: "../worktrees".into(),
        }
    }

    const EMPTY: &str = "projects: []\n\n# Example: see template docs\n";
    const ONE: &str =
        "projects:\n  - id: web\n    name: Web\n    repository:\n      default_branch: main\n";

    #[test]
    fn has_id_detects_existing_and_missing() {
        assert!(has_id(ONE, "web"));
        assert!(!has_id(ONE, "api"));
        assert!(!has_id(EMPTY, "api"));
    }

    #[test]
    fn render_entry_includes_core_fields() {
        let e = render_entry(&sample_project());
        assert!(e.contains("  - id: api"));
        assert!(e.contains("    name: API"));
        assert!(e.contains("url: git@github.com:example/api.git"));
        assert!(e.contains("default_branch: main"));
        assert!(e.contains("main_path: ../repos/api"));
        assert!(e.contains("root: ../worktrees"));
        // a missing repo_url must be omitted, not rendered as empty
        let mut no_url = sample_project();
        no_url.repo_url = None;
        assert!(!render_entry(&no_url).contains("url:"));
    }

    #[test]
    fn append_converts_inline_empty_list() {
        let out = append(EMPTY, &sample_project()).unwrap();
        assert!(out.contains("projects:\n  - id: api"));
        // the trailing example comment is preserved
        assert!(out.contains("# Example: see template docs"));
        assert!(!out.contains("projects: []"));
    }

    #[test]
    fn append_adds_after_existing_entries() {
        let out = append(ONE, &sample_project()).unwrap();
        assert!(out.contains("- id: web"));
        assert!(out.contains("- id: api"));
        // web stays before api (append, not prepend)
        let wi = out.find("id: web").unwrap();
        let ai = out.find("id: api").unwrap();
        assert!(wi < ai);
    }

    #[test]
    fn append_rejects_duplicate_id() {
        let dup = NewProject {
            id: "web".into(),
            ..sample_project()
        };
        assert!(append(ONE, &dup).is_err());
    }

    #[test]
    fn append_result_still_parses_each_id_once() {
        let out = append(ONE, &sample_project()).unwrap();
        assert_eq!(out.matches("- id: ").count(), 2);
    }
}

# Install This Meta-Workspace Template

Use this template for one company at a time. Keep application repositories and worktrees outside the meta-workspace folder.

## Recommended folder layout

```text
work-root/
├── meta-workspace/
├── repos/
├── worktrees/
├── scratch/
├── archives/
└── logs/
```

## Fresh setup

From the intended meta-workspace folder:

```bash
mw init
mw doctor
```

Non-interactive example:

```bash
mw init \
  --company-name "Example Company" \
  --company-id example-company
```

`mw init` will create or verify these parent sibling folders by default:

- `../repos`
- `../worktrees`
- `../scratch`
- `../archives`
- `../logs`

## Add a project

```bash
mw add-project \
  --id api \
  --name "API" \
  --repo-url "git@github.com:example/api.git" \
  --default-branch main
```

This updates `projects/registry.yaml` (rejecting duplicate ids). It does not clone repositories or create worktrees.

## Optional memory

```bash
mw memory --profile mempalace --slug example-company
```

Supported profiles:

- `none`
- `mempalace`
- `prism`
- `full`

This updates the memory profile in `workspace.yaml` and mirrors it into `.env.local`. Do not store secrets in either file.

## Optional SDD/Kiro

```bash
mw sdd install --dry-run-only --targets claude
mw sdd install
```

The SDD installer uses `cc-sdd`, always runs a dry run first, and applies changes in a controlled way.

By default, it applies `cc-sdd` in a temporary staging directory, copies generated skills/settings into controlled locations, stores the generated memory document at `.agents/vendor/cc-sdd/CLAUDE.md`, and preserves the live `CLAUDE.md -> .agents/AGENTS.md` symlink.

Use direct mode only if you intentionally want `cc-sdd` to write live tool files:

```bash
mw sdd install --mode direct --memory-policy replace --targets claude
```

## Agent-assisted installation checklist

AI coding agents should:

1. Confirm the current directory is the intended meta-workspace.
2. List existing files and ask before overwriting any real files.
3. Run `mw init` (it preserves existing files and repairs symlinks).
4. Keep application repositories in `../repos` and worktrees in `../worktrees` unless the user says otherwise.
5. Run `mw doctor`.
6. Report changed files, commands run, and remaining manual steps.

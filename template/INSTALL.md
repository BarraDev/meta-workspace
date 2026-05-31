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
./scripts/bootstrap.sh
./scripts/doctor.sh
```

Non-interactive example:

```bash
./scripts/bootstrap.sh \
  --name="Example Company" \
  --slug="example-company" \
  --init-git=yes \
  --create-dirs=yes \
  --non-interactive
```

Bootstrap will create or verify these parent sibling folders by default:

- `../repos`
- `../worktrees`
- `../scratch`
- `../archives`
- `../logs`

## Add a project

```bash
./scripts/new-project.sh
```

Non-interactive example:

```bash
./scripts/new-project.sh \
  --id=api \
  --name="API" \
  --repo-url="git@github.com:example/api.git" \
  --default-branch=main \
  --language=go \
  --non-interactive
```

The helper updates `projects/registry.yaml` and creates a project instruction stub under `docs/instructions/` by default. It does not clone repositories or create worktrees.

## Optional memory

```bash
./scripts/install-memory.sh
```

Non-interactive example:

```bash
./scripts/install-memory.sh --profile=mempalace --slug=example-company --non-interactive
```

Supported profiles:

- `none`
- `mempalace`
- `prism`
- `full`

This writes `.env.local` and updates the memory section in `workspace.yaml`. Do not store secrets in either file.

## Optional SDD/Kiro

```bash
./scripts/install-sdd.sh --dry-run-only --targets=claude
./scripts/install-sdd.sh
```

The SDD installer uses `cc-sdd`, always runs a dry run first, and asks before applying changes.

By default, it applies `cc-sdd` in a temporary staging directory, copies generated skills/settings into controlled locations, stores the generated memory document at `.agents/vendor/cc-sdd/CLAUDE.md`, and preserves the live `CLAUDE.md -> .agents/AGENTS.md` symlink.

Use direct mode only if you intentionally want `cc-sdd` to write live tool files:

```bash
./scripts/install-sdd.sh --mode=direct --memory-policy=replace --targets=claude
```

## Agent-assisted installation checklist

AI coding agents should:

1. Confirm the current directory is the intended meta-workspace.
2. List existing files and ask before overwriting any real files.
3. Run `./scripts/bootstrap.sh` or reproduce its steps explicitly.
4. Keep application repositories in `../repos` and worktrees in `../worktrees` unless the user says otherwise.
5. Run `./scripts/doctor.sh`.
6. Report changed files, commands run, and remaining manual steps.

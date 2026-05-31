# Generic Meta-Workspace Template

This repository is a **template for creating one company meta-workspace**. It is not an application repository and should not contain application source code.

Use a meta-workspace to coordinate one company's projects, repositories, worktrees, agent instructions, specs, decisions, memory configuration, and PR workflows.

## What this template provides

- A reusable one-company-at-a-time workspace structure.
- Parent sibling folders for repositories and worktrees.
- Canonical agent instructions under `.agents/`.
- Compatibility links for Claude, Pi, Gemini, and related agent tools.
- Optional memory configuration for MemPalace, Prism, or both.
- Optional SDD/Kiro support installed through `cc-sdd`.
- Project registry and the `mw` tool for adding repositories.
- Draft-only PR workflow guidance for AI agents.

## Recommended layout

Create one work root per company or client:

```text
work-root/
├── meta-workspace/   # this repository/template instance
├── repos/            # clean main checkouts
├── worktrees/        # feature, fix, experiment, or PR worktrees
├── scratch/          # temporary human/agent workspace
├── archives/         # old handoffs or exports
└── logs/             # optional runtime logs
```

The default paths are intentionally outside the meta-workspace:

- repositories: `../repos`
- worktrees: `../worktrees`
- scratch: `../scratch`
- archives: `../archives`
- logs: `../logs`

## First run

The `mw` tool materializes and maintains a workspace:

```bash
mw init
mw doctor
```

Non-interactive setup example:

```bash
mw init \
  --company-name "Example Company" \
  --company-id example-company

mw doctor
```

`mw init` creates the parent folders (`../repos`, `../worktrees`, ...), writes
`.env.local`, stamps `company/profile.md`, and recreates the agent compatibility
symlinks. It is idempotent, so re-running it repairs a workspace.

## Add projects

```bash
mw add-project \
  --id api \
  --name "API" \
  --repo-url "git@github.com:example/api.git" \
  --default-branch main
```

This updates `projects/registry.yaml` (rejecting duplicate ids). It does not
clone repositories.

## Optional memory

```bash
mw memory --profile mempalace --slug example-company
```

Supported profiles:

- `none`
- `mempalace`
- `prism`
- `full`

## Optional SDD/Kiro

Dry run only:

```bash
mw sdd install --dry-run-only --targets claude
```

Controlled staged install:

```bash
mw sdd install --targets claude
```

By default, SDD install runs `cc-sdd` in a temporary staging directory and preserves the live `CLAUDE.md` symlink to `.agents/AGENTS.md`. The generated `cc-sdd` memory document is stored at:

```text
.agents/vendor/cc-sdd/CLAUDE.md
```

Use direct mode only when you intentionally want `cc-sdd` to write directly into live tool files:

```bash
mw sdd install --mode direct --memory-policy replace --targets claude
```

## Important files

- `workspace.yaml` — workspace paths and optional feature settings.
- `company/profile.md` — company profile for this workspace instance.
- `projects/registry.yaml` — project and repository registry.
- `.agents/AGENTS.md` — canonical AI-agent instructions.
- `INSTALL.md` — installation and setup guide.
- `BOOTSTRAP.md` — agent-assisted bootstrap procedure.
- `docs/distribution.md` — distribution/package options for this template.

## Safety rules

- Do not store secrets in this repository.
- Do not clone application repositories into this folder.
- Keep clean main checkouts in `../repos` by default.
- Use worktrees in `../worktrees` for feature work, fixes, experiments, and PR reviews.
- AI agents must not post PR comments, approvals, external messages, or status updates without explicit user approval.

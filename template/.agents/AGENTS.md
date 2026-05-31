# Universal Meta-Workspace Agent Instructions

This is a **company meta-workspace**, not an application repository.

Use this workspace to coordinate one company's projects, repositories, worktrees, specifications, decisions, PR work, documentation, and memory.

## Canonical support folder

`.agents/` is the canonical source of truth for agent support.

Tool-specific files and folders are compatibility layers:

- `AGENTS.md` should point to `.agents/AGENTS.md`.
- `CLAUDE.md` should point to `.agents/AGENTS.md`.
- `GEMINI.md` should point to `.agents/AGENTS.md`.
- `.claude/skills`, `.pi/skills`, and similar folders should link to canonical or managed folders when possible.

Do not duplicate instructions across tool-specific files.

## Language

Use English for code, commits, PRs, documentation, project files, and persistent memory unless the user explicitly requests otherwise.

## Workspace boundaries

- Keep application repositories outside this meta-workspace by default.
- Default clean repository path: `../repos`.
- Default worktree path: `../worktrees`.
- Keep clean main checkouts on their default branch when possible.
- Prefer worktrees for feature work, fixes, experiments, and PR review.

## Before working on code

1. Identify the project in `projects/registry.yaml`.
2. Confirm the repository path and default branch.
3. Use a worktree for implementation or PR review unless the user explicitly chooses otherwise.
4. Read project-specific instructions before editing code.
5. Validate current state with live commands; do not rely only on memory.

## PR work

Default behavior is draft-only. Do not post PR comments, approvals, external messages, or status updates without explicit user approval.

Before PR review or PR changes, identify:

1. repository
2. PR number/URL
3. PR author
4. base branch and head branch
5. whether the user wants review, fixes, or both
6. whether posting is allowed

## Memory

Use available memory systems when configured, but never trust memory blindly. Always validate against the live filesystem, git state, remote services, and current user instructions.

Supported optional systems:

- recall/preferences when available in the agent harness
- MemPalace CLI
- Prism MCP

## SDD/Kiro

SDD/Kiro workflow is optional. When enabled, use `cc-sdd` as the source of SDD skills and templates instead of maintaining stale copied skills.

## Safety

- Follow `.agents/policies.yaml` as the harness-neutral workspace policy file.
- Ask before destructive changes.
- Do not store secrets in repository files.
- Avoid mining huge folders or private data without explicit scope.
- Prefer small, reviewable changes.
- Report commands run, files changed, and follow-up steps clearly.

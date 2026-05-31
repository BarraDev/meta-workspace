# Meta-Workspace Bootstrap Guide

This document is written for humans and AI coding agents.

## Goal

Install a minimal company meta-workspace in the current directory. The workspace should coordinate projects and repositories without mixing application source code into this repository.

## Agent procedure

If you are an AI coding agent installing this template:

1. Confirm the current directory is the intended meta-workspace directory.
2. If existing files are present, list them and ask before overwriting anything.
3. Ask the setup questions below.
4. Create or verify parent folders: `../repos`, `../worktrees`, `../scratch`, `../archives`, `../logs`.
5. Create or repair agent compatibility symlinks.
6. Initialize git if requested.
7. Optionally configure memory.
8. Optionally install SDD/Kiro skills using the controlled `./scripts/install-sdd.sh` wrapper around `cc-sdd`.
9. Add projects with `./scripts/new-project.sh` or edit `projects/registry.yaml` manually.
10. Run `./scripts/doctor.sh`.
11. Report what changed and what remains.

## Setup questions

Required:

1. What is the company/workspace name?
2. What slug should identify this workspace?
3. Should this folder be initialized as a git repository?
4. Confirm parent path defaults:
   - repositories: `../repos`
   - worktrees: `../worktrees`
   - scratch: `../scratch`
   - archives: `../archives`
   - logs: `../logs`

Agent support:

5. Which tools should be supported? Default: Claude Code, Pi, Codex CLI, Gemini CLI.

SDD/Kiro:

6. Enable SDD/Kiro with `cc-sdd`?
   - `no`
   - `dry run only`
   - `controlled staged install now`

The default staged installer preserves `CLAUDE.md -> .agents/AGENTS.md` and stores the generated cc-sdd memory document at `.agents/vendor/cc-sdd/CLAUDE.md`.

Memory:

7. Enable memory?
   - `none`
   - `mempalace`
   - `prism`
   - `full` (MemPalace + Prism)

PR workflow:

8. Keep PR comments draft-only by default? Recommended: yes.

## Safety rules

- Do not store secrets in this repository.
- Do not clone application repositories into this folder.
- Use `../repos` for clean main checkouts and `../worktrees` for implementation work by default.
- Do not work directly in clean main checkouts when a worktree is appropriate.
- Do not post PR comments or external messages without explicit user approval.

# Project Registry

Use `projects/registry.yaml` as the source of truth for repositories coordinated by this workspace.

Each project should document:

- stable project id
- display name
- repository URL
- clean checkout path under `../repos`
- worktree root under `../worktrees`
- default branch
- language/framework
- project-specific instruction file

Agents should check the registry before cloning repositories, creating worktrees, or editing code.

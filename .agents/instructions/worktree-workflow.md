# Worktree Workflow

Default paths:

- clean repositories: `../repos`
- worktrees: `../worktrees`

Recommended pattern:

```bash
git -C ../repos/<repo> fetch --all --prune
git -C ../repos/<repo> worktree add ../worktrees/<repo>-<task> -b <branch> origin/main
```

Guidelines:

1. Keep clean checkouts stable and close to the default branch.
2. Do implementation, fixes, and PR review in worktrees.
3. Use clear worktree names that include repository and task/PR.
4. Remove stale worktrees only after confirming they contain no uncommitted work.
5. Run tests from the worktree, not from the meta-workspace.

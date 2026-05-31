# Distribution Options

A distribution path is the way this generic meta-workspace template will be delivered and reused for a new company workspace.

The template itself is reusable. Each installed copy should become a company-specific meta-workspace.

## Option 1: Git template repository

Publish this folder as a Git repository and create new workspaces from it by cloning or using the hosting provider's template feature.

Pros:

- Versioned history.
- Easy updates through Git.
- Familiar workflow for developers and agents.

Cons:

- Users must remember to customize company fields after cloning.
- Template repository history remains visible unless copied with a fresh history.

Example:

```bash
git clone <template-repo-url> meta-workspace
cd meta-workspace
./scripts/bootstrap.sh
```

## Option 2: Archive release

Package this folder as a `.tar.gz` or `.zip` release and unpack it into a new `meta-workspace` folder.

Pros:

- Simple for non-Git installation.
- Can be attached to releases.
- Can avoid carrying template Git history.

Cons:

- Harder to update after installation.
- Requires a release process.

## Option 3: Copy/degit style install

Use a copy tool such as `degit`, `git archive`, or a documented copy command to materialize the template without Git history.

Pros:

- Clean new workspace history.
- Good for one-time template instantiation.

Cons:

- Requires another tool or documented command.
- Template updates are not automatically connected.

## Option 4: Bootstrap-from-URL script

Provide a future `scripts/bootstrap-from-github.sh` or a documented `curl` flow that downloads the template into an empty folder and runs bootstrap.

Pros:

- Fastest user experience.
- Good for automation.

Cons:

- Must be designed carefully to avoid overwriting user files.
- Remote shell install flows require extra trust and review.

## Current recommendation

Start with **Option 1: Git template repository** because it is transparent, versioned, and easy to review.

For clean company instances without template history, add an archive or `degit` workflow later.

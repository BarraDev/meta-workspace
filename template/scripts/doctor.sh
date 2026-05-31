#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

fail=0

check_path() {
  local path="$1"
  if [ -e "$path" ] || [ -L "$path" ]; then
    echo "ok: $path"
  else
    echo "missing: $path"
    fail=1
  fi
}

check_path workspace.yaml
check_path .agents/AGENTS.md
check_path projects/registry.yaml
check_path company/profile.md

if ./scripts/check-symlinks.sh; then
  echo "ok: symlinks"
else
  echo "symlink check failed; run ./scripts/install-agent-links.sh"
  fail=1
fi

for dir in ../repos ../worktrees ../scratch ../archives ../logs; do
  if [ -d "$dir" ]; then
    echo "ok: $dir"
  else
    echo "missing parent dir: $dir"
    fail=1
  fi
done

if command -v git >/dev/null 2>&1; then
  echo "ok: git $(git --version | awk '{print $3}')"
else
  echo "missing: git"
  fail=1
fi

if command -v npx >/dev/null 2>&1; then
  echo "ok: npx available"
else
  echo "notice: npx not available; cc-sdd install will not work until Node/npm are installed"
fi

if command -v mempalace >/dev/null 2>&1; then
  echo "ok: mempalace available"
else
  echo "notice: mempalace not available; memory warm-up will be skipped"
fi

exit $fail

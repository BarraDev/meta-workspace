#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

status=0
for path in AGENTS.md CLAUDE.md GEMINI.md .claude/agents .claude/commands .claude/skills .pi/agents .pi/skills; do
  if [ ! -L "$path" ]; then
    echo "missing symlink: $path"
    status=1
  else
    echo "ok: $path -> $(readlink "$path")"
  fi
done

exit $status

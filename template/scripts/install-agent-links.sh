#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

link_file() {
  local target="$1"
  local link="$2"
  if [ -e "$link" ] && [ ! -L "$link" ]; then
    echo "skip $link: real file exists"
    return 0
  fi
  rm -f "$link"
  ln -s "$target" "$link"
  echo "linked $link -> $target"
}

link_dir() {
  local target="$1"
  local link="$2"
  if [ -e "$link" ] && [ ! -L "$link" ]; then
    echo "skip $link: real directory/file exists"
    return 0
  fi
  rm -f "$link"
  ln -s "$target" "$link"
  echo "linked $link -> $target"
}

mkdir -p .agents/{agents,commands,skills,hooks,instructions,templates,vendor} .claude .pi .codex

link_file ".agents/AGENTS.md" "AGENTS.md"
link_file ".agents/AGENTS.md" "CLAUDE.md"
link_file ".agents/AGENTS.md" "GEMINI.md"

link_dir "../.agents/agents" ".claude/agents"
link_dir "../.agents/commands" ".claude/commands"
link_dir "../.agents/skills" ".claude/skills"

link_dir "../.agents/agents" ".pi/agents"
link_dir "../.agents/skills" ".pi/skills"

echo "agent compatibility links installed"

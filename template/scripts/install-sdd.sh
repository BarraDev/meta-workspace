#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

DRY_RUN_ONLY=false
YES=false
PRESET_TARGETS="${SDD_TARGETS:-}"
INSTALL_MODE="staged"
MEMORY_POLICY="vendor"

usage() {
  cat <<'EOF'
Usage: ./scripts/install-sdd.sh [options]

Options:
  --dry-run-only                 Run cc-sdd dry run and stop.
  --targets=<set>                claude | codex | gemini | all
  --mode=<mode>                  staged | direct  (default: staged)
  --memory-policy=<policy>       vendor | replace (default: vendor)
  --yes, -y                      Do not prompt before applying.
  -h, --help                     Show this help.

Default behavior is controlled and safe:
  - cc-sdd is applied in a temporary staging directory.
  - Generated skills/settings are copied into this workspace.
  - Generated CLAUDE.md is saved as .agents/vendor/cc-sdd/CLAUDE.md.
  - The live CLAUDE.md symlink to .agents/AGENTS.md is preserved.

Use --mode=direct --memory-policy=replace only if you intentionally want cc-sdd
to write directly into this workspace, including CLAUDE.md.
EOF
}

for arg in "$@"; do
  case "$arg" in
    --dry-run-only) DRY_RUN_ONLY=true ;;
    --targets=*) PRESET_TARGETS="${arg#--targets=}" ;;
    --mode=*) INSTALL_MODE="${arg#--mode=}" ;;
    --memory-policy=*) MEMORY_POLICY="${arg#--memory-policy=}" ;;
    --yes|-y) YES=true ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unsupported argument: $arg"; usage; exit 1 ;;
  esac
done

case "$INSTALL_MODE" in
  staged|direct) ;;
  *) echo "Unsupported install mode: $INSTALL_MODE"; exit 1 ;;
esac

case "$MEMORY_POLICY" in
  vendor|replace) ;;
  *) echo "Unsupported memory policy: $MEMORY_POLICY"; exit 1 ;;
esac

if [ "$INSTALL_MODE" = "staged" ] && [ "$MEMORY_POLICY" = "replace" ]; then
  echo "--memory-policy=replace only applies to --mode=direct"
  exit 1
fi

if ! command -v npx >/dev/null 2>&1; then
  echo "npx is required to run cc-sdd. Install Node.js/npm first."
  exit 1
fi

if [ -n "$PRESET_TARGETS" ]; then
  TARGETS="$PRESET_TARGETS"
else
  read -r -p "cc-sdd agent targets (claude,codex,gemini,all) [claude]: " TARGETS
  TARGETS="${TARGETS:-claude}"
fi

TARGETS="$(printf '%s' "$TARGETS" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"

ARGS=("--kiro-dir" ".kiro" "--backup" "--overwrite" "skip")
APPLY_ARGS=("--kiro-dir" ".kiro" "--backup" "--overwrite" "force" "--yes")
INSTALLED_AGENTS_JSON="[]"
case "$TARGETS" in
  claude)
    ARGS+=("--claude-skills")
    APPLY_ARGS+=("--claude-skills")
    INSTALLED_AGENTS_JSON='["claude"]'
    ;;
  codex)
    ARGS+=("--codex-skills")
    APPLY_ARGS+=("--codex-skills")
    INSTALLED_AGENTS_JSON='["codex"]'
    ;;
  gemini)
    ARGS+=("--gemini-skills")
    APPLY_ARGS+=("--gemini-skills")
    INSTALLED_AGENTS_JSON='["gemini"]'
    ;;
  all)
    ARGS+=("--claude-skills" "--codex-skills" "--gemini-skills")
    APPLY_ARGS+=("--claude-skills" "--codex-skills" "--gemini-skills")
    INSTALLED_AGENTS_JSON='["claude","codex","gemini"]'
    ;;
  *) echo "Unsupported target set: $TARGETS"; exit 1 ;;
esac

mkdir -p .sdd .kiro .agents/vendor/cc-sdd

echo "Running cc-sdd dry run for target set: $TARGETS"
npx cc-sdd@latest --dry-run "${ARGS[@]}"

if [ "$DRY_RUN_ONLY" = true ]; then
  echo "dry run complete; no files were changed by this script"
  exit 0
fi

echo
if [ "$INSTALL_MODE" = "staged" ]; then
  echo "Controlled staged install selected. Generated CLAUDE.md will be stored under .agents/vendor/cc-sdd/ and live CLAUDE.md will remain unchanged."
else
  echo "Direct install selected. cc-sdd may write directly to live tool files."
fi

if [ "$YES" != true ]; then
  read -r -p "Apply cc-sdd installation now? [y/N]: " APPLY
  APPLY="${APPLY:-N}"
  if [[ ! "$APPLY" =~ ^[Yy]$ ]]; then
    echo "cancelled after dry run"
    exit 0
  fi
fi

copy_if_exists() {
  local src="$1"
  local dest="$2"
  if [ -e "$src" ]; then
    mkdir -p "$dest"
    cp -a "$src"/. "$dest"/
    echo "copied $src -> $dest"
  fi
}

expose_vendor_claude_skills() {
  local vendor_dir=".agents/vendor/cc-sdd/claude/skills"
  [ -d "$vendor_dir" ] || return 0
  mkdir -p .agents/skills
  find "$vendor_dir" -mindepth 1 -maxdepth 1 -type d | sort | while IFS= read -r skill_dir; do
    local name
    name="$(basename "$skill_dir")"
    local link=".agents/skills/$name"
    local target="../vendor/cc-sdd/claude/skills/$name"
    if [ -e "$link" ] && [ ! -L "$link" ]; then
      echo "skip exposing $name: $link already exists as a real file/directory"
      continue
    fi
    rm -f "$link"
    ln -s "$target" "$link"
    echo "exposed Claude skill: $link -> $target"
  done
}

if [ "$INSTALL_MODE" = "staged" ]; then
  STAGE_DIR="$(mktemp -d)"
  trap 'rm -rf "$STAGE_DIR"' EXIT
  echo "Applying cc-sdd in staging directory: $STAGE_DIR"
  (cd "$STAGE_DIR" && npx cc-sdd@latest "${APPLY_ARGS[@]}")

  rm -rf .agents/vendor/cc-sdd/claude .agents/vendor/cc-sdd/codex .agents/vendor/cc-sdd/gemini
  copy_if_exists "$STAGE_DIR/.claude/skills" ".agents/vendor/cc-sdd/claude/skills"
  copy_if_exists "$STAGE_DIR/.codex/skills" ".agents/vendor/cc-sdd/codex/skills"
  copy_if_exists "$STAGE_DIR/.gemini/skills" ".agents/vendor/cc-sdd/gemini/skills"
  copy_if_exists "$STAGE_DIR/.kiro/settings" ".kiro/settings"

  if [ -f "$STAGE_DIR/CLAUDE.md" ]; then
    cp "$STAGE_DIR/CLAUDE.md" .agents/vendor/cc-sdd/CLAUDE.md
    echo "stored generated memory document: .agents/vendor/cc-sdd/CLAUDE.md"
  fi

  expose_vendor_claude_skills
else
  if [ "$MEMORY_POLICY" != "replace" ]; then
    echo "Direct mode requires --memory-policy=replace so CLAUDE.md changes are explicit."
    exit 1
  fi
  npx cc-sdd@latest "${APPLY_ARGS[@]}"
fi

SDD_TARGETS="$TARGETS" SDD_INSTALLED_AGENTS="$INSTALLED_AGENTS_JSON" SDD_MODE="$INSTALL_MODE" SDD_MEMORY_POLICY="$MEMORY_POLICY" node - <<'NODE'
const fs = require('fs');
const installedAgents = JSON.parse(process.env.SDD_INSTALLED_AGENTS || '[]');
const manifest = {
  enabled: true,
  provider: 'cc-sdd',
  version: 'latest',
  installed_at: new Date().toISOString(),
  selected_target: process.env.SDD_TARGETS || null,
  installed_agents: installedAgents,
  install_mode: process.env.SDD_MODE || 'staged',
  memory_policy: process.env.SDD_MEMORY_POLICY || 'vendor',
  generated_memory_document: process.env.SDD_MODE === 'staged' ? '.agents/vendor/cc-sdd/CLAUDE.md' : 'CLAUDE.md',
  kiro_dir: '.kiro'
};
fs.mkdirSync('.sdd', { recursive: true });
fs.writeFileSync('.sdd/manifest.json', JSON.stringify(manifest, null, 2) + '\n');
NODE

SDD_TARGETS="$TARGETS" SDD_MODE="$INSTALL_MODE" SDD_MEMORY_POLICY="$MEMORY_POLICY" python3 - <<'PY' 2>/dev/null || true
import os
import re
from pathlib import Path

path = Path('workspace.yaml')
if path.exists():
    text = path.read_text()
    text = re.sub(r'^(  enabled: ).*$', r'\1true', text, count=1, flags=re.MULTILINE)
    text = re.sub(r'^(  install_mode: ).*$', rf'\1{os.environ["SDD_MODE"]} # staged | direct', text, count=1, flags=re.MULTILINE)
    text = re.sub(r'^(  memory_policy: ).*$', rf'\1{os.environ["SDD_MEMORY_POLICY"]} # vendor | replace', text, count=1, flags=re.MULTILINE)
    agents = os.environ['SDD_TARGETS']
    if agents == 'all':
        agents_value = '[claude, codex, gemini]'
    else:
        agents_value = f'[{agents}]'
    text = re.sub(r'^(  installed_agents: ).*$', rf'\1{agents_value}', text, count=1, flags=re.MULTILINE)
    path.write_text(text)
PY

./scripts/install-agent-links.sh

echo "cc-sdd installation complete"

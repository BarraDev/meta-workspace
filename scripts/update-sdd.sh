#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "Updating SDD uses the same safe flow as installation."
./scripts/install-sdd.sh

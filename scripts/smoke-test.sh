#!/usr/bin/env bash
set -euo pipefail

BIN="${1:-./target/release/skills-hub-rs}"
ROOT="$(mktemp -d)"
HUB="$ROOT/hub"
CUSTOM="$ROOT/custom-tool/skills"
PROJECT="$ROOT/project"
trap 'rm -rf "$ROOT"' EXIT
mkdir -p "$PROJECT"

echo "[1/9] init"
"$BIN" --home "$HUB" init

echo "[2/9] 47 built-in tools available"
COUNT="$("$BIN" --home "$HUB" tools --json | python3 -c 'import json,sys; print(len(json.load(sys.stdin)))')"
if [[ "$COUNT" -lt 47 ]]; then
  echo "Expected at least 47 tools, got $COUNT" >&2
  exit 1
fi

echo "[3/9] custom tool"
"$BIN" --home "$HUB" custom-tool add local_test \
  --label "Local Test" --global-dir "$CUSTOM" --project-dir ".agent/skills" --mode copy

echo "[4/9] local install + global sync"
"$BIN" --home "$HUB" add-local examples/demo-skill --tag smoke --tool local_test --mode copy
[[ -f "$HUB/skills/demo-skill/SKILL.md" ]]
[[ -f "$CUSTOM/demo-skill/SKILL.md" ]]
[[ -f "$CUSTOM/demo-skill/reference.md" ]]

echo "[5/9] metadata/list"
"$BIN" --home "$HUB" show demo-skill --json >/dev/null
"$BIN" --home "$HUB" list --json >/dev/null

echo "[6/9] disable/enable"
"$BIN" --home "$HUB" disable demo-skill
[[ ! -e "$CUSTOM/demo-skill" ]]
"$BIN" --home "$HUB" enable demo-skill
[[ -f "$CUSTOM/demo-skill/SKILL.md" ]]

echo "[7/9] project sync"
"$BIN" --home "$HUB" sync demo-skill --tool local_test --scope project --project "$PROJECT" --mode copy
[[ -f "$PROJECT/.agent/skills/demo-skill/SKILL.md" ]]

echo "[8/9] recycle + restore"
RID="$("$BIN" --home "$HUB" remove demo-skill | sed -n 's/.*recycle bin: //p')"
[[ -n "$RID" ]]
[[ ! -e "$HUB/skills/demo-skill" ]]
"$BIN" --home "$HUB" recycle restore "$RID"
[[ -f "$HUB/skills/demo-skill/SKILL.md" ]]

echo "[9/9] final state"
"$BIN" --home "$HUB" list

echo "SMOKE TEST OK"

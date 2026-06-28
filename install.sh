#!/usr/bin/env bash
# install.sh — non-interactive installer for prediction-market-settlement-skill
# Copies the skill/ directory into your project and patches CLAUDE.md.
set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/skill"
TARGET_DIR="${1:-.}"

echo "Installing prediction-market-settlement-skill into: $TARGET_DIR"

# Copy skill files
mkdir -p "$TARGET_DIR/skill"
cp -r "$SKILL_DIR/." "$TARGET_DIR/skill/"

# Patch or create CLAUDE.md
CLAUDE_MD="$TARGET_DIR/CLAUDE.md"
SKILL_LINE="skill: ./skill/SKILL.md"

if [ -f "$CLAUDE_MD" ]; then
  if grep -qF "$SKILL_LINE" "$CLAUDE_MD"; then
    echo "CLAUDE.md already references the skill — nothing to patch."
  else
    echo "" >> "$CLAUDE_MD"
    echo "$SKILL_LINE" >> "$CLAUDE_MD"
    echo "Appended skill reference to existing CLAUDE.md."
  fi
else
  echo "$SKILL_LINE" > "$CLAUDE_MD"
  echo "Created CLAUDE.md with skill reference."
fi

echo ""
echo "Done. Claude Code will now load the prediction-market-settlement skill"
echo "when you work on settlement/payout logic in this project."

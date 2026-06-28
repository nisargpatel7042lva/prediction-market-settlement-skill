#!/usr/bin/env bash
# install-custom.sh — interactive installer for prediction-market-settlement-skill
set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/skill"

echo "=== prediction-market-settlement-skill — custom install ==="
echo ""

# Target directory
read -r -p "Install into which project directory? [.] " TARGET_DIR
TARGET_DIR="${TARGET_DIR:-.}"

# Skill subdirectory name
read -r -p "Install skill files into which subdirectory? [skill] " SKILL_SUBDIR
SKILL_SUBDIR="${SKILL_SUBDIR:-skill}"

DEST="$TARGET_DIR/$SKILL_SUBDIR"

echo ""
echo "Available skill files:"
for f in "$SKILL_DIR"/*.md; do
  echo "  $(basename "$f")"
done
echo ""
read -r -p "Install all skill files? [Y/n] " INSTALL_ALL
INSTALL_ALL="${INSTALL_ALL:-Y}"

mkdir -p "$DEST"

if [[ "$INSTALL_ALL" =~ ^[Yy] ]]; then
  cp -r "$SKILL_DIR/." "$DEST/"
  echo "Installed all skill files into $DEST."
else
  echo "Select files to install (space-separated basenames, e.g. 'SKILL.md merkle-outcome-verification.md'):"
  read -r -a FILES
  for f in "${FILES[@]}"; do
    if [ -f "$SKILL_DIR/$f" ]; then
      cp "$SKILL_DIR/$f" "$DEST/$f"
      echo "  Installed $f"
    else
      echo "  WARNING: $f not found, skipping."
    fi
  done
fi

# Agents
if [ -d "$(dirname "${BASH_SOURCE[0]}")/agents" ]; then
  read -r -p "Also install agent definitions (agents/)? [Y/n] " INSTALL_AGENTS
  INSTALL_AGENTS="${INSTALL_AGENTS:-Y}"
  if [[ "$INSTALL_AGENTS" =~ ^[Yy] ]]; then
    mkdir -p "$TARGET_DIR/agents"
    cp -r "$(dirname "${BASH_SOURCE[0]}")/agents/." "$TARGET_DIR/agents/"
    echo "Installed agent definitions."
  fi
fi

# Commands
if [ -d "$(dirname "${BASH_SOURCE[0]}")/commands" ]; then
  read -r -p "Also install slash commands (commands/)? [Y/n] " INSTALL_COMMANDS
  INSTALL_COMMANDS="${INSTALL_COMMANDS:-Y}"
  if [[ "$INSTALL_COMMANDS" =~ ^[Yy] ]]; then
    mkdir -p "$TARGET_DIR/commands"
    cp -r "$(dirname "${BASH_SOURCE[0]}")/commands/." "$TARGET_DIR/commands/"
    echo "Installed slash commands."
  fi
fi

# Patch CLAUDE.md
SKILL_LINE="skill: ./$SKILL_SUBDIR/SKILL.md"
CLAUDE_MD="$TARGET_DIR/CLAUDE.md"

read -r -p "Patch CLAUDE.md to load the skill automatically? [Y/n] " PATCH_CLAUDE
PATCH_CLAUDE="${PATCH_CLAUDE:-Y}"

if [[ "$PATCH_CLAUDE" =~ ^[Yy] ]]; then
  if [ -f "$CLAUDE_MD" ]; then
    if grep -qF "$SKILL_LINE" "$CLAUDE_MD"; then
      echo "CLAUDE.md already references the skill."
    else
      echo "" >> "$CLAUDE_MD"
      echo "$SKILL_LINE" >> "$CLAUDE_MD"
      echo "Appended to existing CLAUDE.md."
    fi
  else
    echo "$SKILL_LINE" > "$CLAUDE_MD"
    echo "Created CLAUDE.md."
  fi
fi

echo ""
echo "Done. Open any Anchor settlement program in this project and Claude will"
echo "automatically apply prediction-market-settlement guidance."

#!/usr/bin/env bash
#
# Claude Code Hook: Block Dangerous Git Commands
# Prevents Claude from executing potentially destructive git operations
#
# Environment variables provided by Claude Code:
# - CLAUDE_PROJECT_DIR: Path to the project directory
# - CLAUDE_TOOL_NAME: Name of the tool that was executed
# - CLAUDE_TOOL_ARGS: JSON string containing tool arguments

set -euo pipefail

# Read hook input from stdin
HOOK_INPUT=""
if [ ! -t 0 ]; then
	HOOK_INPUT=$(cat)
fi

# Extract command from Bash tool arguments
COMMAND=""
if [ -n "$HOOK_INPUT" ]; then
	COMMAND=$(echo "$HOOK_INPUT" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | sed 's/.*"command"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/' | head -1)
fi

# Dangerous git commands that should be blocked
BLOCKED_PATTERNS=(
	"git push --force"
	"git push -f"
	"git push --force-with-lease"
	"git reset --hard HEAD~"
	"git reset --hard origin"
	"git clean -fd"
	"git clean -fx"
	"git checkout -- ."
	"git restore ."
	"git restore"
	"git stash clear"
	"git remote remove"
	"git remote rm"
)

# Check if command matches any blocked pattern
for pattern in "${BLOCKED_PATTERNS[@]}"; do
	if [[ "$COMMAND" == *"$pattern"* ]]; then
		echo "BLOCKED: Dangerous git command detected: $pattern"
		echo "This command requires explicit user approval."
		exit 2 # Exit code 2 blocks the tool call
	fi
done

# Allow the command to proceed
exit 0

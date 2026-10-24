#!/usr/bin/env bash
# Repository policy: commits and pull requests carry no AI provider, AI coding tool or AI co-author
# attribution. Authorship is the human or organisation that commits; tools are not credited.
#
# Usage:
#   check-attribution.sh FILE            check one commit message file (the commit-msg hook)
#   check-attribution.sh --range A..B    check every commit message in a range (CI)
#   check-attribution.sh --stdin         check text on stdin (CI: pull request title and body)
#
# Only attribution forms are matched (trailers, "generated with/by" lines, tool footers, agent author
# addresses), so ordinary words such as the sign Gemini in source code are never affected.
set -euo pipefail

TOOLS='claude|anthropic|openai|chatgpt|gpt-?[0-9o]|codex|copilot|gemini|bard|cursor|windsurf|codeium|devin|aider|tabnine|cody|amazon q|codewhisperer|grok|mistral|llama|deepseek|qwen|minimax|kimi|ai assistant|ai agent|llm'
PATTERNS=(
  "^[[:space:]]*co-authored-by:.*(${TOOLS}|noreply@anthropic\.com|noreply@openai\.com|copilot@github\.com|bot\])"
  "(generated|written|created|authored|assisted|produced|made)[[:space:]]+(with|by|using)[[:space:]]+\[?(${TOOLS})"
  "^[[:space:]]*(🤖|robot)[[:space:]]"
  "(claude\.com/claude-code|claude\.ai/code|chatgpt\.com|openai\.com/codex)"
  "^[[:space:]]*(assisted-by|generated-by|ai-assisted|ai-generated|x-ai-tool|tool):[[:space:]]*.*(${TOOLS})"
)

scan() { # $1 label; text on stdin
  local label=$1 text hit=0 p
  text=$(grep -v '^#' || true)
  for p in "${PATTERNS[@]}"; do
    if grep -Eiq -- "$p" <<<"$text"; then
      grep -Ein -- "$p" <<<"$text" | sed "s/^/  ${label}: line /" >&2
      hit=1
    fi
  done
  return $hit
}

fail() {
  echo "Attribution policy: remove AI provider, AI tool and AI co-author attribution (see CONTRIBUTING.md)." >&2
  exit 1
}

case "${1:-}" in
  --range)
    range=${2:?--range needs A..B}
    status=0
    shas=$(git rev-list --no-merges "$range")  # a bad range fails the check instead of passing it
    for sha in $shas; do
      git log -1 --format='%an <%ae>%n%cn <%ce>%n%B' "$sha" | scan "commit ${sha:0:10}" || status=1
    done
    [ "$status" -eq 0 ] || fail ;;
  --stdin)
    scan "pull request" || fail ;;
  ""|-h|--help)
    sed -n '2,12p' "$0"; exit 64 ;;
  *)
    scan "commit message" < "$1" || fail ;;
esac

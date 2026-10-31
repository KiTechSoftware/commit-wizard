#!/usr/bin/env bash
set -euo pipefail

PRUNE_LOCAL=0

for arg in "$@"; do
  case "$arg" in
    --local) PRUNE_LOCAL=1 ;;
    *)
      echo "Unknown argument: $arg" >&2
      echo "Usage: $0 [--local]" >&2
      exit 1
      ;;
  esac
done

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Error: not inside a Git repository" >&2
  exit 1
fi

CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"

echo "🔄 Fetching and pruning remote branches..."
git fetch --prune

declare -a gone_branches=()
declare -a branches_to_delete=()
declare -a skipped_current=()

while IFS=$'\t' read -r branch upstream_track; do
  if [[ "$upstream_track" == *"[gone]"* ]]; then
    gone_branches+=("$branch")
  fi
done < <(
  git for-each-ref \
    --format='%(refname:short)%09%(upstream:track)' \
    refs/heads
)

if [[ "$PRUNE_LOCAL" -eq 1 ]]; then
  echo "🧹 Evaluating local branches with gone upstreams..."

  for branch in "${gone_branches[@]}"; do
    if [[ "$branch" == "$CURRENT_BRANCH" ]]; then
      skipped_current+=("$branch")
      continue
    fi

    branches_to_delete+=("$branch")
  done

  if [[ ${#skipped_current[@]} -gt 0 ]]; then
    echo "Skipping current branch:"
    printf '  %s\n' "${skipped_current[@]}"
  fi

  if [[ ${#branches_to_delete[@]} -eq 0 ]]; then
    echo "No local branches to delete."
    echo "✅ Done"
    exit 0
  fi

  echo "The following local branches will be force deleted because their upstream no longer exists:"
  printf '  %s\n' "${branches_to_delete[@]}"
  echo
  echo "WARNING: This may remove branches with unmerged local commits."
  echo "This action is intended to make local branch state match remote."

  if [[ ! -t 0 ]]; then
    echo "Error: confirmation required, but stdin is not interactive." >&2
    exit 1
  fi

  read -r -p "Proceed with force deletion? [y/N]: " confirm
  case "$confirm" in
    y|Y|yes|YES)
      for branch in "${branches_to_delete[@]}"; do
        git branch -D -- "$branch"
      done
      ;;
    *)
      echo "Aborted."
      exit 1
      ;;
  esac
fi

echo "✅ Done"
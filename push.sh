#!/bin/bash
# RUSSH - Git push helper
set -e

msg="${1:-update}"
branch="${2:-$(git branch --show-current)}"

echo "📦 Staging changes..."
git add -A

if git diff --cached --quiet; then
    echo "✅ Nothing to commit"
    exit 0
fi

echo "💾 Committing: $msg"
git commit -m "$msg"

echo "🚀 Pushing to $branch..."
git push origin "$branch"

echo "✅ Done!"

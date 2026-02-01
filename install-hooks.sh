#!/bin/bash
# Install pre-commit hooks

echo "📦 Installing pre-commit hooks..."

# Make the hook executable
chmod +x .pre-commit-config.sh

# Create symlink
ln -sf ../../.pre-commit-config.sh .git/hooks/pre-commit

echo "✅ Pre-commit hooks installed successfully!"
echo "💡 The hook will run format + lint + test before each commit"
echo "💡 To bypass: git commit --no-verify"

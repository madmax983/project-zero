#!/bin/bash
# Pre-commit hook: format + lint + test
# Install: ln -s ../../.pre-commit-config.sh .git/hooks/pre-commit

set -e

echo "🔍 Running pre-commit checks..."

echo "📝 Checking formatting..."
cargo fmt --all -- --check || {
    echo "❌ Format check failed. Run 'cargo fmt' to fix."
    exit 1
}

echo "🔎 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery || {
    echo "❌ Clippy found issues. Fix them before committing."
    exit 1
}

echo "🧪 Running tests..."
cargo test --all-features || {
    echo "❌ Tests failed. Fix them before committing."
    exit 1
}

echo "✅ All pre-commit checks passed!"

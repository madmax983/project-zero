# Pre-commit hook for Windows: format + lint + test
# Install: Copy to .git\hooks\pre-commit (remove .ps1 extension) or run via Git Bash

Write-Host "🔍 Running pre-commit checks..." -ForegroundColor Cyan

Write-Host "📝 Checking formatting..." -ForegroundColor Yellow
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Format check failed. Run 'cargo fmt' to fix." -ForegroundColor Red
    exit 1
}

Write-Host "🔎 Running clippy..." -ForegroundColor Yellow
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Clippy found issues. Fix them before committing." -ForegroundColor Red
    exit 1
}

Write-Host "🧪 Running tests..." -ForegroundColor Yellow
cargo test --all-features
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Tests failed. Fix them before committing." -ForegroundColor Red
    exit 1
}

Write-Host "✅ All pre-commit checks passed!" -ForegroundColor Green

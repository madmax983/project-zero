# CI/CD Setup

This project follows Mark's Global Coding Standards for quality gates and continuous integration.

## Quality Standards

- **85-90% test coverage minimum**
- **TDD (RED-GREEN-REFACTOR)**: Tests before implementation
- **Strict linting**: Clippy pedantic + nursery warnings enabled
- **Format enforcement**: `cargo fmt` required before commits

## GitHub Actions Workflows

### CI Pipeline (`.github/workflows/ci.yml`)

Runs on every push and PR to `trunk`:

1. **Format Check**: `cargo fmt --check`
2. **Clippy Lints**: Pedantic + Nursery warnings as errors
3. **Tests**: Full test suite with all features
4. **Coverage**: `cargo-tarpaulin` with 85% minimum threshold

### Benchmarks (`.github/workflows/benchmark.yml`)

Runs criterion benchmarks on PRs to track performance regressions.

## Local Pre-Commit Hooks

Install locally to run checks before each commit:

### Windows (PowerShell)
```powershell
.\install-hooks.ps1
```

### Linux/Mac (Bash)
```bash
chmod +x install-hooks.sh
./install-hooks.sh
```

### Manual Installation
The hooks run:
1. Format check
2. Clippy (pedantic + nursery)
3. Full test suite

To bypass (use sparingly): `git commit --no-verify`

## Cargo Aliases

Configured in `.cargo/config.toml`:

```bash
cargo fmt-check      # Check formatting without modifying
cargo lint          # Run clippy with pedantic + nursery
cargo precommit     # Run all pre-commit checks manually
```

## Coverage Reports

- **Local**: `cargo tarpaulin --all-features --workspace --out html`
- **CI**: Uploaded to Codecov (requires `CODECOV_TOKEN` secret)
- **Threshold**: Fails if coverage drops below 85%

## Performance Benchmarks

```bash
cargo criterion          # Run all benchmarks
cargo criterion --bench benchmarks  # Specific benchmark
```

Results saved in `target/criterion/` with HTML reports.

## Required GitHub Secrets

For full CI functionality, configure these in your repository settings:

- `CODECOV_TOKEN`: Token for uploading coverage reports to Codecov.io

## CI Badge

Add to README.md:

```markdown
[![CI](https://github.com/madmax983/project-zero/workflows/CI/badge.svg)](https://github.com/madmax983/project-zero/actions)
```

# Install pre-commit hooks for Windows

Write-Host "📦 Installing pre-commit hooks..." -ForegroundColor Cyan

# Copy the PowerShell script to .git\hooks\pre-commit
Copy-Item -Path ".pre-commit-config-windows.ps1" -Destination ".git\hooks\pre-commit.ps1" -Force

# Create a wrapper bash script for Git Bash compatibility
$bashWrapper = @"
#!/bin/bash
powershell.exe -ExecutionPolicy Bypass -File .git/hooks/pre-commit.ps1
"@

Set-Content -Path ".git\hooks\pre-commit" -Value $bashWrapper -NoNewline

Write-Host "✅ Pre-commit hooks installed successfully!" -ForegroundColor Green
Write-Host "💡 The hook will run format + lint + test before each commit" -ForegroundColor Yellow
Write-Host "💡 To bypass: git commit --no-verify" -ForegroundColor Yellow

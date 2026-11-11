param([switch]$Full)

# Single source of truth cho check local va CI.
#   pwsh scripts/check.ps1          # fast, < 15s: lint + build examples
#   pwsh scripts/check.ps1 -Full    # fast + toan bo test
#
# CI goi dung script nay de local va CI khong drift.

$ErrorActionPreference = "Stop"

function Invoke-Step {
    param([string]$Label, [string]$Command)

    Write-Host "-> $Label" -ForegroundColor Cyan
    Invoke-Expression $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $Label (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

# Fast: chay moi lan push.
Invoke-Step "ruff check ." "ruff check ."
Invoke-Step "cargo build --workspace --examples" "cargo build --workspace --examples"
Invoke-Step "cargo build --workspace --features context-lm --examples" "cargo build --workspace --features context-lm --examples"

if (-not $Full) {
    Write-Host "OK: fast checks passed. Run with -Full for tests." -ForegroundColor Green
    exit 0
}

# Full: truoc PR / release.
$pytest = if (Test-Path ".venv/Scripts/python.exe") {
    ".venv/Scripts/python.exe -m pytest"
} elseif (Test-Path ".venv/bin/python") {
    ".venv/bin/python -m pytest"
} else {
    "pytest"
}

Invoke-Step "cargo test --workspace --all-targets" "cargo test --workspace --all-targets"
Invoke-Step "pytest -q" "$pytest -q"

Write-Host "OK: all checks passed." -ForegroundColor Green

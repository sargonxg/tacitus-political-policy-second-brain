param(
    [Parameter(Position=0)][ValidateSet('doctor','test','setup','up','demo','smoke','benchmark','down')][string]$Command = 'doctor',
    [Parameter(ValueFromRemainingArguments=$true)][string[]]$ExtraArgs
)
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
switch ($Command) {
    'doctor' {
        $missing = @()
        foreach ($name in @('git','docker')) {
            if (Get-Command $name -ErrorAction SilentlyContinue) { Write-Output "$name command present" }
            else { $missing += $name; Write-Output "$name command MISSING" }
        }
        if ($missing -contains 'docker') { Write-Output 'BLOCKED: install/start a compatible Linux-container runtime. No inference test ran.'; exit 2 }
        & docker version
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        & docker compose version
        exit $LASTEXITCODE
    }
    'test' {
        if (!(Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Development tests require Rust 1.95; product runtime is not implemented.' }
        # Isolate the new libraries from unrelated upstream resolver/build requirements.
        # No model or document directories are copied. No network: cached crates only.
        $testRoot = Join-Path $repoRoot '.tacitus/library-tests'
        New-Item -ItemType Directory -Force -Path $testRoot | Out-Null
        foreach ($crate in @('tacitus-core','tacitus-capsules','tacitus-analysis','tacitus-evals')) {
            Copy-Item -LiteralPath (Join-Path $repoRoot "edgequake/crates/$crate") -Destination $testRoot -Recurse -Force
        }
        $workspace = '[workspace]' + "`n" + 'resolver = "2"' + "`n" + 'members = ["tacitus-core", "tacitus-capsules", "tacitus-analysis", "tacitus-evals"]' + "`n"
        [IO.File]::WriteAllText((Join-Path $testRoot 'Cargo.toml'), $workspace)
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'tacitus-library.Cargo.lock') -Destination (Join-Path $testRoot 'Cargo.lock') -Force
        & cargo test --offline --locked --manifest-path (Join-Path $testRoot 'Cargo.toml') @ExtraArgs
        exit $LASTEXITCODE
    }
    default {
        Write-Error "$Command is NOT IMPLEMENTED. Milestone 1 live baseline is blocked; see IMPLEMENTATION_STATUS.md. No services or volumes were changed."
        exit 2
    }
}

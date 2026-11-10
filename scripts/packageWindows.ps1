param(
  [switch]$SkipInstall
)

$ErrorActionPreference = "Stop"
$buildArgs = @("tauri", "build", "--bundles", "nsis")

if ([string]::IsNullOrWhiteSpace($env:TAURI_SIGNING_PRIVATE_KEY)) {
  $buildArgs += "--no-sign"
}

& pnpm @buildArgs
if ($LASTEXITCODE -ne 0) {
  throw "Windows package build failed with exit code $LASTEXITCODE."
}

if ($SkipInstall) {
  return
}

$installScript = Join-Path $PSScriptRoot "installWindows.ps1"
& $installScript

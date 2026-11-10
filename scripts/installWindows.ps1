param(
  [string]$InstallerPath = ""
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$installDir = Join-Path $env:ProgramFiles "EcoPaste"
$installedExe = Join-Path $installDir "EcoPaste.exe"

if ([string]::IsNullOrWhiteSpace($InstallerPath)) {
  $bundleDir = Join-Path $repoRoot "src-tauri\target\release\bundle\nsis"
  $latestInstaller = Get-ChildItem -LiteralPath $bundleDir -Filter "EcoPaste_*-setup.exe" -File |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

  if ($null -eq $latestInstaller) {
    throw "No NSIS installer found in $bundleDir."
  }

  $InstallerPath = $latestInstaller.FullName
}

$resolvedInstaller = (Resolve-Path -LiteralPath $InstallerPath).Path
Write-Host "Installing $resolvedInstaller"

$installerProcess = Start-Process `
  -FilePath $resolvedInstaller `
  -ArgumentList @("/P", "/UPDATE", "/R") `
  -PassThru

if (-not $installerProcess.WaitForExit(180000)) {
  throw "EcoPaste installer timed out after 180 seconds."
}

if ($installerProcess.ExitCode -ne 0) {
  throw "EcoPaste installer exited with code $($installerProcess.ExitCode)."
}

$deadline = (Get-Date).AddSeconds(30)
do {
  Start-Sleep -Milliseconds 250
  $mainProcesses = @(
    Get-Process -Name "EcoPaste" -ErrorAction SilentlyContinue |
      Where-Object { $_.Path -eq $installedExe }
  )
} while ($mainProcesses.Count -eq 0 -and (Get-Date) -lt $deadline)

if ($mainProcesses.Count -eq 0) {
  throw "EcoPaste was installed but no main process started within 30 seconds."
}

if ($mainProcesses.Count -ne 1) {
  throw "Expected one EcoPaste main process, found $($mainProcesses.Count)."
}

$mainProcess = $mainProcesses[0]
Write-Host "EcoPaste started successfully (PID $($mainProcess.Id), path $installedExe)."

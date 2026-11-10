param(
  [string]$ReleaseBaseUrl = "https://github.com/FeatherBirth/EcoPaste/releases/latest/download"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if (-not [Environment]::Is64BitOperatingSystem) {
  throw "EcoPaste requires 64-bit Windows."
}

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$ProgressPreference = "SilentlyContinue"

$programFiles = if ([string]::IsNullOrWhiteSpace($env:ProgramW6432)) {
  $env:ProgramFiles
} else {
  $env:ProgramW6432
}
if ([string]::IsNullOrWhiteSpace($programFiles)) {
  throw "Windows Program Files directory could not be resolved."
}

$installDir = Join-Path $programFiles "EcoPaste"
$installedExe = Join-Path $installDir "EcoPaste.exe"
$releaseBaseUrl = $ReleaseBaseUrl.TrimEnd("/")
$releaseUri = [Uri]$releaseBaseUrl
if ($releaseUri.Scheme -ne "https" -and -not ($releaseUri.Scheme -eq "http" -and $releaseUri.IsLoopback)) {
  throw "The install source must use HTTPS or a local loopback URL."
}

$tempDir = Join-Path ([IO.Path]::GetTempPath()) ("ecopaste-install-" + [Guid]::NewGuid().ToString("N"))
$installerPath = Join-Path $tempDir "EcoPaste_x64-setup.exe"
$checksumPath = Join-Path $tempDir "EcoPaste_x64-setup.exe.sha256"

New-Item -ItemType Directory -Path $tempDir | Out-Null

try {
  Write-Host "Downloading EcoPaste from $releaseBaseUrl"
  Invoke-WebRequest -UseBasicParsing `
    -Uri "$releaseBaseUrl/EcoPaste_x64-setup.exe.sha256" `
    -OutFile $checksumPath
  Invoke-WebRequest -UseBasicParsing `
    -Uri "$releaseBaseUrl/EcoPaste_x64-setup.exe" `
    -OutFile $installerPath

  $checksumText = Get-Content -LiteralPath $checksumPath -Raw
  if ($checksumText -notmatch "^\s*([0-9a-fA-F]{64})\s+EcoPaste_x64-setup\.exe\s*$") {
    throw "The release checksum file has an invalid format."
  }

  $expectedHash = $Matches[1].ToUpperInvariant()
  $actualHash = (Get-FileHash -LiteralPath $installerPath -Algorithm SHA256).Hash.ToUpperInvariant()
  if ($actualHash -ne $expectedHash) {
    throw "The downloaded installer checksum does not match the published checksum."
  }

  $installer = Get-Item -LiteralPath $installerPath
  $expectedVersion = $installer.VersionInfo.ProductVersion.Trim()
  if ([string]::IsNullOrWhiteSpace($expectedVersion)) {
    throw "The downloaded installer does not contain a valid product version."
  }

  Write-Host "Installing EcoPaste $expectedVersion"
  $installerProcess = Start-Process `
    -FilePath $installerPath `
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
        Where-Object {
          try {
            $_.Path -eq $installedExe
          } catch {
            $false
          }
        }
    )
  } while ($mainProcesses.Count -eq 0 -and (Get-Date) -lt $deadline)

  if ($mainProcesses.Count -eq 0) {
    throw "EcoPaste was installed but no main process started within 30 seconds."
  }
  if ($mainProcesses.Count -ne 1) {
    throw "Expected one EcoPaste main process, found $($mainProcesses.Count)."
  }

  $installedVersion = (Get-Item -LiteralPath $installedExe).VersionInfo.ProductVersion.Trim()
  if ($installedVersion -ne $expectedVersion) {
    throw "Installed version $installedVersion does not match package version $expectedVersion."
  }

  Write-Host "EcoPaste $installedVersion installed successfully (PID $($mainProcesses[0].Id))."
} finally {
  if (Test-Path -LiteralPath $tempDir) {
    Remove-Item -LiteralPath $tempDir -Recurse -Force
  }
}

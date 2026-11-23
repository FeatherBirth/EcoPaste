param(
  [string]$IconPath = "src-tauri/icons/icon.png",
  [string]$OutputDirectory = "src-tauri/nsis"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$repoRoot = Split-Path -Parent $PSScriptRoot
$iconFullPath = Join-Path $repoRoot $IconPath
$outputFullPath = Join-Path $repoRoot $OutputDirectory
New-Item -ItemType Directory -Path $outputFullPath -Force | Out-Null

function New-BrandBitmap {
  param(
    [int]$Width,
    [int]$Height
  )

  $bitmap = New-Object System.Drawing.Bitmap($Width, $Height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
  $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit

  $rect = New-Object System.Drawing.Rectangle(0, 0, $Width, $Height)
  $background = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    $rect,
    [System.Drawing.Color]::FromArgb(19, 96, 184),
    [System.Drawing.Color]::FromArgb(17, 161, 238),
    25
  )
  $graphics.FillRectangle($background, $rect)
  $background.Dispose()

  return @{
    Bitmap = $bitmap
    Graphics = $graphics
  }
}

function Add-BrandText {
  param(
    [System.Drawing.Graphics]$Graphics,
    [string]$Text,
    [float]$X,
    [float]$Y,
    [float]$Size,
    [bool]$Secondary = $false
  )

  $font = New-Object System.Drawing.Font(
    "Segoe UI Semibold",
    $Size,
    [System.Drawing.FontStyle]::Regular,
    [System.Drawing.GraphicsUnit]::Pixel
  )
  $color = if ($Secondary) {
    [System.Drawing.Color]::FromArgb(205, 238, 252, 255)
  } else {
    [System.Drawing.Color]::White
  }
  $brush = New-Object System.Drawing.SolidBrush($color)
  $Graphics.DrawString($Text, $font, $brush, [System.Drawing.PointF]::new($X, $Y))
  $brush.Dispose()
  $font.Dispose()
}

$icon = [System.Drawing.Image]::FromFile($iconFullPath)

$header = New-BrandBitmap -Width 150 -Height 57
$header.Graphics.DrawImage($icon, 8, 7, 44, 44)
Add-BrandText -Graphics $header.Graphics -Text "EcoPaste" -X 59 -Y 9 -Size 17
Add-BrandText -Graphics $header.Graphics -Text "Clipboard manager" -X 60 -Y 31 -Size 9 -Secondary $true
$header.Bitmap.Save((Join-Path $outputFullPath "header.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$header.Graphics.Dispose()
$header.Bitmap.Dispose()

$sidebar = New-BrandBitmap -Width 164 -Height 314
$sidebar.Graphics.DrawImage($icon, 30, 42, 104, 104)
Add-BrandText -Graphics $sidebar.Graphics -Text "EcoPaste" -X 31 -Y 170 -Size 22
Add-BrandText -Graphics $sidebar.Graphics -Text "Local-first clipboard" -X 32 -Y 199 -Size 10 -Secondary $true
$divider = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(70, 255, 255, 255), 1)
$sidebar.Graphics.DrawLine($divider, 32, 228, 132, 228)
$divider.Dispose()
Add-BrandText -Graphics $sidebar.Graphics -Text "Fast. Private. Native." -X 32 -Y 243 -Size 10 -Secondary $true
$sidebar.Bitmap.Save((Join-Path $outputFullPath "sidebar.bmp"), [System.Drawing.Imaging.ImageFormat]::Bmp)
$sidebar.Graphics.Dispose()
$sidebar.Bitmap.Dispose()

$icon.Dispose()

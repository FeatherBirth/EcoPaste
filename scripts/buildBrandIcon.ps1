param(
  [string]$SourcePath = "src-tauri/assets/logo-source.png",
  [string]$OutputPath = "src-tauri/assets/logo.png"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$repoRoot = Split-Path -Parent $PSScriptRoot
$sourceFullPath = Join-Path $repoRoot $SourcePath
$outputFullPath = Join-Path $repoRoot $OutputPath
$source = [System.Drawing.Bitmap]::FromFile($sourceFullPath)

$size = 1024
$output = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$graphics = [System.Drawing.Graphics]::FromImage($output)
$graphics.Clear([System.Drawing.Color]::Transparent)
$graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
$graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

$bounds = New-Object System.Drawing.RectangleF(64, 64, 896, 896)
$radius = 222
$diameter = $radius * 2
$path = New-Object System.Drawing.Drawing2D.GraphicsPath
$path.AddArc($bounds.Left, $bounds.Top, $diameter, $diameter, 180, 90)
$path.AddArc($bounds.Right - $diameter, $bounds.Top, $diameter, $diameter, 270, 90)
$path.AddArc($bounds.Right - $diameter, $bounds.Bottom - $diameter, $diameter, $diameter, 0, 90)
$path.AddArc($bounds.Left, $bounds.Bottom - $diameter, $diameter, $diameter, 90, 90)
$path.CloseFigure()

$background = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
  $bounds,
  [System.Drawing.Color]::FromArgb(91, 201, 255),
  [System.Drawing.Color]::FromArgb(24, 121, 218),
  42
)
$graphics.FillPath($background, $path)
$background.Dispose()

$highlight = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(22, 255, 255, 255))
$graphics.FillEllipse($highlight, 112, 70, 720, 500)
$highlight.Dispose()

$border = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(38, 255, 255, 255), 4)
$graphics.DrawPath($border, $path)
$border.Dispose()
$path.Dispose()

$mark = New-Object System.Drawing.Bitmap(
  $source.Width,
  $source.Height,
  [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
)
for ($y = 0; $y -lt $source.Height; $y++) {
  for ($x = 0; $x -lt $source.Width; $x++) {
    $pixel = $source.GetPixel($x, $y)
    $minimum = [Math]::Min($pixel.R, [Math]::Min($pixel.G, $pixel.B))
    $alphaValue = ($minimum - 145) * 255 / 110
    $alpha = [int][Math]::Max(0, [Math]::Min(255, $alphaValue))
    if ($alpha -gt 0 -and $pixel.A -gt 0) {
      $mark.SetPixel($x, $y, [System.Drawing.Color]::FromArgb($alpha, 255, 255, 255))
    }
  }
}

$graphics.DrawImage($mark, 64, 64, 896, 896)
$mark.Dispose()
$graphics.Dispose()
$output.Save($outputFullPath, [System.Drawing.Imaging.ImageFormat]::Png)
$output.Dispose()
$source.Dispose()

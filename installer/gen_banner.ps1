Add-Type -AssemblyName System.Drawing

$banner = "D:\Mis Juegos\HyperChomik\chomik-hamster\banner.jpeg"
$out = "D:\Mis Juegos\HyperChomik\chomik-hamster\installer"

$img = [System.Drawing.Image]::FromFile($banner)

function Make-Banner {
  param([int]$w, [int]$h, [string]$path, [float]$fontSize)
  $bmp = New-Object System.Drawing.Bitmap($w, $h)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.DrawImage($img, 0, 0, $w, $h)
  $font = New-Object System.Drawing.Font("Arial Black", $fontSize, [System.Drawing.FontStyle]::Bold)
  $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
  $fmt = New-Object System.Drawing.StringFormat
  $fmt.Alignment = [System.Drawing.StringAlignment]::Center
  $fmt.LineAlignment = [System.Drawing.StringAlignment]::Center
  $rx = [float]($w * 0.45)
  $ry = [float]($h * 0.50)
  $rw = [float]($w * 0.50)
  $rh = [float]($h * 0.45)
  $rect = New-Object System.Drawing.RectangleF($rx, $ry, $rw, $rh)
  $g.DrawString("HiperChomik", $font, $brush, $rect, $fmt)
  $font.Dispose()
  $brush.Dispose()
  $fmt.Dispose()
  $g.Dispose()
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Bmp)
  $bmp.Dispose()
}

Make-Banner 493 58 "$out\banner_top.bmp" 20
Make-Banner 493 312 "$out\banner_bg.bmp" 36
$img.Dispose()
Write-Host "Banners regenerated with white text"

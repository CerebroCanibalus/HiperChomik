Add-Type -AssemblyName System.Drawing

$banner = "D:\Mis Juegos\HyperChomik\chomik-hamster\banner.jpeg"
$out = "D:\Mis Juegos\HyperChomik\chomik-hamster\installer"

$img = [System.Drawing.Image]::FromFile($banner)

# Top banner 493x58
$top = New-Object System.Drawing.Bitmap(493, 58)
$g = [System.Drawing.Graphics]::FromImage($top)
$g.DrawImage($img, 0, 0, 493, 58)
$g.Dispose()
$top.Save("$out\banner_top.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$top.Dispose()
Write-Host "Top banner: $((Get-Item "$out\banner_top.bmp").Length / 1KB) KB"

# Dialog bg 493x312
$bg = New-Object System.Drawing.Bitmap(493, 312)
$g = [System.Drawing.Graphics]::FromImage($bg)
$g.DrawImage($img, 0, 0, 493, 312)
$g.Dispose()
$bg.Save("$out\banner_bg.bmp", [System.Drawing.Imaging.ImageFormat]::Bmp)
$bg.Dispose()
Write-Host "BG banner: $((Get-Item "$out\banner_bg.bmp").Length / 1KB) KB"

$img.Dispose()

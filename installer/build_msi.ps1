$wixBin = "C:\Program Files (x86)\WiX Toolset v3.14\bin"
$root = "D:\Mis Juegos\HyperChomik\chomik-hamster"
$out = "$root\installer"

$env:Path += ";$wixBin"
Set-Location $out

# Generate WXS files and banners
& "$out\gen_banner.ps1"
& "$out\gen_mainwxs.ps1"
& "$out\gen_sprites.ps1"

# Compile
& candle -nologo -arch x64 "$out\HiperChomik.wxs" -out "$out/" 2>&1
if ($LASTEXITCODE -ne 0) { exit 1 }

& candle -nologo -arch x64 "$out\sprites.wxs" -out "$out/" 2>&1
if ($LASTEXITCODE -ne 0) { exit 1 }

# Link
& light -nologo -cultures:null -sw1076 -loc "$PSScriptRoot\HiperChomik.wxl" -ext WixUIExtension "$out\HiperChomik.wixobj" "$out\sprites.wixobj" -out "$out\HiperChomik.msi" 2>&1
if ($LASTEXITCODE -eq 0) {
  # Sign MSI
  $signtool = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe" | Select-Object -Last 1 -Expand FullName
  & $signtool sign /fd SHA256 /a /tr "http://timestamp.digicert.com" /td SHA256 "$out\HiperChomik.msi" 2>&1 | Out-Null
  # Sign exe
  $exe = "$root\target\x86_64-pc-windows-gnu\release\chomik-hamster.exe"
  if (Test-Path $exe) { & $signtool sign /fd SHA256 /a /tr "http://timestamp.digicert.com" /td SHA256 $exe 2>&1 | Out-Null }
  Write-Host "Signed MSI: $out\HiperChomik.msi" -ForegroundColor Green
  Get-Item "$out\HiperChomik.msi" | Select-Object Length
}

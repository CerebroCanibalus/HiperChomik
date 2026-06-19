$wixBin = "C:\Program Files (x86)\WiX Toolset v3.14\bin"
$root = "D:\Mis Juegos\HyperChomik\chomik-hamster"
$out = "$root\installer"

$env:Path += ";$wixBin"
Set-Location $out

# Generate WXS files
& "$out\gen_mainwxs.ps1"
& "$out\gen_sprites.ps1"

# Compile
& candle -nologo -arch x64 "$out\HiperChomik.wxs" -out "$out/" 2>&1
if ($LASTEXITCODE -ne 0) { exit 1 }

& candle -nologo -arch x64 "$out\sprites.wxs" -out "$out/" 2>&1
if ($LASTEXITCODE -ne 0) { exit 1 }

# Link
& light -nologo -cultures:null -sw1076 -ext WixUIExtension "$out\HiperChomik.wixobj" "$out\sprites.wixobj" -out "$out\HiperChomik.msi" 2>&1
if ($LASTEXITCODE -eq 0) {
  Write-Host "MSI created: $out\HiperChomik.msi" -ForegroundColor Green
  Get-Item "$out\HiperChomik.msi" | Select-Object Length
}

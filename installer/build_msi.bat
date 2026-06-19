$wixBin = "C:\Program Files (x86)\WiX Toolset v3.14\bin"
$root = "D:\Mis Juegos\HyperChomik\chomik-hamster"
$out = "$root\installer"

$env:Path += ";$wixBin"

# Harvest sprites directory
heat dir "$root\sprites" -o "$out\sprites.wxs" -srd -gg -cg SpritesComponent -dr INSTALLDIR -var var.SpriteDir -sfrag -nologo 2>&1

candle "$root\installer\HiperChomik.wxs" "$out\sprites.wxs" -out "$out\" -nologo -arch x64 -dSpriteDir="$root\sprites" 2>&1
light "$out\HiperChomik.wixobj" "$out\sprites.wixobj" -out "$out\HiperChomik.msi" -nologo -cultures:null -sw1076 2>&1

$root = "D:\Mis Juegos\HyperChomik\chomik-hamster"
$out = "$root\installer"

$sprites = Get-ChildItem "$root\sprites\*" -Include "*.png","*.ico","*.xcf"
$componentId = 0
$comps = ""
$componentRefs = ""

foreach ($s in $sprites) {
  $componentId++
  $g = [guid]::NewGuid()
  $name = $s.Name
  $fid = "SF_$([guid]::NewGuid().ToString('N').Substring(0,8))"
  $comps += "      <Component Id=`"S$componentId`" Guid=`"$g`"><File Id=`"$fid`" Name=`"$name`" Source=`"$root\sprites\$name`" /><RegistryValue Root=`"HKCU`" Key=`"Software\HiperChomik\Sprites`" Name=`"S$componentId`" Type=`"integer`" Value=`"1`" KeyPath=`"yes`" /></Component>`n"
  $componentRefs += "        <ComponentRef Id=`"S$componentId`" />`n"
}

$xml = @"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Fragment>
    <DirectoryRef Id="SpritesDir">
$comps    </DirectoryRef>
    <ComponentGroup Id="SpritesComponentGroup">
$componentRefs    </ComponentGroup>
  </Fragment>
</Wix>
"@

Set-Content "$out\sprites.wxs" $xml
Write-Host "Generated $componentId sprite components"

$root = "D:\Mis Juegos\HyperChomik\chomik-hamster"

$wxs = @"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="HiperChomik" Language="1033" Version="1.0.0.0" Manufacturer="CerebroCanibalus" UpgradeCode="F1A2B3C4-D5E6-7890-ABCD-EF1234567890">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perUser" />
    <Media Id="1" Cabinet="HiperChomik.cab" EmbedCab="yes" />

    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="LocalAppDataFolder">
        <Directory Id="INSTALLDIR" Name="HiperChomik">
          <Component Id="MainExe" Guid="A1B2C3D4-E5F6-7890-ABCD-EF1234567890">
            <File Id="ExeFile" Name="chomik-hamster.exe" Source="$root\target\x86_64-pc-windows-gnu\release\chomik-hamster.exe" />
            <RegistryValue Root="HKCU" Key="Software\HiperChomik" Name="Installed" Type="integer" Value="1" KeyPath="yes" />
          </Component>
          <Component Id="AnimsTxt" Guid="B2C3D4E5-F6A7-8901-BCDE-F12345678901">
            <File Id="AnimsFile" Name="anims.txt" Source="$root\anims.txt" />
            <RegistryValue Root="HKCU" Key="Software\HiperChomik" Name="AnimsInstalled" Type="integer" Value="1" KeyPath="yes" />
          </Component>
          <Component Id="QuotesTxt" Guid="C3D4E5F6-A7B8-9012-CDEF-123456789012">
            <File Id="QuotesFile" Name="quotes.txt" Source="$root\quotes.txt" />
            <RegistryValue Root="HKCU" Key="Software\HiperChomik" Name="QuotesInstalled" Type="integer" Value="1" KeyPath="yes" />
          </Component>
          <Directory Id="SpritesDir" Name="sprites">
            <Component Id="SpritesDirComp" Guid="D4E5F6A7-B8C9-0123-DEF1-234567890123">
              <CreateFolder />
              <RemoveFile Id="CleanSprites" Name="*.*" On="uninstall" />
              <RemoveFolder Id="RemoveSpritesDir" On="uninstall" />
              <RegistryValue Root="HKCU" Key="Software\HiperChomik" Name="SpritesDirInstalled" Type="integer" Value="1" KeyPath="yes" />
            </Component>
          </Directory>
        </Directory>
      </Directory>
    </Directory>

    <DirectoryRef Id="INSTALLDIR">
      <Component Id="CleanInstallDir" Guid="E5F6A7B8-C9D0-1234-EFAB-345678901234">
        <RemoveFolder Id="RemoveInstallDir" On="uninstall" />
        <RegistryValue Root="HKCU" Key="Software\HiperChomik" Name="InstallDirClean" Type="integer" Value="1" KeyPath="yes" />
      </Component>
      <Component Id="AutoStart" Guid="F6A7B8C9-D0E1-2345-ABCD-456789012345">
        <RegistryValue Root="HKCU" Key="Software\Microsoft\Windows\CurrentVersion\Run" Name="HiperChomik" Type="string" Value="[INSTALLDIR]chomik-hamster.exe" KeyPath="yes" />
      </Component>
    </DirectoryRef>

    <Feature Id="ProductFeature" Title="HiperChomik" Level="1">
      <ComponentRef Id="MainExe" />
      <ComponentRef Id="AnimsTxt" />
      <ComponentRef Id="QuotesTxt" />
      <ComponentRef Id="SpritesDirComp" />
      <ComponentRef Id="AutoStart" />
      <ComponentRef Id="CleanInstallDir" />
      <ComponentGroupRef Id="SpritesComponentGroup" />
    </Feature>

    <Icon Id="AppIcon" SourceFile="$root\sprites\chomik_icon.ico" />
    <Property Id="ARPPRODUCTICON" Value="AppIcon" />
    <Property Id="ARPHELPLINK" Value="https://github.com/CerebroCanibalus/HiperChomik" />
    <Property Id="ARPNOREPAIR" Value="1" />
    <Property Id="ARPNOMODIFY" Value="1" />

    <CustomAction Id="LaunchApp" Directory="INSTALLDIR" ExeCommand="[INSTALLDIR]chomik-hamster.exe" Return="asyncNoWait" />
    <InstallExecuteSequence>
      <Custom Action="LaunchApp" After="InstallFinalize">NOT Installed</Custom>
    </InstallExecuteSequence>
  </Product>
</Wix>
"@

Set-Content "$root\installer\HiperChomik.wxs" $wxs

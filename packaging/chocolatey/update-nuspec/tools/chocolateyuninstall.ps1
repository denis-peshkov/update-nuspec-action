$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
Uninstall-ChocolateyZipPackage -PackageName $env:ChocolateyPackageName -ZipFileName "$toolsDir\update-nuspec.exe"

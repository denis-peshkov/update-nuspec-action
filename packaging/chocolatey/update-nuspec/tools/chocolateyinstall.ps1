$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$packageArgs = @{
  packageName    = $env:ChocolateyPackageName
  unzipLocation  = $toolsDir
  url64bit       = 'https://github.com/denis-peshkov/update-nuspec-action/releases/download/v0.0.0/update-nuspec-0.0.0-x86_64-pc-windows-msvc.zip'
  checksum64     = '0000000000000000000000000000000000000000000000000000000000000000'
  checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
Install-BinFile -Path "$toolsDir\update-nuspec.exe" -Name 'update-nuspec'

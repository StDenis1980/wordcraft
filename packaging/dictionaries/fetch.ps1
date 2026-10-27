<#
.SYNOPSIS
  Put the Tinta Office spelling dictionaries (sources.tsv) into a folder, checking each file's SHA-256.

.DESCRIPTION
  Downloads are kept in $env:TINTA_DICTIONARIES_CACHE (default: target\dictionaries-cache), so a
  second run needs no network. Used by packaging/windows/package.ps1; Linux and macOS use fetch.sh.

.EXAMPLE
  pwsh packaging/dictionaries/fetch.ps1 -Dest target\windows-package\x64\dictionaries
#>
param([Parameter(Mandatory)] [string] $Dest)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$Cache = if ($env:TINTA_DICTIONARIES_CACHE) { $env:TINTA_DICTIONARIES_CACHE } else {
  Join-Path $(if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }) 'dictionaries-cache'
}
New-Item -ItemType Directory -Force -Path (Join-Path $Dest 'licences'), $Cache | Out-Null
function Get-Sha256([string] $Path) { (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant() }
foreach ($line in Get-Content -Encoding UTF8 (Join-Path $PSScriptRoot 'sources.tsv')) {
  if (-not $line -or $line.StartsWith('#')) { continue }
  $file, $url, $sha = $line.Split("`t")[0..2]
  $cached = Join-Path $Cache $sha
  if (-not (Test-Path -LiteralPath $cached) -or (Get-Sha256 $cached) -ne $sha) {
    Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile "$cached.download"
    if ((Get-Sha256 "$cached.download") -ne $sha) {
      Remove-Item -Force "$cached.download"
      throw "$url does not match its SHA-256 in sources.tsv"
    }
    Move-Item -Force "$cached.download" $cached
  }
  $target = Join-Path $Dest ($file -replace '/', '\')
  New-Item -ItemType Directory -Force -Path (Split-Path $target) | Out-Null
  Copy-Item -Force -LiteralPath $cached -Destination $target
}
Copy-Item -Force (Join-Path $PSScriptRoot 'README.md') (Join-Path $Dest 'licences\README.md')
Write-Output "dictionaries: $((Get-ChildItem -Path $Dest -Filter *.dic).Count) in $Dest"

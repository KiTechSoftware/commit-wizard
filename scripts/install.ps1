# Install the prebuilt `cw` binary from GitHub Releases (Windows, x86_64).
#
#   iwr -useb https://raw.githubusercontent.com/KiTechSoftware/commit-wizard/main/scripts/install.ps1 | iex
#   .\scripts\install.ps1 -Version 0.0.5
param(
  [string]$Version = "latest",
  [string]$InstallDir = "$env:USERPROFILE\AppData\Local\Programs\cw"
)
$ErrorActionPreference = "Stop"
$repo = "KiTechSoftware/commit-wizard"

if ($Version -eq "latest") {
  $tag = (Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest").tag_name
} else {
  $tag = "v" + $Version.TrimStart("v")
}

$name = "cw-$tag-windows-x86_64"
$base = "https://github.com/$repo/releases/download/$tag"
$tmp = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
  Invoke-WebRequest "$base/$name.zip" -OutFile "$tmp\$name.zip"
  try {
    Invoke-WebRequest "$base/SHASUMS256.txt" -OutFile "$tmp\SHASUMS256.txt"
    $expected = ((Get-Content "$tmp\SHASUMS256.txt" | Where-Object { $_ -match "$name.zip$" }) -split "\s+")[0]
    $actual = (Get-FileHash "$tmp\$name.zip" -Algorithm SHA256).Hash.ToLower()
    if ($expected -ne $actual) { throw "checksum mismatch for $name.zip" }
  } catch [System.Net.WebException] {
    Write-Warning "SHASUMS256.txt not found; skipping checksum verification"
  }
  Expand-Archive "$tmp\$name.zip" -DestinationPath $tmp
  New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
  Copy-Item "$tmp\$name\cw.exe" "$InstallDir\cw.exe" -Force
  Write-Host "Installed cw $tag to $InstallDir\cw.exe"
  if (($env:PATH -split ";") -notcontains $InstallDir) { Write-Host "Add $InstallDir to your PATH, then run: cw --version" }
} finally {
  Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}

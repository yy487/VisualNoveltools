param(
    [string[]]$SourceDirs = @("arc2_z_named\files", "arc3_z_named\files"),
    [string]$OutputDir = "pic",
    [switch]$Overwrite
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$Converter = Join-Path $PSScriptRoot "tlg2png\tlg2png.exe"
if (!(Test-Path $Converter)) {
    throw "tlg2png.exe not found: $Converter"
}

$OutRoot = Join-Path $Root $OutputDir
New-Item -ItemType Directory -Force -Path $OutRoot | Out-Null

$Total = 0
$Skipped = 0
$Ok = 0
$Fail = 0
$Failed = New-Object System.Collections.Generic.List[string]

foreach ($SourceDir in $SourceDirs) {
    $Src = Join-Path $Root $SourceDir
    if (!(Test-Path $Src)) {
        Write-Warning "source not found: $Src"
        continue
    }

    Get-ChildItem -Path $Src -Recurse -Filter *.tlg -File | ForEach-Object {
        $Total++
        $Rel = $_.FullName.Substring($Src.Length).TrimStart("\", "/")
        $Dst = Join-Path $OutRoot ([System.IO.Path]::ChangeExtension($Rel, ".png"))

        if ((Test-Path $Dst) -and !$Overwrite) {
            $Skipped++
            return
        }

        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $Dst) | Out-Null
        & $Converter $_.FullName $Dst *> $null
        if ($LASTEXITCODE -eq 0 -and (Test-Path $Dst)) {
            $Ok++
        } else {
            $Fail++
            $Failed.Add($_.FullName)
        }

        if ((($Ok + $Fail) % 250) -eq 0) {
            Write-Output "converted=$($Ok + $Fail) skipped=$Skipped fail=$Fail"
        }
    }
}

Write-Output "DONE total=$Total skipped=$Skipped ok=$Ok fail=$Fail output=$OutRoot"

if ($Failed.Count -gt 0) {
    $FailedPath = Join-Path $OutRoot "failed_tlg2png.txt"
    $Failed | Set-Content -Path $FailedPath -Encoding UTF8
    Write-Output "failed list: $FailedPath"
}

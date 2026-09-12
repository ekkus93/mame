param(
    [Parameter(Mandatory = $true)]
    [string]$InstallerPath,

    [string]$ProductName = "MAME Tauri Frontend"
)

$ErrorActionPreference = "Stop"

function Get-InstalledProduct {
    param([string]$Name)

    $roots = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall"
    )

    foreach ($root in $roots) {
        if (-not (Test-Path $root)) {
            continue
        }
        $entry = Get-ChildItem $root | ForEach-Object {
            Get-ItemProperty $_.PSPath
        } | Where-Object { $_.DisplayName -eq $Name } | Select-Object -First 1
        if ($null -ne $entry) {
            return $entry
        }
    }
    return $null
}

function Assert-Path {
    param(
        [string]$Path,
        [string]$Description
    )

    if (-not (Test-Path $Path)) {
        throw "Missing $Description at $Path"
    }
}

$installer = (Resolve-Path $InstallerPath).Path
$installEntry = $null
$installDir = $null
$uninstaller = $null
$installed = $false

try {
    $process = Start-Process -FilePath $installer -ArgumentList "/S" -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        throw "NSIS installer exited with code $($process.ExitCode)"
    }
    $installed = $true

    $installEntry = Get-InstalledProduct -Name $ProductName
    if ($null -eq $installEntry) {
        throw "Installed product registry entry was not found for $ProductName"
    }

    $installDir = [string]$installEntry.InstallLocation
    $installDir = $installDir.Trim('"').TrimEnd('\')
    if ([string]::IsNullOrWhiteSpace($installDir)) {
        throw "Installer registry entry did not record InstallLocation"
    }

    Assert-Path -Path $installDir -Description "install directory"
    Assert-Path -Path (Join-Path $installDir "mame-runtime\bin\mame.exe") -Description "bundled MAME executable"
    Assert-Path -Path (Join-Path $installDir "mame-runtime\hash\fixture.xml") -Description "bundled MAME hash resource"
    Assert-Path -Path (Join-Path $installDir "mame-runtime\bgfx\chains\fixture.json") -Description "bundled MAME BGFX resource"
    Assert-Path -Path (Join-Path $installDir "mame-runtime\licenses\COPYING") -Description "MAME COPYING notice"
    Assert-Path -Path (Join-Path $installDir "mame-runtime\licenses\legal\GPL-2.0") -Description "MAME legal material"

    $mainExecutables = Get-ChildItem -Path $installDir -Filter "*.exe" -File | Where-Object {
        $_.Name -ne "uninstall.exe"
    }
    if ($mainExecutables.Count -lt 1) {
        throw "Installed application executable was not found in $installDir"
    }

    $uninstaller = Join-Path $installDir "uninstall.exe"
    Assert-Path -Path $uninstaller -Description "NSIS uninstaller"

    Write-Host "MT-1303 clean install and packaged runtime layout passed at $installDir"
}
finally {
    if ($installed -and $null -ne $installDir) {
        if ($null -eq $uninstaller) {
            $uninstaller = Join-Path $installDir "uninstall.exe"
        }
        if (Test-Path $uninstaller) {
            $uninstallProcess = Start-Process -FilePath $uninstaller -ArgumentList "/S" -Wait -PassThru
            if ($uninstallProcess.ExitCode -ne 0) {
                throw "NSIS uninstaller exited with code $($uninstallProcess.ExitCode)"
            }
        }
    }
}

for ($attempt = 0; $attempt -lt 30; $attempt++) {
    if (($null -eq $installDir -or -not (Test-Path $installDir)) -and
        ($null -eq (Get-InstalledProduct -Name $ProductName))) {
        Write-Host "MT-1303 clean uninstall passed"
        exit 0
    }
    Start-Sleep -Seconds 1
}

if ($null -ne $installDir -and (Test-Path $installDir)) {
    throw "Install directory remains after uninstall: $installDir"
}
if ($null -ne (Get-InstalledProduct -Name $ProductName)) {
    throw "Uninstall registry entry remains after uninstall for $ProductName"
}

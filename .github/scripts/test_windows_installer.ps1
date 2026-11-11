param(
    [Parameter(Mandatory = $true)][string]$Installer,
    [Parameter(Mandatory = $true)][string]$Version
)
$ErrorActionPreference = 'Stop'
$Installer = (Resolve-Path $Installer).Path
$uninstallRoot = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall'
$existing = @(Get-ChildItem $uninstallRoot -ErrorAction SilentlyContinue |
    Get-ItemProperty | Where-Object { $_.DisplayName -eq 'Purge' })
if ($existing.Count -ne 0) { throw 'An existing Purge installation must not be modified by this test' }
$shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) 'Purge.lnk'
if (Test-Path $shortcut) { throw 'An existing Purge shortcut must not be modified by this test' }

$temporaryRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$fixture = Join-Path $temporaryRoot ('Purge-installer-smoke-' + [guid]::NewGuid().ToString('N'))
$installDir = Join-Path $fixture 'application with spaces'
$stateDir = Join-Path $env:LOCALAPPDATA 'Purge'
$stateExisted = Test-Path $stateDir
$marker = Join-Path $stateDir ('installer-retention-' + [guid]::NewGuid().ToString('N') + '.json')
New-Item -ItemType Directory -Force -Path $fixture,$stateDir | Out-Null
Set-Content $marker '{"fixture":"keep user data"}' -Encoding utf8
$uninstaller = Join-Path $installDir 'unins000.exe'
$key = $null

function Invoke-Installer([string]$LogName) {
    $log = Join-Path $fixture $LogName
    $arguments = @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/SP-',
        ('/DIR="' + $installDir + '"'),('/LOG="' + $log + '"'))
    $process = Start-Process $Installer -ArgumentList $arguments -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        if (Test-Path $log) { Get-Content $log -Tail 60 | Write-Output }
        throw "Installer failed: $($process.ExitCode)"
    }
}

try {
    Invoke-Installer 'install.log'
    $binary = Join-Path $installDir 'purge.exe'
    if (!(Test-Path $binary) -or !(Test-Path $uninstaller)) { throw 'Executable or uninstaller missing' }
    $info = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($binary)
    if ($info.ProductVersion -ne $Version) { throw 'Installed executable version differs from the release' }
    if (!(Test-Path (Join-Path $installDir 'licenses/LICENSE-GPUI'))) { throw 'Third-party license missing' }
    if (!(Test-Path $shortcut)) { throw 'Start menu shortcut missing' }
    $shell = New-Object -ComObject WScript.Shell
    if ($shell.CreateShortcut($shortcut).TargetPath -ne $binary) { throw 'Shortcut points to the wrong executable' }
    $installed = @(Get-ChildItem $uninstallRoot | Get-ItemProperty | Where-Object { $_.DisplayName -eq 'Purge' })
    if ($installed.Count -ne 1 -or $installed[0].DisplayVersion -ne $Version) { throw 'Uninstall registration missing or duplicated' }
    $key = $installed[0].PSPath
    Write-Output 'Installation, version, license, shortcut and uninstall registration verified.'

    Invoke-Installer 'reinstall.log'
    $installed = @(Get-ChildItem $uninstallRoot | Get-ItemProperty | Where-Object { $_.DisplayName -eq 'Purge' })
    if ($installed.Count -ne 1 -or $installed[0].PSPath -ne $key) { throw 'Reinstall created a different installation' }
    if (!(Test-Path $marker)) { throw 'Reinstall removed user data' }
    Write-Output 'In-place reinstall preserves the installation and user data.'

    $process = Start-Process $uninstaller -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "Uninstall failed: $($process.ExitCode)" }
    if ((Test-Path $binary) -or (Test-Path $key) -or (Test-Path $shortcut)) { throw 'Uninstall left application files or registration' }
    if (!(Test-Path $marker)) { throw 'Uninstall removed user data' }
    Write-Output 'Uninstall removes application files and registration while preserving user data.'
} finally {
    if (Test-Path $uninstaller) {
        Start-Process $uninstaller -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -Wait | Out-Null
    }
    Remove-Item $marker -Force -ErrorAction SilentlyContinue
    if (!$stateExisted -and (Test-Path $stateDir) -and @(Get-ChildItem $stateDir -Force).Count -eq 0) {
        Remove-Item $stateDir -Force
    }
    if (!(Test-Path $uninstaller)) {
        Remove-Item $fixture -Recurse -Force -ErrorAction SilentlyContinue
    } else {
        Write-Output "Test installation retained for recovery: $fixture"
    }
}

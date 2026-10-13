[CmdletBinding()]
param([string]$SetupPath = (Join-Path (Split-Path -Parent $PSScriptRoot) 'dist\portal-setup.exe'))
$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$setupExecutable = (Resolve-Path -LiteralPath $SetupPath).Path
$appRegistryPath = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{C471416E-8E08-4911-A7D1-F4390BB36E7C}_is1'
$registry = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser, [Microsoft.Win32.RegistryView]::Registry64)
try {
    $existing = $registry.OpenSubKey($appRegistryPath)
    if ($existing) { $existing.Dispose(); throw 'Portal is already installed for this user. Run this check in a clean account.' }
} finally { $registry.Dispose() }

$testId = [Guid]::NewGuid().ToString('N')
$evidenceRoot = Join-Path $repositoryRoot "evidence\installer-$testId"
$installDirectory = Join-Path $evidenceRoot 'installed'
$groupName = "Portal installer test $testId"
$groupDirectory = Join-Path ([Environment]::GetFolderPath('Programs')) $groupName
$settingsDirectory = Join-Path $env:LOCALAPPDATA 'Portal'
$settingsSentinel = Join-Path $settingsDirectory "installer-test-$testId.txt"
$unrelatedSentinel = Join-Path $evidenceRoot 'unrelated-game.txt'
$userSentinel = Join-Path $installDirectory 'user-file.txt'
New-Item -ItemType Directory -Path $evidenceRoot,$settingsDirectory -Force | Out-Null
[IO.File]::WriteAllText($settingsSentinel, 'Test settings: preserve me')
[IO.File]::WriteAllText($unrelatedSentinel, 'Test unrelated game: preserve me')
$settingsHash = (Get-FileHash -LiteralPath $settingsSentinel).Hash
$unrelatedHash = (Get-FileHash -LiteralPath $unrelatedSentinel).Hash
$existingSettings = Join-Path $settingsDirectory 'settings.json'
$originalSettingsHash = if (Test-Path -LiteralPath $existingSettings) { (Get-FileHash -LiteralPath $existingSettings).Hash } else { $null }

function Run-Setup([string]$LogName) {
    $arguments = @('/CURRENTUSER','/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/SP-',
        "/DIR=`"$installDirectory`"", "/GROUP=`"$groupName`"", '/TASKS=""',
        "/LOG=`"$(Join-Path $evidenceRoot $LogName)`"")
    $process = Start-Process -FilePath $setupExecutable -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "Setup failed: $($process.ExitCode)" }
}

Run-Setup 'install.log'
$installedExecutable = Join-Path $installDirectory 'portal.exe'
$uninstaller = Join-Path $installDirectory 'unins000.exe'
foreach ($required in @($installedExecutable,$uninstaller,(Join-Path $groupDirectory 'Portal.lnk'),(Join-Path $groupDirectory 'Uninstall Portal.lnk'))) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) { throw "Missing installed artifact: $required" }
}
if ((Get-FileHash -LiteralPath $installedExecutable).Hash -ne (Get-FileHash -LiteralPath (Join-Path $repositoryRoot 'target\release\portal.exe')).Hash) {
    throw 'Installed executable differs from the release build.'
}
$registry = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser, [Microsoft.Win32.RegistryView]::Registry64)
try {
    $entry = $registry.OpenSubKey($appRegistryPath)
    if (-not $entry -or $entry.GetValue('InstallLocation').TrimEnd('\') -ne $installDirectory) { throw 'Installed Apps registration points to the wrong location.' }
    $entry.Dispose()
} finally { $registry.Dispose() }

$scanOutput = Join-Path $evidenceRoot 'installed-scan.txt'
$scanProcess = Start-Process -FilePath $installedExecutable -ArgumentList '--scan' -WindowStyle Hidden -RedirectStandardOutput $scanOutput -RedirectStandardError (Join-Path $evidenceRoot 'installed-scan-error.txt') -Wait -PassThru
$catalogSource = Get-Content -LiteralPath (Join-Path $repositoryRoot 'src\catalog.rs') -Raw
$launcherCountMatch = [regex]::Match($catalogSource, 'pub const ALL:\s*\[Self;\s*(\d+)\]')
if (-not $launcherCountMatch.Success) { throw 'Could not read the launcher count from the catalog.' }
if ($scanProcess.ExitCode -ne 0 -or @(Get-Content -LiteralPath $scanOutput).Count -ne [int]$launcherCountMatch.Groups[1].Value) { throw 'The installed executable could not scan every supported launcher.' }
[IO.File]::WriteAllText($userSentinel, 'Test user file: preserve me')
$userHash = (Get-FileHash -LiteralPath $userSentinel).Hash
Run-Setup 'upgrade.log'
if (@(Get-ChildItem -LiteralPath $installDirectory -Filter 'unins*.exe').Count -ne 1) { throw 'Reinstall produced duplicate uninstallers.' }

# Only execute the uninstaller that this test installed, under its unique evidence root.
$resolvedUninstaller = (Resolve-Path -LiteralPath $uninstaller).Path
$expectedRoot = [IO.Path]::GetFullPath($evidenceRoot) + [IO.Path]::DirectorySeparatorChar
if (-not $resolvedUninstaller.StartsWith($expectedRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Uninstaller escaped the test directory.' }
$uninstallProcess = Start-Process -FilePath $resolvedUninstaller -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',"/LOG=`"$(Join-Path $evidenceRoot 'uninstall.log')`"") -WindowStyle Hidden -Wait -PassThru
if ($uninstallProcess.ExitCode -ne 0) { throw "Uninstall failed: $($uninstallProcess.ExitCode)" }
for ($attempt = 0; $attempt -lt 50 -and (Test-Path -LiteralPath $uninstaller); $attempt++) { Start-Sleep -Milliseconds 100 }
foreach ($removed in @($installedExecutable,$uninstaller,(Join-Path $installDirectory 'unins000.dat'),(Join-Path $installDirectory 'licenses\GPUI-APACHE-2.0.txt'),(Join-Path $groupDirectory 'Portal.lnk'),(Join-Path $groupDirectory 'Uninstall Portal.lnk'))) {
    if (Test-Path -LiteralPath $removed) { throw "Uninstall left an installed artifact: $removed" }
}
foreach ($sentinel in @(@($settingsSentinel,$settingsHash),@($unrelatedSentinel,$unrelatedHash),@($userSentinel,$userHash))) {
    if (-not (Test-Path -LiteralPath $sentinel[0]) -or (Get-FileHash -LiteralPath $sentinel[0]).Hash -ne $sentinel[1]) { throw 'Uninstall changed a settings or unrelated-file sentinel.' }
}
if ($originalSettingsHash -and (Get-FileHash -LiteralPath $existingSettings).Hash -ne $originalSettingsHash) { throw 'Uninstall changed existing Portal settings.' }
$registry = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser, [Microsoft.Win32.RegistryView]::Registry64)
try {
    $entry = $registry.OpenSubKey($appRegistryPath)
    if ($entry) { $entry.Dispose(); throw 'Uninstall registration was not removed.' }
} finally { $registry.Dispose() }

# Remove only this test's uniquely named settings sentinel after checking its hash.
if ((Get-FileHash -LiteralPath $settingsSentinel).Hash -eq $settingsHash) { Remove-Item -LiteralPath $settingsSentinel }
$result = [ordered]@{ Install = 'passed'; InstalledScan = 'passed'; Reinstall = 'passed'; Uninstall = 'passed'; SettingsPreserved = $true; UnrelatedFilesPreserved = $true; RegistrationRemoved = $true; Evidence = $evidenceRoot }
$result | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $evidenceRoot 'result.json')
$result | ConvertTo-Json

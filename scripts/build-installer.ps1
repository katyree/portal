[CmdletBinding()]
param(
    [string]$CompilerPath,
    [switch]$SkipBuild
)
$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$installerOutput = Join-Path $repositoryRoot 'dist'
$portalExecutable = Join-Path $repositoryRoot 'target\release\portal.exe'

if (-not $IsWindows -and $PSVersionTable.PSVersion.Major -ge 6) {
    throw 'Portal installer builds require Windows.'
}

if (-not $SkipBuild) {
    Push-Location $repositoryRoot
    try {
        & cargo build --locked --release
        if ($LASTEXITCODE -ne 0) { throw 'Portal release build failed.' }
    } finally { Pop-Location }
}
if (-not (Test-Path -LiteralPath $portalExecutable -PathType Leaf)) {
    throw 'Build target\release\portal.exe before packaging.'
}

if (-not $CompilerPath) {
    $compilerDirectory = Join-Path $repositoryRoot '.tools\inno-6.7.1'
    $CompilerPath = Join-Path $compilerDirectory 'ISCC.exe'
    if (-not (Test-Path -LiteralPath $CompilerPath -PathType Leaf)) {
        $toolsDirectory = Join-Path $repositoryRoot '.tools'
        New-Item -ItemType Directory -Path $toolsDirectory -Force | Out-Null
        $compilerSetup = Join-Path $toolsDirectory 'innosetup-6.7.1.exe'
        $compilerUrl = 'https://github.com/jrsoftware/issrc/releases/download/is-6_7_1/innosetup-6.7.1.exe'
        Invoke-WebRequest -Uri $compilerUrl -OutFile $compilerSetup
        $expectedHash = '4D11E8050B6185E0D49BD9E8CC661A7A59F44959A621D31D11033124C4E8A7B0'
        if ((Get-FileHash -LiteralPath $compilerSetup -Algorithm SHA256).Hash -ne $expectedHash) {
            throw 'Inno Setup download hash did not match the pinned official release.'
        }
        if ((Get-AuthenticodeSignature -LiteralPath $compilerSetup).Status -ne 'Valid') {
            throw 'Inno Setup compiler download did not have a valid Windows signature.'
        }
        $compilerProcess = Start-Process -FilePath $compilerSetup -ArgumentList @(
            '/PORTABLE=1', '/CURRENTUSER', '/VERYSILENT', '/SUPPRESSMSGBOXES',
            '/NORESTART', '/SP-', "/DIR=`"$compilerDirectory`""
        ) -WindowStyle Hidden -Wait -PassThru
        if ($compilerProcess.ExitCode -ne 0) { throw "Inno Setup extraction failed: $($compilerProcess.ExitCode)" }
    }
}
if (-not (Test-Path -LiteralPath $CompilerPath -PathType Leaf)) { throw 'ISCC.exe was not found.' }

$manifest = Get-Content -LiteralPath (Join-Path $repositoryRoot 'Cargo.toml') -Raw
$versionMatch = [regex]::Match($manifest, '(?m)^version\s*=\s*"(\d+\.\d+\.\d+)"\s*$')
if (-not $versionMatch.Success) { throw 'Cargo.toml must contain a numeric package version.' }
$portalVersion = $versionMatch.Groups[1].Value
New-Item -ItemType Directory -Path $installerOutput -Force | Out-Null
& $CompilerPath "/DAppVersion=$portalVersion" "/DPortalExecutable=$portalExecutable" "/DRepositoryRoot=$repositoryRoot" "/DInstallerOutput=$installerOutput" (Join-Path $repositoryRoot 'installer\portal.iss')
if ($LASTEXITCODE -ne 0) { throw 'Portal installer compilation failed.' }
$setupPath = Join-Path $installerOutput 'portal-setup.exe'
if (-not (Test-Path -LiteralPath $setupPath -PathType Leaf)) { throw 'portal-setup.exe was not generated.' }
$setupHash = (Get-FileHash -LiteralPath $setupPath -Algorithm SHA256).Hash
[System.IO.File]::WriteAllText((Join-Path $installerOutput 'portal-setup.exe.sha256'), "$setupHash  portal-setup.exe`n")
Write-Output "Installer: $setupPath"
Write-Output "SHA256: $setupHash"

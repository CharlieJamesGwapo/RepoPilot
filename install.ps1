# Install a checksum-verified RepoPilot release without changing shell configuration.
[CmdletBinding()]
param(
    [ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?$')]
    [string]$Version = 'v0.3.0',
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'RepoPilot\bin')
)
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'This installer requires Windows. Use install.sh on macOS or Linux.' }
if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() -ne 'X64') {
    throw 'The Windows binary supports x64. Build from source for other architectures.'
}
$workDir = Join-Path ([IO.Path]::GetTempPath()) ('repopilot-install-' + [guid]::NewGuid().ToString('N'))
$installTemp = $null
New-Item -ItemType Directory -Path $workDir | Out-Null
try {
    $name = "repopilot-$Version-x86_64-pc-windows-msvc"
    $asset = "$name.zip"
    $base = "https://github.com/CharlieJamesGwapo/RepoPilot/releases/download/$Version"
    $archive = Join-Path $workDir $asset
    Write-Output "Downloading RepoPilot $Version for Windows x64..."
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri "$base/$asset" -OutFile $archive -UseBasicParsing -TimeoutSec 180
    $checksum = Join-Path $workDir 'checksum'
    Invoke-WebRequest -Uri "$base/$asset.sha256" -OutFile $checksum -UseBasicParsing -TimeoutSec 180
    $expected = ((Get-Content -LiteralPath $checksum -Raw).Trim() -split '\s+')[0]
    if ($expected -notmatch '^[0-9a-fA-F]{64}$') { throw 'Invalid SHA-256 checksum file.' }
    if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne $expected) {
        throw 'SHA-256 mismatch; nothing was installed.'
    }
    # Read only the expected executable; never extract arbitrary ZIP paths.
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [IO.Compression.ZipFile]::OpenRead($archive)
    $binary = Join-Path $workDir 'repopilot.exe'
    try {
        $entry = $zip.GetEntry("$name/repopilot.exe")
        if ($null -eq $entry) { throw 'Executable is missing from the archive.' }
        [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $binary)
    } finally { $zip.Dispose() }
    $actualVersion = (& $binary --version | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $actualVersion -ne "repopilot $($Version.Substring(1))") {
        throw 'Downloaded binary version did not match; nothing was installed.'
    }
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $destination = Join-Path $InstallDir 'repopilot.exe'
    $installTemp = Join-Path $InstallDir ('.repopilot-install-' + [guid]::NewGuid().ToString('N') + '.exe')
    Copy-Item -LiteralPath $binary -Destination $installTemp
    if (Test-Path -LiteralPath $destination) {
        # PowerShell casts $null to an empty string for this .NET parameter.
        [IO.File]::Replace($installTemp, $destination, [NullString]::Value)
    } else {
        [IO.File]::Move($installTemp, $destination)
    }
    $installTemp = $null
    Write-Output "Installed $destination"
    Write-Output "Add this directory to your user PATH, then open a new terminal: $InstallDir"
    Write-Output 'Run repopilot to open your shell, or repopilot --help to explore.'
} finally {
    if ($null -ne $installTemp -and (Test-Path -LiteralPath $installTemp)) { Remove-Item -LiteralPath $installTemp -Force }
    Remove-Item -LiteralPath $workDir -Recurse -Force
}

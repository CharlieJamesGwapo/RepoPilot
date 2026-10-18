param([Parameter(Mandatory=$true)][string]$PackageDir, [string]$Version='v0.1.0')
$ErrorActionPreference='Stop'
$fixtureDir = (Resolve-Path $PackageDir).Path
$script:corruptChecksum = $false
function Invoke-WebRequest {
    param([string]$Uri, [string]$OutFile, [switch]$UseBasicParsing)
    $filename = ([Uri]$Uri).Segments[-1]
    if ($script:corruptChecksum -and $filename.EndsWith('.sha256')) {
        [IO.File]::WriteAllText($OutFile, ('0' * 64))
    } else {
        Copy-Item -LiteralPath (Join-Path $fixtureDir $filename) -Destination $OutFile
    }
}
$location = Join-Path ([IO.Path]::GetTempPath()) ('repopilot-test-' + [guid]::NewGuid().ToString('N'))
try {
    $installDir = Join-Path $location 'bin with spaces'
    & "$PSScriptRoot/../install.ps1" -Version $Version -InstallDir $installDir
    $binary = Join-Path $installDir 'repopilot.exe'
    $expected = "repopilot $($Version.Substring(1))"
    if ((& $binary --version) -ne $expected -or $LASTEXITCODE -ne 0) { throw 'Installed binary smoke check failed.' }
    $before = (Get-FileHash -LiteralPath $binary).Hash
    # Check updating an existing installation using the same archive.
    & "$PSScriptRoot/../install.ps1" -Version $Version -InstallDir $installDir
    if ((Get-FileHash -LiteralPath $binary).Hash -ne $before) { throw 'Repeated installation changed binary unexpectedly.' }
    $script:corruptChecksum = $true
    $rejected = $false
    try { & "$PSScriptRoot/../install.ps1" -Version $Version -InstallDir $installDir }
    catch { if ($_.Exception.Message -notlike '*SHA-256 mismatch*') { throw }; $rejected = $true }
    if (-not $rejected) { throw 'A bad checksum was accepted.' }
    if ((Get-FileHash -LiteralPath $binary).Hash -ne $before) { throw 'A rejected download damaged the existing binary.' }
    Write-Output 'Windows installer: clean install, update, checksum rejection and existing binary preservation passed.'
} finally {
    if (Test-Path -LiteralPath $location) { Remove-Item -LiteralPath $location -Recurse -Force }
}

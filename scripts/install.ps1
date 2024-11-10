# Check if running with appropriate permissions
if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "Please run this script as Administrator"
    exit 1
}

$InstallDir = "$env:LOCALAPPDATA\StitchSync\bin"
$ExeName = "stitch-sync.exe"
$RepoWithOwner = "osteele/stitch-sync"

# Create installation directory if it doesn't exist
if (-not (Test-Path $InstallDir)) {
    Write-Host "Creating installation directory..."
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Get latest release info from GitHub API
Write-Host "Fetching latest release information..."
try {
    $ReleaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$RepoWithOwner/releases/latest"
    $ReleaseVersion = $ReleaseInfo.tag_name
} catch {
    Write-Error "Failed to fetch release information"
    exit 1
}

Write-Host "Latest version: $ReleaseVersion"
$AssetName = "stitch-sync-x86_64-pc-windows-msvc.tar.gz"
$DownloadUrl = "https://github.com/$RepoWithOwner/releases/download/$ReleaseVersion/$AssetName"

# Create temporary directory
$TempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
$TempFile = Join-Path $TempDir $AssetName

# Download release
Write-Host "Downloading StitchSync $ReleaseVersion..."
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile
} catch {
    Write-Error "Failed to download release"
    Remove-Item -Recurse -Force $TempDir
    exit 1
}

# Extract archive
Write-Host "Extracting archive..."
try {
    tar -xzf $TempFile -C $TempDir
} catch {
    Write-Error "Failed to extract archive"
    Remove-Item -Recurse -Force $TempDir
    exit 1
}

# Install executable
Write-Host "Installing to $InstallDir..."
try {
    Move-Item -Force -Path (Join-Path $TempDir $ExeName) -Destination (Join-Path $InstallDir $ExeName)
} catch {
    Write-Error "Failed to install executable"
    Remove-Item -Recurse -Force $TempDir
    exit 1
}

# Clean up
Remove-Item -Recurse -Force $TempDir

# Add to PATH if not already present
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding installation directory to PATH..."
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$UserPath;$InstallDir",
        "User"
    )
    $env:Path = "$env:Path;$InstallDir"
}

# Print styled success message
Write-Host "`n╔════════════════════════════════════════════════════════════════╗"
Write-Host "║                   StitchSync Installation                      ║"
Write-Host "╚════════════════════════════════════════════════════════════════╝`n"

Write-Host "✓ Successfully installed StitchSync $ReleaseVersion to $InstallDir\$ExeName`n"

Write-Host "⚠️  Warning: Stitch-sync has been minimally tested on Windows. USB drive detection is experimental and has not been thoroughly tested on this platform.`n"

Write-Host "Getting Started:"
Write-Host "───────────────"
Write-Host "  • Run 'stitch-sync --help' to see all available commands"
Write-Host "  • Run 'stitch-sync watch' to start watching for new designs`n"

Write-Host "Configuration (Optional):"
Write-Host "─────────────────────────"
Write-Host "  • Run 'stitch-sync set machine' to set your embroidery machine"
Write-Host
Write-Host "  This is necessary if your embroidery machine requires a different"
Write-Host "  format than the default (DST), or if it requires the output files"
Write-Host "  to be located in a specific directory on the USB drive.`n"

Write-Host "  • Run 'stitch-sync list-machines' to see supported machines`n"

# Notify user that they need to restart their terminal to use the updated PATH
Write-Host "Note: You may need to restart your terminal to use the 'stitch-sync' command.`n"

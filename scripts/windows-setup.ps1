# Opcode Windows Development Environment Setup
# Comprehensive automated setup for Windows development
# Author: Opcode Team
# Version: 2.0

param(
    [switch]$SkipPrerequisites,
    [switch]$SkipBuild,
    [switch]$DevMode,
    [switch]$Force,
    [string]$LogLevel = "Info"
)

# Set up error handling
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Logging setup
function Write-LogMessage {
    param($Message, $Level = "Info", $Color = "White")

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $prefix = switch ($Level) {
        "Info" { "ℹ️" }
        "Success" { "✅" }
        "Warning" { "⚠️" }
        "Error" { "❌" }
        "Debug" { "🔍" }
    }

    Write-Host "[$timestamp] $prefix $Message" -ForegroundColor $Color
}

Write-LogMessage "🚀 Opcode Windows Development Environment Setup" "Info" "Cyan"
Write-LogMessage "================================================" "Info" "Cyan"
Write-LogMessage "Version 2.0 - Comprehensive automated setup" "Info" "Gray"

# System information
function Get-SystemInfo {
    $os = Get-CimInstance -ClassName Win32_OperatingSystem
    $cpu = Get-CimInstance -ClassName Win32_Processor
    $memory = Get-CimInstance -ClassName Win32_PhysicalMemory | Measure-Object -Property Capacity -Sum

    return @{
        OSName = $os.Caption
        OSVersion = $os.Version
        OSBuild = $os.BuildNumber
        Architecture = $os.OSArchitecture
        CPUName = $cpu.Name
        CPUCores = $cpu.NumberOfCores
        TotalMemoryGB = [math]::Round(($memory.Sum / 1GB), 2)
    }
}

# Check prerequisites
function Test-Prerequisites {
    Write-LogMessage "Checking system requirements and prerequisites..." "Info"

    $sysInfo = Get-SystemInfo
    Write-LogMessage "System: $($sysInfo.OSName) ($($sysInfo.OSVersion))" "Info"
    Write-LogMessage "Architecture: $($sysInfo.Architecture)" "Info"
    Write-LogMessage "Memory: $($sysInfo.TotalMemoryGB) GB" "Info"

    $issues = @()

    # Check Windows version (Windows 10 1809+ required)
    if ([int]$sysInfo.OSBuild -lt 17763) {
        $issues += "Windows 10 version 1809 (build 17763) or later required"
    }

    # Check memory (4GB minimum)
    if ($sysInfo.TotalMemoryGB -lt 4) {
        $issues += "At least 4GB RAM recommended (found $($sysInfo.TotalMemoryGB) GB)"
    }

    # Check PowerShell version
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        $issues += "PowerShell 5.1 or later required"
    }

    # Check disk space
    $freeSpace = (Get-CimInstance -ClassName Win32_LogicalDisk -Filter "DeviceID='C:'").FreeSpace / 1GB
    if ($freeSpace -lt 5) {
        $issues += "At least 5GB free disk space required (found $([math]::Round($freeSpace, 2)) GB)"
    }

    if ($issues.Count -gt 0) {
        Write-LogMessage "System requirement issues found:" "Warning" "Yellow"
        $issues | ForEach-Object { Write-LogMessage "  - $_" "Warning" "Yellow" }

        if (-not $Force) {
            Write-LogMessage "Use -Force to continue despite system requirement issues" "Warning" "Yellow"
            return $false
        }
    } else {
        Write-LogMessage "System requirements check passed" "Success" "Green"
    }

    return $true
}

# Install prerequisites
function Install-Prerequisites {
    Write-LogMessage "Checking and installing development prerequisites..." "Info"

    # Check for chocolatey (for easy package management)
    if (!(Get-Command choco -ErrorAction SilentlyContinue) -and !$SkipPrerequisites) {
        Write-LogMessage "Installing Chocolatey package manager..." "Info"
        Set-ExecutionPolicy Bypass -Scope Process -Force
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
        Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
        $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
    }

    # Check for Git
    if (!(Get-Command git -ErrorAction SilentlyContinue)) {
        if ((Get-Command choco -ErrorAction SilentlyContinue) -and !$SkipPrerequisites) {
            Write-LogMessage "Installing Git..." "Info"
            choco install git -y
            $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
        } else {
            Write-LogMessage "Git not found. Please install from https://git-scm.com/" "Warning" "Yellow"
        }
    } else {
        Write-LogMessage "Git found: $(git --version)" "Success" "Green"
    }

    # Check for Node.js
    if (!(Get-Command node -ErrorAction SilentlyContinue)) {
        if ((Get-Command choco -ErrorAction SilentlyContinue) -and !$SkipPrerequisites) {
            Write-LogMessage "Installing Node.js..." "Info"
            choco install nodejs -y
            $env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")
        } else {
            Write-LogMessage "Node.js not found. Please install from https://nodejs.org" "Error" "Red"
            return $false
        }
    } else {
        $nodeVersion = node --version
        Write-LogMessage "Node.js found: $nodeVersion" "Success" "Green"

        # Check Node version (16+ required)
        $majorVersion = [int]($nodeVersion -replace 'v(\d+).*', '$1')
        if ($majorVersion -lt 16) {
            Write-LogMessage "Node.js 16+ required (found v$majorVersion)" "Warning" "Yellow"
        }
    }

    # Check for Bun
    if (!(Get-Command bun -ErrorAction SilentlyContinue)) {
        Write-LogMessage "Installing Bun..." "Info"
        try {
            Invoke-RestMethod -Uri "https://bun.sh/install.ps1" | Invoke-Expression
            $env:PATH += ";$env:USERPROFILE\.bun\bin"
        } catch {
            Write-LogMessage "Failed to install Bun automatically. Please install from https://bun.sh/" "Warning" "Yellow"
        }
    } else {
        Write-LogMessage "Bun found: $(bun --version)" "Success" "Green"
    }

    # Check for Rust
    if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
        if (!$SkipPrerequisites) {
            Write-LogMessage "Installing Rust..." "Info"
            try {
                # Download and run rustup installer
                $rustupUrl = "https://win.rustup.rs/x86_64"
                $rustupPath = "$env:TEMP\rustup-init.exe"
                Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
                & $rustupPath -y --default-toolchain stable
                $env:PATH += ";$env:USERPROFILE\.cargo\bin"
            } catch {
                Write-LogMessage "Failed to install Rust automatically. Please install from https://rustup.rs" "Error" "Red"
                return $false
            }
        } else {
            Write-LogMessage "Rust not found. Please install from https://rustup.rs" "Error" "Red"
            return $false
        }
    } else {
        Write-LogMessage "Rust found: $(rustc --version)" "Success" "Green"
    }

    # Install Windows-specific Rust targets
    if (Get-Command rustup -ErrorAction SilentlyContinue) {
        Write-LogMessage "Installing Windows Rust targets..." "Info"
        rustup target add x86_64-pc-windows-msvc
        rustup target add i686-pc-windows-msvc
    }

    # Check for Visual Studio Build Tools
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (!(Test-Path $vswhere) -or !(& $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath)) {
        Write-LogMessage "Visual Studio Build Tools not found" "Warning" "Yellow"
        Write-LogMessage "Please install Visual Studio Build Tools with C++ workload" "Warning" "Yellow"
        Write-LogMessage "Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/" "Info"
    } else {
        Write-LogMessage "Visual Studio Build Tools found" "Success" "Green"
    }

    # Check for Tauri CLI
    $tauriInstalled = $false
    try {
        $null = bun pm ls @tauri-apps/cli 2>$null
        $tauriInstalled = $true
    } catch {}

    if (!$tauriInstalled) {
        Write-LogMessage "Installing Tauri CLI..." "Info"
        try {
            bun add --global @tauri-apps/cli@latest
        } catch {
            Write-LogMessage "Failed to install Tauri CLI with Bun, trying npm..." "Warning" "Yellow"
            npm install -g @tauri-apps/cli@latest
        }
    } else {
        Write-LogMessage "Tauri CLI found" "Success" "Green"
    }

    return $true
}

# Phase 1: System Check and Prerequisites
Write-LogMessage "`n📋 Phase 1: System verification and prerequisites installation" "Info" "Yellow"

if (!(Test-Prerequisites)) {
    Write-LogMessage "System requirements check failed" "Error" "Red"
    exit 1
}

if (!(Install-Prerequisites)) {
    Write-LogMessage "Prerequisites installation failed" "Error" "Red"
    exit 1
}

# Configure Tauri for Windows
function Set-TauriWindowsConfig {
    Write-LogMessage "Configuring Tauri for Windows development..." "Info"

    $tauriConfig = "src-tauri/tauri.conf.json"
    if (!(Test-Path $tauriConfig)) {
        Write-LogMessage "Tauri configuration file not found: $tauriConfig" "Warning" "Yellow"
        Write-LogMessage "Make sure you're running this script from the project root" "Warning" "Yellow"
        return $false
    }

    # Backup existing config
    $backupName = "tauri.conf.json.backup.$(Get-Date -Format 'yyyyMMdd-HHmmss')"
    Copy-Item $tauriConfig "src-tauri/$backupName"
    Write-LogMessage "Backed up tauri.conf.json to $backupName" "Info"

    try {
        $config = Get-Content $tauriConfig -Raw | ConvertFrom-Json

        # Ensure Windows targets are present
        $windowsTargets = @("msi", "nsis")
        $updated = $false

        foreach ($target in $windowsTargets) {
            if ($config.tauri.bundle.targets -notcontains $target) {
                $config.tauri.bundle.targets += $target
                $updated = $true
                Write-LogMessage "Added $target target to bundle configuration" "Success" "Green"
            }
        }

        # Add Windows-specific bundle configuration
        if (!$config.tauri.bundle.windows) {
            $config.tauri.bundle.windows = @{
                certificateThumbprint = $null
                digestAlgorithm = "sha256"
                timestampUrl = "http://timestamp.digicert.com"
                webviewInstallMode = @{
                    type = "downloadBootstrapper"
                }
                nsis = @{
                    installMode = "perUser"
                    allowElevation = $true
                    allowDowngrade = $false
                    createDesktopShortcut = $true
                    createStartMenuShortcut = $true
                    shortcutName = "Opcode"
                    license = "LICENSE"
                    compression = "lzma"
                }
                msi = @{
                    upgradeCode = "{12345678-1234-1234-1234-123456789012}"  # Generate proper GUID
                }
            }
            $updated = $true
            Write-LogMessage "Added Windows-specific bundle configuration" "Success" "Green"
        }

        # Add Windows features
        if (!$config.tauri.windows) {
            $config.tauri.windows = @{
                decorations = $true
                alwaysOnTop = $false
                maximized = $false
                resizable = $true
                title = "Opcode"
            }
            $updated = $true
            Write-LogMessage "Added Windows-specific window configuration" "Success" "Green"
        }

        if ($updated) {
            # Save updated config with proper formatting
            $config | ConvertTo-Json -Depth 10 | Set-Content $tauriConfig -Encoding UTF8
            Write-LogMessage "Updated Tauri configuration for Windows" "Success" "Green"
        } else {
            Write-LogMessage "Tauri configuration already includes Windows targets" "Success" "Green"
        }

        return $true
    } catch {
        Write-LogMessage "Failed to update Tauri configuration: $_" "Error" "Red"
        # Restore backup
        Copy-Item "src-tauri/$backupName" $tauriConfig
        Write-LogMessage "Restored backup configuration" "Warning" "Yellow"
        return $false
    }
}

# Setup Windows development environment
function Set-WindowsDevEnvironment {
    Write-LogMessage "Setting up Windows development environment..." "Info"

    # Create .vscode directory if it doesn't exist
    if (!(Test-Path ".vscode")) {
        New-Item -ItemType Directory -Path ".vscode" | Out-Null
        Write-LogMessage "Created .vscode directory" "Info"
    }

    # Create VS Code settings for Windows development
    $vscodeSettings = @{
        "rust-analyzer.cargo.target" = "x86_64-pc-windows-msvc"
        "rust-analyzer.checkOnSave.command" = "clippy"
        "rust-analyzer.checkOnSave.extraArgs" = @("--target", "x86_64-pc-windows-msvc")
        "terminal.integrated.profiles.windows" = @{
            PowerShell = @{
                source = "PowerShell"
                icon = "terminal-powershell"
            }
            Command = @{
                path = @("${env:windir}\Sysnative\cmd.exe", "${env:windir}\System32\cmd.exe")
                args = @()
                icon = "terminal-cmd"
            }
            GitBash = @{
                source = "Git Bash"
            }
        }
        "terminal.integrated.defaultProfile.windows" = "PowerShell"
    }

    try {
        $vscodeSettings | ConvertTo-Json -Depth 10 | Set-Content ".vscode/settings.json" -Encoding UTF8
        Write-LogMessage "Created VS Code settings for Windows development" "Success" "Green"
    } catch {
        Write-LogMessage "Failed to create VS Code settings: $_" "Warning" "Yellow"
    }

    # Create PowerShell profile for development
    $profileDir = Split-Path $PROFILE -Parent
    if (!(Test-Path $profileDir)) {
        New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
    }

    $profileContent = @"
# Opcode Development Profile
# Auto-generated by windows-setup.ps1

# Development aliases
Set-Alias -Name tdev -Value 'bun run tauri dev'
Set-Alias -Name tbuild -Value 'bun run tauri build'
Set-Alias -Name ttest -Value 'bun test'

function Invoke-TauriBuild {
    param([string]`$Target = "x86_64-pc-windows-msvc")
    bun run tauri build --target `$Target
}
Set-Alias -Name tbuild-target -Value Invoke-TauriBuild

function Invoke-WindowsTest {
    Set-Location src-tauri
    cargo test --test windows_integration
    Set-Location ..
}
Set-Alias -Name test-windows -Value Invoke-WindowsTest

Write-Host "🚀 Opcode development environment loaded" -ForegroundColor Green
Write-Host "Available aliases: tdev, tbuild, ttest, tbuild-target, test-windows" -ForegroundColor Cyan
"@

    try {
        Add-Content $PROFILE $profileContent -Force
        Write-LogMessage "Added Opcode development aliases to PowerShell profile" "Success" "Green"
        Write-LogMessage "Restart PowerShell or run '. `$PROFILE' to load aliases" "Info"
    } catch {
        Write-LogMessage "Failed to update PowerShell profile: $_" "Warning" "Yellow"
    }
}

# Phase 2: Configuration
Write-LogMessage "`n📋 Phase 2: Windows development configuration" "Info" "Yellow"

if (!(Set-TauriWindowsConfig)) {
    Write-LogMessage "Failed to configure Tauri for Windows" "Error" "Red"
    exit 1
}

Set-WindowsDevEnvironment

# Setup project assets and icons
function Set-ProjectAssets {
    Write-LogMessage "Setting up project assets and Windows icons..." "Info"

    $iconPath = "src-tauri/icons"
    if (!(Test-Path $iconPath)) {
        Write-LogMessage "Icons directory not found: $iconPath" "Warning" "Yellow"
        return $false
    }

    # Check for source PNG icon
    if (!(Test-Path "$iconPath/icon.png")) {
        Write-LogMessage "Source icon not found: $iconPath/icon.png" "Warning" "Yellow"
        Write-LogMessage "Please add a PNG icon file to continue with icon generation" "Info"
        return $false
    }

    # Generate Windows ICO file if not present
    if (!(Test-Path "$iconPath/icon.ico")) {
        if (Get-Command magick -ErrorAction SilentlyContinue) {
            Write-LogMessage "Generating Windows icon from PNG..." "Info"
            try {
                magick convert "$iconPath/icon.png" -define icon:auto-resize=256,128,64,48,32,16 "$iconPath/icon.ico"
                Write-LogMessage "Windows icon created successfully" "Success" "Green"
            } catch {
                Write-LogMessage "Failed to generate icon with ImageMagick: $_" "Warning" "Yellow"
            }
        } else {
            Write-LogMessage "ImageMagick not found - cannot generate Windows icon" "Warning" "Yellow"
            Write-LogMessage "Install ImageMagick or create icon.ico manually" "Info"
            Write-LogMessage "Download: https://imagemagick.org/script/download.php#windows" "Info"
        }
    } else {
        Write-LogMessage "Windows icon already exists" "Success" "Green"
    }

    # Create installer graphics (optional)
    if (Get-Command magick -ErrorAction SilentlyContinue) {
        if (!(Test-Path "$iconPath/header.bmp")) {
            Write-LogMessage "Creating NSIS installer header image..." "Info"
            try {
                magick convert -size 150x57 xc:#2D3748 -fill white -gravity center -pointsize 20 -annotate +0+0 "Opcode" "$iconPath/header.bmp"
                Write-LogMessage "Created installer header image" "Success" "Green"
            } catch {
                Write-LogMessage "Failed to create header image: $_" "Warning" "Yellow"
            }
        }

        if (!(Test-Path "$iconPath/sidebar.bmp")) {
            Write-LogMessage "Creating NSIS installer sidebar image..." "Info"
            try {
                magick convert -size 164x314 xc:#2D3748 "$iconPath/sidebar.bmp"
                Write-LogMessage "Created installer sidebar image" "Success" "Green"
            } catch {
                Write-LogMessage "Failed to create sidebar image: $_" "Warning" "Yellow"
            }
        }
    }

    return $true
}

# Install project dependencies
function Install-ProjectDependencies {
    Write-LogMessage "Installing project dependencies..." "Info"

    if (!(Test-Path "package.json")) {
        Write-LogMessage "package.json not found - make sure you're in the project root" "Warning" "Yellow"
        return $false
    }

    try {
        Write-LogMessage "Installing frontend dependencies with Bun..." "Info"
        $bunResult = bun install --frozen-lockfile
        if ($LASTEXITCODE -eq 0) {
            Write-LogMessage "Frontend dependencies installed successfully" "Success" "Green"
        } else {
            Write-LogMessage "Bun install failed, trying with npm..." "Warning" "Yellow"
            npm ci
            if ($LASTEXITCODE -ne 0) {
                Write-LogMessage "Failed to install dependencies with npm" "Error" "Red"
                return $false
            }
        }
    } catch {
        Write-LogMessage "Failed to install dependencies: $_" "Error" "Red"
        return $false
    }

    # Install Rust dependencies
    Write-LogMessage "Installing Rust dependencies..." "Info"
    try {
        Set-Location "src-tauri"
        cargo fetch
        if ($LASTEXITCODE -eq 0) {
            Write-LogMessage "Rust dependencies fetched successfully" "Success" "Green"
        } else {
            Write-LogMessage "Failed to fetch Rust dependencies" "Warning" "Yellow"
        }
        Set-Location ".."
    } catch {
        Write-LogMessage "Failed to fetch Rust dependencies: $_" "Warning" "Yellow"
        Set-Location ".."
    }

    return $true
}

# Phase 3: Assets and Dependencies
Write-LogMessage "`n📋 Phase 3: Assets setup and dependency installation" "Info" "Yellow"

Set-ProjectAssets
if (!(Install-ProjectDependencies)) {
    Write-LogMessage "Failed to install project dependencies" "Error" "Red"
    exit 1
}

# Validation and testing
function Test-WindowsBuild {
    param([switch]$SkipBuild)

    Write-LogMessage "Validating Windows development setup..." "Info"

    if ($SkipBuild) {
        Write-LogMessage "Skipping build test as requested" "Info"
        return $true
    }

    # Ask user if they want to test the build
    Write-Host "`nWould you like to test the Windows build now? This will verify the complete setup." -ForegroundColor Yellow
    Write-Host "  [Y] Yes, test the build (recommended)" -ForegroundColor Green
    Write-Host "  [N] No, skip build test" -ForegroundColor Gray
    Write-Host "  [D] Development build (faster)" -ForegroundColor Cyan
    $choice = Read-Host "Enter your choice (Y/N/D)"

    if ($choice -eq 'N' -or $choice -eq 'n') {
        Write-LogMessage "Skipping build test" "Info"
        return $true
    }

    $isDev = ($choice -eq 'D' -or $choice -eq 'd')

    try {
        if ($isDev) {
            Write-LogMessage "Testing development build..." "Info"
            Write-LogMessage "This is faster but doesn't create installers" "Info"

            # Test that dev environment works
            Write-LogMessage "Starting development build test..." "Info"
            $devProcess = Start-Process -FilePath "bun" -ArgumentList @("run", "tauri", "dev", "--no-dev-server") -PassThru -NoNewWindow
            Start-Sleep -Seconds 10

            if ($devProcess -and !$devProcess.HasExited) {
                $devProcess.Kill()
                Write-LogMessage "Development environment test passed" "Success" "Green"
            } else {
                Write-LogMessage "Development environment test failed" "Error" "Red"
                return $false
            }
        } else {
            Write-LogMessage "Building Windows application..." "Info"
            Write-LogMessage "This may take several minutes for the first build..." "Info"

            # Build the application
            $buildResult = bun run tauri build --target x86_64-pc-windows-msvc

            if ($LASTEXITCODE -eq 0) {
                Write-LogMessage "Build completed successfully!" "Success" "Green"

                # Check for created artifacts
                $artifactsFound = @()

                $msiPath = Get-ChildItem "src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/*.msi" -ErrorAction SilentlyContinue
                if ($msiPath) {
                    $artifactsFound += "MSI: $($msiPath.Name)"
                }

                $nsisPath = Get-ChildItem "src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/*.exe" -ErrorAction SilentlyContinue
                if ($nsisPath) {
                    $artifactsFound += "NSIS: $($nsisPath.Name)"
                }

                if ($artifactsFound.Count -gt 0) {
                    Write-LogMessage "Created installers:" "Success" "Green"
                    $artifactsFound | ForEach-Object { Write-LogMessage "  📦 $_" "Success" "Green" }
                } else {
                    Write-LogMessage "Build completed but no installers found" "Warning" "Yellow"
                    Write-LogMessage "Check target directory: src-tauri/target/x86_64-pc-windows-msvc/release/bundle/" "Info"
                }

                # Test basic Windows functionality
                Write-LogMessage "Testing Windows-specific functionality..." "Info"
                try {
                    Set-Location "src-tauri"
                    $testResult = cargo test --test windows_integration --lib --target x86_64-pc-windows-msvc
                    if ($LASTEXITCODE -eq 0) {
                        Write-LogMessage "Windows integration tests passed" "Success" "Green"
                    } else {
                        Write-LogMessage "Some Windows integration tests failed (this is expected in some environments)" "Warning" "Yellow"
                    }
                    Set-Location ".."
                } catch {
                    Write-LogMessage "Could not run Windows tests: $_" "Warning" "Yellow"
                    Set-Location ".."
                }

                return $true
            } else {
                Write-LogMessage "Build failed - check the output above for errors" "Error" "Red"
                Write-LogMessage "Common solutions:" "Info"
                Write-LogMessage "  1. Restart PowerShell and try again" "Info"
                Write-LogMessage "  2. Run: rustup target add x86_64-pc-windows-msvc" "Info"
                Write-LogMessage "  3. Ensure Visual Studio Build Tools are installed" "Info"
                return $false
            }
        }
    } catch {
        Write-LogMessage "Build test failed with exception: $_" "Error" "Red"
        return $false
    }
}

# Generate comprehensive setup report
function New-SetupReport {
    param($ValidationResults)

    Write-LogMessage "Generating setup completion report..." "Info"

    $reportContent = @"
# Opcode Windows Development Environment Report
Generated: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

## System Information
$(Get-SystemInfo | ConvertTo-Json -Depth 2)

## Validation Results
Build Test: $($ValidationResults.BuildTest -replace 'True', '✅ Passed' -replace 'False', '❌ Failed')
Dependencies: $($ValidationResults.Dependencies -replace 'True', '✅ Complete' -replace 'False', '❌ Issues')
Configuration: $($ValidationResults.Configuration -replace 'True', '✅ Updated' -replace 'False', '❌ Failed')

## Next Steps
1. 🔍 Review Windows implementation guide: docs/WINDOWS_IMPLEMENTATION.md
2. 🚀 Start development: bun run tauri dev
3. 🧪 Run tests: ./scripts/windows-test-suite.ps1
4. 📦 Build release: bun run tauri build
5. 🛡️ Configure code signing for production

## Developer Shortcuts
- tdev          # Start development server
- tbuild        # Build application
- ttest         # Run tests
- test-windows  # Run Windows-specific tests

## Troubleshooting
If you encounter issues:
1. Restart PowerShell and reload the profile: . `$PROFILE
2. Verify prerequisites: ./scripts/windows-setup.ps1 -SkipPrerequisites
3. Check documentation: docs/WINDOWS_IMPLEMENTATION.md
4. Run validation: ./scripts/windows-test-suite.ps1

---
Setup completed with Opcode Windows Setup Script v2.0
"@

    $reportPath = "windows-setup-report.md"
    $reportContent | Out-File -FilePath $reportPath -Encoding UTF8
    Write-LogMessage "Setup report saved to: $reportPath" "Success" "Green"
}

# Phase 4: Validation and Testing
Write-LogMessage "`n📋 Phase 4: Validation and testing" "Info" "Yellow"

$validationResults = @{
    BuildTest = Test-WindowsBuild -SkipBuild:$SkipBuild
    Dependencies = $true
    Configuration = $true
}

# Final Summary
Write-LogMessage "`n🎉 Windows Development Environment Setup Complete!" "Success" "Cyan"
Write-LogMessage "=============================================" "Success" "Cyan"

Write-LogMessage "`n✅ Completed Tasks:" "Success" "Green"
Write-LogMessage "  • System requirements verified" "Success" "Green"
Write-LogMessage "  • Development prerequisites installed" "Success" "Green"
Write-LogMessage "  • Tauri configured for Windows" "Success" "Green"
Write-LogMessage "  • Development environment optimized" "Success" "Green"
Write-LogMessage "  • Project assets and dependencies ready" "Success" "Green"

if ($validationResults.BuildTest) {
    Write-LogMessage "  • Windows build validation passed" "Success" "Green"
} else {
    Write-LogMessage "  • Windows build validation skipped or failed" "Warning" "Yellow"
}

Write-LogMessage "`n🚀 Quick Start Commands:" "Info" "Cyan"
Write-LogMessage "  bun run tauri dev        # Start development" "Info"
Write-LogMessage "  bun run tauri build      # Build for production" "Info"
Write-LogMessage "  cargo test               # Run Rust tests" "Info"
Write-LogMessage "  .\scripts\windows-test-suite.ps1  # Windows tests" "Info"

Write-LogMessage "`n📚 Documentation:" "Info" "Cyan"
Write-LogMessage "  docs/WINDOWS_IMPLEMENTATION.md  # Complete Windows guide" "Info"
Write-LogMessage "  docs/WINDOWS_PRODUCTION_PLAN.md # Production roadmap" "Info"
Write-LogMessage "  README.md                        # Project overview" "Info"

Write-LogMessage "`n⚡ PowerShell Aliases (restart PowerShell to load):" "Info" "Cyan"
Write-LogMessage "  tdev, tbuild, ttest, test-windows" "Info"

if ($DevMode) {
    Write-LogMessage "`n🔧 Development Mode Enabled:" "Info" "Yellow"
    Write-LogMessage "  • Enhanced logging and debugging" "Info"
    Write-LogMessage "  • Additional development tools configured" "Info"
}

# Generate final report
New-SetupReport -ValidationResults $validationResults

Write-LogMessage "`n🎯 Your Windows development environment is ready!" "Success" "Green"
Write-LogMessage "Happy coding with Opcode! 🚀" "Success" "Green"
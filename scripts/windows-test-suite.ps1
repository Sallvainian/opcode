# Windows Comprehensive Test Suite Runner
# Executes all Windows-specific tests and generates detailed reports

param(
    [switch]$Full,              # Run full test suite including ignored tests
    [switch]$Performance,       # Run only performance tests
    [switch]$Security,          # Run only security tests
    [switch]$Build,             # Run only build validation tests
    [switch]$Installer,         # Run only installer tests (requires build)
    [switch]$Coverage,          # Generate coverage report
    [switch]$Verbose,           # Enable verbose output
    [string]$OutputDir = "test-reports"  # Output directory for reports
)

# Initialize script
$ErrorActionPreference = "Continue"
$StartTime = Get-Date
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$ProjectRoot = Split-Path -Parent $ScriptDir
$SrcTauriDir = Join-Path $ProjectRoot "src-tauri"

Write-Host "🧪 Windows Comprehensive Test Suite" -ForegroundColor Green
Write-Host "====================================" -ForegroundColor Green
Write-Host "Start Time: $StartTime" -ForegroundColor Cyan
Write-Host "Project Root: $ProjectRoot" -ForegroundColor Gray
Write-Host ""

# Ensure output directory exists
$ReportDir = Join-Path $ProjectRoot $OutputDir
if (!(Test-Path $ReportDir)) {
    New-Item -ItemType Directory -Path $ReportDir -Force | Out-Null
    Write-Host "📁 Created report directory: $ReportDir" -ForegroundColor Yellow
}

# Test execution results
$TestResults = @{
    Performance = @{ Passed = 0; Failed = 0; Ignored = 0; Duration = 0 }
    BuildValidation = @{ Passed = 0; Failed = 0; Ignored = 0; Duration = 0 }
    SecurityCompatibility = @{ Passed = 0; Failed = 0; Ignored = 0; Duration = 0 }
    Integration = @{ Passed = 0; Failed = 0; Ignored = 0; Duration = 0 }
    Installer = @{ Passed = 0; Failed = 0; Ignored = 0; Duration = 0 }
}

# Function to run test category
function Run-TestCategory {
    param(
        [string]$CategoryName,
        [string]$TestModule,
        [switch]$IgnoreFlag = $false,
        [switch]$Required = $true
    )

    Write-Host "🧪 Running $CategoryName Tests..." -ForegroundColor Yellow

    $TestStartTime = Get-Date

    # Build cargo test command
    $CargoArgs = @("test", $TestModule, "--target", "x86_64-pc-windows-msvc")
    if ($IgnoreFlag) {
        $CargoArgs += @("--", "--ignored")
    }
    if ($Verbose) {
        $CargoArgs += "--verbose"
    }

    # Execute tests
    $TestOutput = ""
    $TestError = ""

    try {
        Push-Location $SrcTauriDir

        $Process = Start-Process -FilePath "cargo" -ArgumentList $CargoArgs -Wait -NoNewWindow -PassThru -RedirectStandardOutput "$ReportDir\$TestModule-output.txt" -RedirectStandardError "$ReportDir\$TestModule-error.txt"

        $TestOutput = Get-Content "$ReportDir\$TestModule-output.txt" -Raw -ErrorAction SilentlyContinue
        $TestError = Get-Content "$ReportDir\$TestModule-error.txt" -Raw -ErrorAction SilentlyContinue

        $ExitCode = $Process.ExitCode
    }
    catch {
        Write-Host "❌ Error running $CategoryName tests: $($_.Exception.Message)" -ForegroundColor Red
        $ExitCode = 1
    }
    finally {
        Pop-Location
    }

    $TestDuration = ((Get-Date) - $TestStartTime).TotalSeconds

    # Parse test results
    $Passed = 0
    $Failed = 0
    $Ignored = 0

    if ($TestOutput) {
        # Parse "test result: ok. X passed; Y failed; Z ignored" format
        if ($TestOutput -match "(\d+) passed.*?(\d+) failed.*?(\d+) ignored") {
            $Passed = [int]$matches[1]
            $Failed = [int]$matches[2]
            $Ignored = [int]$matches[3]
        }
    }

    # Store results
    $TestResults[$CategoryName.Replace(" ", "")] = @{
        Passed = $Passed
        Failed = $Failed
        Ignored = $Ignored
        Duration = [math]::Round($TestDuration, 2)
        ExitCode = $ExitCode
        Output = $TestOutput
        Error = $TestError
    }

    # Report results
    if ($ExitCode -eq 0) {
        Write-Host "✅ $CategoryName completed: $Passed passed, $Failed failed, $Ignored ignored (${TestDuration}s)" -ForegroundColor Green
    } else {
        Write-Host "⚠️ $CategoryName had issues: $Passed passed, $Failed failed, $Ignored ignored (${TestDuration}s)" -ForegroundColor Yellow
        if ($TestError -and $Verbose) {
            Write-Host "Error output:" -ForegroundColor Red
            Write-Host $TestError -ForegroundColor Red
        }
    }

    return $ExitCode -eq 0
}

# Function to check prerequisites
function Test-Prerequisites {
    Write-Host "🔧 Checking prerequisites..." -ForegroundColor Yellow

    $Prerequisites = @(
        @{ Name = "Rust/Cargo"; Command = "cargo"; Args = @("--version") }
        @{ Name = "Windows Target"; Command = "rustup"; Args = @("target", "list", "--installed") }
        @{ Name = "PowerShell"; Command = "powershell"; Args = @("-Version") }
    )

    $AllGood = $true

    foreach ($Prereq in $Prerequisites) {
        try {
            $Result = & $Prereq.Command @($Prereq.Args) 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "✅ $($Prereq.Name): Available" -ForegroundColor Green

                if ($Prereq.Name -eq "Windows Target" -and $Result -notmatch "x86_64-pc-windows-msvc") {
                    Write-Host "⚠️ Installing Windows MSVC target..." -ForegroundColor Yellow
                    rustup target add x86_64-pc-windows-msvc
                }
            } else {
                Write-Host "❌ $($Prereq.Name): Not available" -ForegroundColor Red
                $AllGood = $false
            }
        }
        catch {
            Write-Host "❌ $($Prereq.Name): Not available" -ForegroundColor Red
            $AllGood = $false
        }
    }

    return $AllGood
}

# Function to generate comprehensive report
function Generate-Report {
    Write-Host ""
    Write-Host "📊 GENERATING COMPREHENSIVE REPORT" -ForegroundColor Cyan
    Write-Host "==================================" -ForegroundColor Cyan

    $TotalPassed = 0
    $TotalFailed = 0
    $TotalIgnored = 0
    $TotalDuration = 0

    # Calculate totals
    foreach ($Category in $TestResults.Keys) {
        $Result = $TestResults[$Category]
        $TotalPassed += $Result.Passed
        $TotalFailed += $Result.Failed
        $TotalIgnored += $Result.Ignored
        $TotalDuration += $Result.Duration
    }

    # Create detailed report
    $ReportContent = @"
# Windows Test Suite Report
Generated: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
Project: Opcode Windows Testing
Duration: $([math]::Round($TotalDuration, 2)) seconds

## Executive Summary
- **Total Tests**: $(($TotalPassed + $TotalFailed + $TotalIgnored))
- **Passed**: $TotalPassed ✅
- **Failed**: $TotalFailed $(if ($TotalFailed -gt 0) { "❌" } else { "" })
- **Ignored**: $TotalIgnored ⚠️
- **Success Rate**: $([math]::Round(($TotalPassed / ($TotalPassed + $TotalFailed)) * 100, 1))%

## Category Results

"@

    foreach ($Category in $TestResults.Keys | Sort-Object) {
        $Result = $TestResults[$Category]
        $ReportContent += @"

### $Category
- Passed: $($Result.Passed)
- Failed: $($Result.Failed)
- Ignored: $($Result.Ignored)
- Duration: $($Result.Duration)s
- Status: $(if ($Result.Failed -eq 0) { "✅ PASS" } else { "⚠️ ISSUES" })

"@
    }

    $ReportContent += @"

## Performance Metrics
- Average test execution time: $([math]::Round($TotalDuration / $TestResults.Count, 2))s per category
- Total execution time: $([math]::Round($TotalDuration, 2))s

## Recommendations
"@

    if ($TotalFailed -eq 0) {
        $ReportContent += "🎉 All tests passed! Windows platform is production-ready.`n"
    } else {
        $ReportContent += "⚠️ $TotalFailed tests failed. Review failures before deployment.`n"
    }

    # Save report
    $ReportPath = Join-Path $ReportDir "windows-test-report.md"
    $ReportContent | Out-File -FilePath $ReportPath -Encoding UTF8

    # Display summary
    Write-Host ""
    Write-Host "📈 FINAL RESULTS:" -ForegroundColor White
    Write-Host "  Total Passed:  $TotalPassed" -ForegroundColor Green
    Write-Host "  Total Failed:  $TotalFailed" $(if ($TotalFailed -gt 0) { -ForegroundColor Red } else { -ForegroundColor Green })
    Write-Host "  Total Ignored: $TotalIgnored" -ForegroundColor Yellow
    Write-Host "  Success Rate:  $([math]::Round(($TotalPassed / ($TotalPassed + $TotalFailed)) * 100, 1))%" -ForegroundColor Cyan
    Write-Host "  Total Time:    $([math]::Round($TotalDuration, 2))s" -ForegroundColor Gray
    Write-Host ""
    Write-Host "📄 Report saved: $ReportPath" -ForegroundColor Cyan
}

# Main execution logic
try {
    # Check prerequisites
    if (!(Test-Prerequisites)) {
        Write-Host "❌ Prerequisites not met. Please install required tools." -ForegroundColor Red
        exit 1
    }

    Write-Host ""

    # Determine which tests to run
    $RunAll = !($Performance -or $Security -or $Build -or $Installer)

    if ($Performance -or $RunAll) {
        Run-TestCategory -CategoryName "Performance" -TestModule "windows_performance"
    }

    if ($Build -or $RunAll) {
        Run-TestCategory -CategoryName "BuildValidation" -TestModule "windows_build_validation"
    }

    if ($Security -or $RunAll) {
        Run-TestCategory -CategoryName "SecurityCompatibility" -TestModule "windows_security_compatibility"
    }

    if ($RunAll) {
        Run-TestCategory -CategoryName "Integration" -TestModule "windows_integration"
    }

    if ($Installer -or $Full) {
        Write-Host "📦 Note: Installer tests require a full build and may take several minutes..." -ForegroundColor Yellow
        Run-TestCategory -CategoryName "Installer" -TestModule "windows_installer_validation" -IgnoreFlag -Required:$false
    }

    # Generate final report
    Generate-Report

    # Determine exit code
    $OverallSuccess = $true
    foreach ($Result in $TestResults.Values) {
        if ($Result.Failed -gt 0) {
            $OverallSuccess = $false
            break
        }
    }

    if ($OverallSuccess) {
        Write-Host "🎉 ALL TESTS PASSED!" -ForegroundColor Green
        exit 0
    } else {
        Write-Host "⚠️ Some tests failed. Check the report for details." -ForegroundColor Yellow
        exit 1
    }
}
catch {
    Write-Host "❌ Unexpected error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
finally {
    $EndTime = Get-Date
    $TotalScriptTime = ($EndTime - $StartTime).TotalMinutes
    Write-Host ""
    Write-Host "⏱️ Script execution completed in $([math]::Round($TotalScriptTime, 2)) minutes" -ForegroundColor Gray
}
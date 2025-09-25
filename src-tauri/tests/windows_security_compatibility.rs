//! Windows Security and Compatibility Tests
//!
//! Comprehensive security testing for Windows platform including:
//! - UAC and elevation testing
//! - Windows security features
//! - Antivirus compatibility
//! - Windows 10/11 compatibility testing

#[cfg(target_os = "windows")]
#[cfg(test)]
mod security_compatibility_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    // ===================================================================
    // SECURITY AND UAC TESTS
    // ===================================================================

    #[test]
    fn test_uac_elevation_detection() {
        println!("🛡️ Testing UAC and elevation detection");

        let script = r#"
            $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = New-Object System.Security.Principal.WindowsPrincipal($identity)
            $isAdmin = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
            $isUacEnabled = try {
                (Get-ItemProperty "HKLM:SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System" -Name "EnableLUA" -ErrorAction Stop).EnableLUA -eq 1
            } catch {
                $false
            }
            Write-Output "IsAdmin: $isAdmin"
            Write-Output "UacEnabled: $isUacEnabled"
            Write-Output "TokenElevationType: $([System.Security.Principal.WindowsIdentity]::GetCurrent().Name)"
        "#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output()
            .expect("Failed to execute PowerShell security check");

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);

            assert!(output_str.contains("IsAdmin:"), "Should report admin status");
            assert!(output_str.contains("UacEnabled:"), "Should report UAC status");

            // Parse and validate security context
            let lines: Vec<&str> = output_str.lines().collect();
            for line in lines {
                if line.starts_with("IsAdmin:") {
                    let is_admin = line.contains("True");
                    println!("🔒 Running as administrator: {}", is_admin);
                }
                if line.starts_with("UacEnabled:") {
                    let uac_enabled = line.contains("True");
                    println!("🛡️ UAC enabled: {}", uac_enabled);
                }
            }

            println!("✅ UAC detection successful");
        } else {
            println!("⚠️ UAC detection failed - may be restricted environment");
        }
    }

    #[test]
    fn test_windows_security_features() {
        println!("🔒 Testing Windows security features");

        // Test Windows Defender status
        let defender_output = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-MpComputerStatus | Select-Object AntivirusEnabled, RealTimeProtectionEnabled"])
            .output();

        match defender_output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                println!("🛡️ Windows Defender status: {}", output_str.trim());
            }
            _ => println!("⚠️ Cannot query Windows Defender status")
        }

        // Test Windows Firewall status
        let firewall_output = Command::new("netsh")
            .args(["advfirewall", "show", "currentprofile", "state"])
            .output();

        match firewall_output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("ON") {
                    println!("🔥 Windows Firewall: Active");
                } else {
                    println!("⚠️ Windows Firewall: Status unclear");
                }
            }
            _ => println!("⚠️ Cannot query Windows Firewall status")
        }

        println!("✅ Security features check completed");
    }

    #[test]
    fn test_file_system_permissions() {
        println!("📁 Testing file system permissions");

        let test_locations = vec![
            (std::env::temp_dir(), "Temp directory", true),
            (PathBuf::from(r"C:\Program Files"), "Program Files", false),
            (PathBuf::from(r"C:\Windows\System32"), "System32", false),
        ];

        for (location, description, should_write) in test_locations {
            if location.exists() {
                println!("📍 Testing {}: {}", description, location.display());

                let test_file = location.join("opcode_permission_test.txt");
                let write_result = fs::write(&test_file, b"permission test");

                match write_result {
                    Ok(_) => {
                        if should_write {
                            println!("✅ {} - Write access: Available", description);
                            let _ = fs::remove_file(&test_file); // Cleanup
                        } else {
                            println!("⚠️ {} - Unexpected write access (may indicate security issue)", description);
                            let _ = fs::remove_file(&test_file); // Cleanup
                        }
                    }
                    Err(_) => {
                        if should_write {
                            println!("⚠️ {} - Expected write access denied", description);
                        } else {
                            println!("✅ {} - Write access properly restricted", description);
                        }
                    }
                }
            }
        }

        println!("✅ File system permissions test completed");
    }

    #[test]
    fn test_antivirus_compatibility() {
        println!("🦠 Testing antivirus compatibility");

        let temp_dir = std::env::temp_dir();
        let test_exe = temp_dir.join("opcode_av_test.exe");

        // Create a small test executable (copy cmd.exe for testing)
        let cmd_exe = PathBuf::from(r"C:\Windows\System32\cmd.exe");

        if cmd_exe.exists() {
            match fs::copy(&cmd_exe, &test_exe) {
                Ok(_) => {
                    println!("📁 Created test executable: {}", test_exe.display());

                    // Test if we can execute it (antivirus might block)
                    let exec_result = Command::new(&test_exe)
                        .arg("/C")
                        .arg("echo AV_TEST")
                        .output();

                    match exec_result {
                        Ok(output) if output.status.success() => {
                            println!("✅ Test executable runs successfully (antivirus compatible)");
                        }
                        Ok(_) => {
                            println!("⚠️ Test executable execution issues");
                        }
                        Err(e) => {
                            println!("⚠️ Test executable blocked: {} (possible antivirus interference)", e);
                        }
                    }

                    // Cleanup
                    let _ = fs::remove_file(&test_exe);
                }
                Err(_) => {
                    println!("⚠️ Cannot create test executable for AV testing");
                }
            }
        }

        println!("✅ Antivirus compatibility test completed");
    }

    // ===================================================================
    // WINDOWS VERSION COMPATIBILITY TESTS
    // ===================================================================

    #[test]
    fn test_windows_version_compatibility() {
        println!("🪟 Testing Windows version compatibility");

        // Get Windows version information
        let version_script = r#"
            $version = [System.Environment]::OSVersion.Version
            $edition = (Get-WmiObject -Class Win32_OperatingSystem).Caption
            Write-Output "Version: $($version.Major).$($version.Minor).$($version.Build)"
            Write-Output "Edition: $edition"
        "#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", version_script])
            .output()
            .expect("Failed to get Windows version");

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            println!("📊 Windows version info:");

            for line in output_str.lines() {
                if line.starts_with("Version:") || line.starts_with("Edition:") {
                    println!("  {}", line.trim());
                }
            }

            // Check for Windows 10/11 compatibility
            if output_str.contains("Windows 10") || output_str.contains("Windows 11") {
                println!("✅ Running on supported Windows version");
            } else {
                println!("⚠️ Windows version may not be officially supported");
            }
        }

        println!("✅ Windows version compatibility check completed");
    }

    #[test]
    fn test_windows_feature_availability() {
        println!("🔧 Testing Windows feature availability");

        let features_to_test = vec![
            ("PowerShell", "powershell", vec!["-Version"]),
            ("Windows Management", "wmic", vec!["os", "get", "Caption"]),
            ("Network Tools", "ipconfig", vec!["/?"]),
            ("Task Management", "tasklist", vec!["/?"]),
            ("Registry Tools", "reg", vec!["/?"])
        ];

        let mut available_features = 0;

        for (name, command, args) in features_to_test {
            let test_result = Command::new(command)
                .args(args)
                .output();

            match test_result {
                Ok(output) if output.status.success() => {
                    println!("✅ {}: Available", name);
                    available_features += 1;
                }
                _ => {
                    println!("⚠️ {}: Not available or restricted", name);
                }
            }
        }

        assert!(available_features >= 3,
            "At least 3 Windows features should be available, got {}", available_features);

        println!("✅ Windows feature availability test completed");
    }

    #[test]
    fn test_windows_api_compatibility() {
        println!("🔌 Testing Windows API compatibility");

        // Test basic Windows API access through Rust std library
        let api_tests: Vec<(&str, Box<dyn Fn() -> bool>)> = vec![
            ("Current Directory", Box::new(|| std::env::current_dir().is_ok())),
            ("Environment Variables", Box::new(|| std::env::var("USERPROFILE").is_ok())),
            ("Temp Directory", Box::new(|| { let temp = std::env::temp_dir(); temp.exists() })),
            ("Process ID", Box::new(|| std::process::id() > 0)),
        ];

        let mut successful_tests = 0;

        for (name, test_fn) in api_tests {
            if test_fn() {
                println!("✅ {}: Working", name);
                successful_tests += 1;
            } else {
                println!("⚠️ {}: Failed", name);
            }
        }

        assert_eq!(successful_tests, 4, "All Windows API tests should pass");
        println!("✅ Windows API compatibility test completed");
    }

    #[test]
    fn test_windows_service_integration() {
        println!("⚙️ Testing Windows service integration");

        // Test Windows service enumeration
        let service_output = Command::new("sc")
            .args(["query", "state=", "running"])
            .output()
            .expect("Failed to query Windows services");

        if service_output.status.success() {
            let output_str = String::from_utf8_lossy(&service_output.stdout);
            let service_count = output_str.matches("SERVICE_NAME:").count();

            println!("📊 Found {} running Windows services", service_count);
            assert!(service_count > 10, "Should find multiple running services");

            // Check for essential services
            let essential_services = vec![
                "Themes",
                "AudioSrv",
                "Winmgmt"
            ];

            for service in essential_services {
                if output_str.to_lowercase().contains(&service.to_lowercase()) {
                    println!("✅ Essential service running: {}", service);
                }
            }
        }

        println!("✅ Windows service integration test completed");
    }

    #[test]
    fn test_windows_registry_security() {
        println!("📋 Testing Windows registry security");

        // Test reading safe registry keys
        let safe_keys = vec![
            (r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion", "ProductName"),
            (r"HKEY_CURRENT_USER\Environment", "TEMP"),
        ];

        let mut successful_reads = 0;

        for (key_path, value_name) in safe_keys {
            let reg_output = Command::new("reg")
                .args(["query", key_path, "/v", value_name])
                .output()
                .expect("Failed to execute registry query");

            if reg_output.status.success() {
                println!("✅ Registry read successful: {}", key_path);
                successful_reads += 1;
            } else {
                println!("⚠️ Registry read failed: {} (may be access restricted)", key_path);
            }
        }

        // Should be able to read at least one safe registry key
        assert!(successful_reads > 0, "Should be able to read some registry keys");
        println!("✅ Windows registry security test completed");
    }

    #[test]
    fn test_windows_security_boundaries() {
        println!("🔐 Testing Windows security boundaries");

        // Test that we cannot access restricted system files
        let restricted_paths = vec![
            r"C:\Windows\System32\config\SAM",
            r"C:\Windows\System32\config\SECURITY",
            r"C:\pagefile.sys",
        ];

        let mut properly_restricted = 0;

        for path in restricted_paths {
            let path_buf = PathBuf::from(path);

            match fs::read(&path_buf) {
                Ok(_) => {
                    println!("⚠️ Unexpected access to restricted file: {}", path);
                }
                Err(_) => {
                    println!("✅ Properly restricted access to: {}", path);
                    properly_restricted += 1;
                }
            }
        }

        assert!(properly_restricted >= 2,
            "Most restricted files should be inaccessible without elevation");

        println!("✅ Windows security boundaries test completed");
    }
}
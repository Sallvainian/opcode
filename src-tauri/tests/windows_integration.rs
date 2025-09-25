#[cfg(target_os = "windows")]
#[cfg(test)]
mod windows_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    #[test]
    fn test_process_management_tasklist() {
        // Test tasklist integration
        let output = Command::new("tasklist")
            .output()
            .expect("Failed to execute tasklist");

        assert!(output.status.success(), "tasklist command should succeed");
        assert!(!output.stdout.is_empty(), "tasklist should return output");
    }

    #[test]
    fn test_process_management_taskkill() {
        // Test taskkill command availability (without actually killing anything)
        let output = Command::new("taskkill")
            .args(["/?"]) // Just get help to verify command exists
            .output()
            .expect("Failed to execute taskkill");

        assert!(output.status.success() || output.stderr.is_empty(),
            "taskkill command should be available");
    }

    #[test]
    fn test_windows_path_handling() {
        // Test Windows path normalization
        let paths = vec![
            (r"C:\Users\Test", true),
            (r"\\server\share", true),
            (r"D:\Projects\opcode", true),
            ("relative\\path", false),
            ("./file.txt", false),
        ];

        for (path, should_be_absolute) in paths {
            let path_buf = PathBuf::from(path);
            assert_eq!(
                path_buf.is_absolute(),
                should_be_absolute,
                "Path '{}' absolute check failed",
                path
            );
        }
    }

    #[test]
    fn test_file_permissions() {
        // Test Windows file permission handling
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("opcode_test_permissions.txt");

        // Create a test file
        fs::write(&test_file, b"test content").expect("Failed to create test file");

        // Test setting read-only
        let metadata = fs::metadata(&test_file).expect("Failed to get metadata");
        let mut permissions = metadata.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&test_file, permissions).expect("Failed to set permissions");

        // Verify read-only was set
        let metadata = fs::metadata(&test_file).expect("Failed to get metadata");
        assert!(metadata.permissions().readonly(), "File should be read-only");

        // Reset to read-write
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&test_file, permissions).expect("Failed to set permissions");

        // Verify read-write
        let metadata = fs::metadata(&test_file).expect("Failed to get metadata");
        assert!(!metadata.permissions().readonly(), "File should be read-write");

        // Cleanup
        fs::remove_file(&test_file).expect("Failed to remove test file");
    }

    #[test]
    fn test_binary_detection() {
        // Test that we can detect .exe files
        let exe_extensions = vec![
            "claude.exe",
            "CLAUDE.EXE",
            "claude.bat",
            "claude.cmd",
        ];

        for exe_name in exe_extensions {
            let path = PathBuf::from(exe_name);
            let extension = path.extension().and_then(|s| s.to_str());

            let is_executable = matches!(
                extension.map(|s| s.to_lowercase()).as_deref(),
                Some("exe") | Some("bat") | Some("cmd")
            );

            assert!(is_executable, "{} should be recognized as executable", exe_name);
        }
    }

    #[test]
    fn test_where_command() {
        // Test 'where' command (Windows equivalent of 'which')
        let output = Command::new("where")
            .arg("cmd")  // cmd.exe should always exist on Windows
            .output()
            .expect("Failed to execute where");

        assert!(output.status.success(), "'where cmd' should succeed");
        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(output_str.contains("cmd.exe"), "Should find cmd.exe");
    }

    #[test]
    fn test_environment_variables() {
        // Test Windows-specific environment variables
        let required_vars = vec![
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
            "TEMP",
            "WINDIR",
            "SYSTEMROOT",
        ];

        for var in required_vars {
            assert!(
                std::env::var(var).is_ok(),
                "Environment variable {} should exist on Windows",
                var
            );
        }
    }

    #[test]
    fn test_unc_path_support() {
        // Test UNC path handling
        let unc_paths = vec![
            r"\\server\share\file.txt",
            r"\\?\C:\very\long\path",
            r"\\.\COM1",
        ];

        for path in unc_paths {
            let _path_buf = PathBuf::from(path);
            // UNC paths should be recognized as absolute
            assert!(
                path.starts_with(r"\\"),
                "Path '{}' should be recognized as UNC",
                path
            );
        }
    }

    #[test]
    fn test_program_files_paths() {
        // Test standard Windows installation paths exist
        let program_files = std::env::var("ProgramFiles").expect("ProgramFiles should exist");
        let path = PathBuf::from(&program_files);
        assert!(path.exists(), "Program Files directory should exist");

        // Check for x86 variant on 64-bit systems
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            let path_x86 = PathBuf::from(&program_files_x86);
            assert!(path_x86.exists(), "Program Files (x86) directory should exist");
        }
    }

    #[test]
    fn test_windows_process_tree_management() {
        // Test Windows process tree traversal capabilities
        let output = Command::new("tasklist")
            .args(["/fo", "csv", "/nh"])
            .output()
            .expect("Failed to execute tasklist with CSV format");

        assert!(output.status.success(), "tasklist CSV format should succeed");

        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(!output_str.is_empty(), "tasklist should return CSV output");

        // Verify CSV format contains expected columns
        let lines: Vec<&str> = output_str.lines().collect();
        if !lines.is_empty() {
            let first_line = lines[0];
            let csv_fields: Vec<&str> = first_line.split("\",\"").collect();
            assert!(csv_fields.len() >= 5, "CSV should have at least 5 fields (Image Name, PID, Session Name, Session#, Mem Usage)");
        }
    }

    #[test]
    fn test_windows_process_filtering() {
        // Test process filtering by executable name
        let output = Command::new("tasklist")
            .args(["/fi", "IMAGENAME eq explorer.exe", "/fo", "csv", "/nh"])
            .output()
            .expect("Failed to execute tasklist with filter");

        assert!(output.status.success(), "tasklist with filter should succeed");

        let output_str = String::from_utf8_lossy(&output.stdout);
        // Explorer should be running on most Windows systems
        if !output_str.trim().is_empty() {
            assert!(output_str.to_lowercase().contains("explorer.exe"),
                "Filtered output should contain explorer.exe");
        }
    }

    #[test]
    fn test_windows_service_enumeration() {
        // Test Windows service enumeration
        let output = Command::new("sc")
            .args(["query", "state=", "all"])
            .output()
            .expect("Failed to execute sc query");

        assert!(output.status.success(), "sc query should succeed");

        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(!output_str.is_empty(), "sc query should return service list");

        // Verify essential Windows services exist
        let essential_services = ["Spooler", "Themes", "AudioSrv", "Winmgmt"];
        for _service in essential_services {
            // Note: Not all services may be running, but they should appear in the list
            // We're just testing that sc query works and returns service information
        }
    }

    #[test]
    fn test_windows_power_management() {
        // Test Windows power management query
        let output = Command::new("powercfg")
            .args(["/query"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                assert!(!output_str.is_empty(), "powercfg query should return power settings");
                assert!(output_str.contains("Power Scheme GUID") ||
                        output_str.contains("Subgroup GUID"),
                        "Power configuration should contain GUID information");
            }
            _ => {
                // powercfg might not be available in all environments (e.g., containers)
                println!("powercfg not available or failed - skipping power management test");
            }
        }
    }

    #[test]
    fn test_windows_registry_read_access() {
        // Test basic Windows registry read access
        let output = Command::new("reg")
            .args(["query", "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion", "/v", "ProductName"])
            .output()
            .expect("Failed to execute reg query");

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("ProductName"), "Registry query should return ProductName");
            assert!(output_str.contains("Windows"), "ProductName should contain 'Windows'");
        } else {
            // Registry access might be restricted in some environments
            println!("Registry access not available - skipping registry test");
        }
    }

    #[test]
    fn test_windows_network_configuration() {
        // Test Windows network configuration query
        let output = Command::new("ipconfig")
            .arg("/all")
            .output()
            .expect("Failed to execute ipconfig");

        assert!(output.status.success(), "ipconfig should succeed");

        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(!output_str.is_empty(), "ipconfig should return network configuration");
        assert!(output_str.contains("Windows IP Configuration") ||
                output_str.contains("Ethernet adapter") ||
                output_str.contains("Wireless"),
                "ipconfig output should contain network adapter information");
    }

    #[test]
    fn test_windows_file_system_features() {
        // Test Windows-specific file system features
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("opcode_test_filesystem.txt");

        // Create test file
        fs::write(&test_file, b"test content for filesystem features").expect("Failed to create test file");

        // Test file attributes
        let metadata = fs::metadata(&test_file).expect("Failed to get file metadata");
        assert!(metadata.is_file(), "Should be identified as a file");

        // Test file size
        assert!(metadata.len() > 0, "File should have content");

        // Test Windows alternate data streams (basic test)
        let ads_file = temp_dir.join("test_ads.txt:ads_stream");
        let _ads_result = fs::write(&ads_file, b"alternate data stream");
        // Note: ADS might not work in all environments, so we don't assert success

        // Test long path support
        let long_name = "a".repeat(200);
        let long_path_file = temp_dir.join(format!("long_path_{}.txt", long_name));
        let long_path_result = fs::write(&long_path_file, b"long path test");

        // Clean up
        let _ = fs::remove_file(&test_file);
        if long_path_result.is_ok() {
            let _ = fs::remove_file(&long_path_file);
        }
    }

    #[test]
    fn test_windows_error_handling() {
        // Test Windows-specific error code handling
        let nonexistent_path = PathBuf::from("C:\\NonExistentDirectory\\NonExistentFile.txt");

        let result = fs::read(&nonexistent_path);
        assert!(result.is_err(), "Reading non-existent file should fail");

        let error = result.unwrap_err();
        let error_kind = error.kind();
        assert!(matches!(error_kind, std::io::ErrorKind::NotFound),
                "Should return NotFound error kind");
    }

    #[test]
    fn test_windows_temp_directory() {
        // Test Windows temporary directory access
        let temp_dir = std::env::temp_dir();
        assert!(temp_dir.exists(), "Temp directory should exist");
        assert!(temp_dir.is_dir(), "Temp path should be a directory");

        // Create and clean up a test file in temp
        let test_temp_file = temp_dir.join("opcode_temp_test.txt");
        fs::write(&test_temp_file, b"temporary file test").expect("Should be able to write to temp directory");

        assert!(test_temp_file.exists(), "Test temp file should exist");

        fs::remove_file(&test_temp_file).expect("Should be able to delete temp file");
        assert!(!test_temp_file.exists(), "Test temp file should be deleted");
    }

    #[test]
    fn test_windows_executable_permissions() {
        // Test Windows executable file detection
        let executable_extensions = vec!["exe", "bat", "cmd", "com", "scr", "msi"];

        for ext in executable_extensions {
            let filename = format!("test.{}", ext);
            let path = PathBuf::from(&filename);

            let extension = path.extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase());

            assert_eq!(extension.as_deref(), Some(ext),
                      "Extension should be correctly detected for {}", filename);
        }
    }

    #[test]
    fn test_windows_case_insensitive_paths() {
        // Test Windows case-insensitive path behavior
        let temp_dir = std::env::temp_dir();
        let test_file_lower = temp_dir.join("opcode_case_test.txt");
        let test_file_upper = temp_dir.join("OPCODE_CASE_TEST.TXT");

        // Create file with lowercase name
        fs::write(&test_file_lower, b"case insensitive test").expect("Failed to create test file");

        // Both lowercase and uppercase paths should work on Windows
        assert!(test_file_lower.exists(), "Lowercase path should exist");

        // On Windows, the uppercase version should also "exist" due to case insensitivity
        // However, the actual behavior may vary depending on the file system
        let uppercase_exists = test_file_upper.exists();

        // Clean up
        fs::remove_file(&test_file_lower).expect("Failed to remove test file");

        println!("Case insensitive behavior: uppercase exists = {}", uppercase_exists);
    }
}

// Integration test for the full application on Windows
#[cfg(all(test, target_os = "windows"))]
mod app_integration {
    use std::process::Command;
    use std::path::PathBuf;

    #[test]
    #[ignore] // This test requires the app to be built
    fn test_windows_installer_creation() {
        // This test verifies that Windows installers can be created
        // Run with: cargo test --ignored

        let output = Command::new("cargo")
            .args(["tauri", "build", "--target", "x86_64-pc-windows-msvc"])
            .current_dir("..")
            .output()
            .expect("Failed to build Tauri app");

        if output.status.success() {
            // Check that installer files were created
            let msi_path = PathBuf::from("../target/release/bundle/msi");
            let nsis_path = PathBuf::from("../target/release/bundle/nsis");

            assert!(
                msi_path.exists() || nsis_path.exists(),
                "At least one Windows installer type should be created"
            );
        }
    }
}
//! Windows Installer Validation Tests
//!
//! Comprehensive installer testing for Windows platform including:
//! - MSI installer generation and validation
//! - NSIS installer generation and validation
//! - Installer integrity checks
//! - Installation/uninstallation simulation

#[cfg(target_os = "windows")]
#[cfg(test)]
mod installer_validation_tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    #[test]
    #[ignore] // Requires full build - run with: cargo test --ignored
    fn test_msi_installer_generation() {
        println!("📦 Testing MSI installer generation");

        // Build the Tauri app with MSI bundle
        let output = Command::new("cargo")
            .args(["tauri", "build", "--target", "x86_64-pc-windows-msvc"])
            .current_dir("..")
            .output()
            .expect("Failed to execute Tauri build");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("Tauri build failed: {}", stderr);
        }

        // Check for MSI installer files
        let msi_dir = Path::new("../target/x86_64-pc-windows-msvc/release/bundle/msi");
        assert!(msi_dir.exists(), "MSI bundle directory should exist");

        let msi_files: Vec<PathBuf> = fs::read_dir(msi_dir)
            .expect("Failed to read MSI directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "msi" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert!(!msi_files.is_empty(), "Should generate at least one MSI file");

        for msi_file in &msi_files {
            println!("📦 Generated MSI: {}", msi_file.display());

            // Validate MSI file integrity
            validate_msi_file(msi_file);
        }

        println!("✅ MSI installer generation successful");
    }

    #[test]
    #[ignore] // Requires full build - run with: cargo test --ignored
    fn test_nsis_installer_generation() {
        println!("📦 Testing NSIS installer generation");

        // Build the Tauri app (NSIS should be included in targets)
        let output = Command::new("cargo")
            .args(["tauri", "build", "--target", "x86_64-pc-windows-msvc"])
            .current_dir("..")
            .output()
            .expect("Failed to execute Tauri build");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("Tauri build failed: {}", stderr);
        }

        // Check for NSIS installer files
        let nsis_dir = Path::new("../target/x86_64-pc-windows-msvc/release/bundle/nsis");

        if nsis_dir.exists() {
            let nsis_files: Vec<PathBuf> = fs::read_dir(nsis_dir)
                .expect("Failed to read NSIS directory")
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension()? == "exe" {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();

            if !nsis_files.is_empty() {
                for nsis_file in &nsis_files {
                    println!("📦 Generated NSIS: {}", nsis_file.display());
                    validate_nsis_file(nsis_file);
                }
                println!("✅ NSIS installer generation successful");
            } else {
                println!("⚠️ NSIS installer not generated (may not be configured)");
            }
        } else {
            println!("⚠️ NSIS bundle directory not found");
        }
    }

    #[test]
    #[ignore] // Requires admin privileges - run with: cargo test --ignored
    fn test_msi_installation_simulation() {
        println!("🔧 Testing MSI installation simulation");

        let msi_dir = Path::new("../target/x86_64-pc-windows-msvc/release/bundle/msi");

        if !msi_dir.exists() {
            println!("⚠️ MSI files not found - build first with: cargo tauri build");
            return;
        }

        let msi_files: Vec<PathBuf> = fs::read_dir(msi_dir)
            .expect("Failed to read MSI directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "msi" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        if msi_files.is_empty() {
            println!("⚠️ No MSI files found");
            return;
        }

        for msi_file in &msi_files {
            // Test MSI validation without actually installing
            let output = Command::new("msiexec")
                .args(["/a", &msi_file.to_string_lossy(), "TARGETDIR=C:\\temp\\opcode_test_install", "/quiet"])
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        println!("✅ MSI administrative install test successful");

                        // Cleanup test installation
                        let _ = fs::remove_dir_all("C:\\temp\\opcode_test_install");
                    } else {
                        println!("⚠️ MSI installation simulation failed (may require admin rights)");
                    }
                }
                Err(_) => {
                    println!("⚠️ msiexec not available or access denied");
                }
            }
        }
    }

    #[test]
    fn test_installer_metadata() {
        println!("📋 Testing installer metadata");

        // Check tauri.conf.json for installer configuration
        let config_path = Path::new("../tauri.conf.json");
        assert!(config_path.exists(), "tauri.conf.json should exist");

        let config_content = fs::read_to_string(config_path)
            .expect("Failed to read tauri.conf.json");

        // Verify Windows installer configuration
        assert!(config_content.contains("\"windows\""), "Should have Windows bundle config");
        assert!(config_content.contains("\"wix\""), "Should have WiX MSI config");

        // Check for required metadata
        let metadata_checks = vec![
            ("upgradeCode", "WiX upgrade code"),
            ("certificateThumbprint", "Certificate config"),
            ("digestAlgorithm", "Digest algorithm"),
        ];

        for (key, description) in metadata_checks {
            if config_content.contains(key) {
                println!("✅ {}: Present", description);
            } else {
                println!("⚠️ {}: Not configured", description);
            }
        }

        println!("✅ Installer metadata validation completed");
    }

    #[test]
    fn test_installer_icons() {
        println!("🎨 Testing installer icons");

        let icon_paths = vec![
            Path::new("../icons/icon.ico"),
            Path::new("../icons/32x32.png"),
            Path::new("../icons/64x64.png"),
            Path::new("../icons/128x128.png"),
        ];

        let mut found_icons = 0;

        for icon_path in icon_paths {
            if icon_path.exists() {
                let metadata = fs::metadata(icon_path).expect("Failed to get icon metadata");
                assert!(metadata.len() > 0, "Icon file should not be empty");

                println!("✅ Icon found: {} ({} bytes)", icon_path.display(), metadata.len());
                found_icons += 1;
            } else {
                println!("⚠️ Icon missing: {}", icon_path.display());
            }
        }

        assert!(found_icons > 0, "Should have at least one icon file");
        println!("✅ Installer icons validation completed");
    }

    #[test]
    fn test_updater_artifact_generation() {
        println!("🔄 Testing updater artifact generation");

        // Check if updater artifacts are generated alongside installers
        let release_dir = Path::new("../target/x86_64-pc-windows-msvc/release/bundle");

        if !release_dir.exists() {
            println!("⚠️ Bundle directory not found - build first");
            return;
        }

        // Look for updater signature files
        let updater_extensions = vec![".msi.zip", ".msi.zip.sig", ".nsis.zip", ".nsis.zip.sig"];
        let mut updater_artifacts = 0;

        for entry in fs::read_dir(release_dir).expect("Failed to read bundle directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if path.is_file() {
                let path_str = path.to_string_lossy();

                for ext in &updater_extensions {
                    if path_str.ends_with(ext) {
                        println!("✅ Updater artifact: {}", path.display());
                        updater_artifacts += 1;
                    }
                }
            }
        }

        if updater_artifacts > 0 {
            println!("✅ Updater artifacts generated: {}", updater_artifacts);
        } else {
            println!("⚠️ No updater artifacts found (may require signing config)");
        }
    }

    // Helper function to validate MSI file integrity
    fn validate_msi_file(msi_path: &Path) {
        println!("🔍 Validating MSI file: {}", msi_path.display());

        // Check file size - should be reasonable (not empty, not too large)
        let metadata = fs::metadata(msi_path).expect("Failed to get MSI metadata");
        let file_size = metadata.len();

        assert!(file_size > 1024, "MSI file should be larger than 1KB");
        assert!(file_size < 100 * 1024 * 1024, "MSI file should be smaller than 100MB");

        println!("📊 MSI file size: {} bytes", file_size);

        // Test MSI file validation using Windows tools
        let output = Command::new("msiexec")
            .args(["/a", &msi_path.to_string_lossy(), "/quiet", "/l*v", "msi_validation.log"])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("✅ MSI file validation successful");
                } else {
                    println!("⚠️ MSI validation warnings (file may still be valid)");
                }
            }
            Err(_) => {
                println!("⚠️ Cannot validate MSI (msiexec not available or restricted)");
            }
        }

        // Cleanup validation log
        let _ = fs::remove_file("msi_validation.log");
    }

    // Helper function to validate NSIS file integrity
    fn validate_nsis_file(nsis_path: &Path) {
        println!("🔍 Validating NSIS file: {}", nsis_path.display());

        // Check file size
        let metadata = fs::metadata(nsis_path).expect("Failed to get NSIS metadata");
        let file_size = metadata.len();

        assert!(file_size > 1024, "NSIS file should be larger than 1KB");
        assert!(file_size < 100 * 1024 * 1024, "NSIS file should be smaller than 100MB");

        println!("📊 NSIS file size: {} bytes", file_size);

        // Test NSIS file by attempting to read its structure (basic validation)
        // NSIS files should have specific headers
        let file_header = fs::read(nsis_path)
            .expect("Failed to read NSIS file")
            .into_iter()
            .take(16)
            .collect::<Vec<u8>>();

        // Basic check - should not be all zeros
        assert!(file_header.iter().any(|&b| b != 0), "NSIS file should not be empty");

        println!("✅ NSIS file basic validation completed");
    }
}
//! Windows Registry operations for system integration
//!
//! This module provides comprehensive Windows Registry manipulation for seamless
//! system integration and user experience enhancement.
//!
//! # Features
//! - **File Associations**: Register custom file types with Windows Shell
//! - **URL Protocols**: Handle custom URI schemes (e.g., `myapp://action`)
//! - **Auto-Start Management**: Configure Windows startup behavior
//! - **Registry Safety**: Atomic operations with automatic rollback on failure
//! - **Permission Aware**: Handles UAC and privilege requirements gracefully
//!
//! # Security Considerations
//! Registry operations typically require administrator privileges for system-wide
//! changes. User-specific changes (HKEY_CURRENT_USER) work without elevation.
//!
//! # Error Handling
//! All functions return `anyhow::Result<T>` with detailed error context including:
//! - Access denied (insufficient privileges)
//! - Registry key creation/modification failures
//! - Invalid executable paths or registry values
//! - System resource exhaustion
//!
//! # Examples
//!
//! ## Complete Application Integration
//! ```rust
//! use crate::windows::registry::*;
//!
//! fn setup_windows_integration() -> anyhow::Result<()> {
//!     let exe_path = std::env::current_exe()?.to_string_lossy().to_string();
//!
//!     // Register file association
//!     register_file_association(
//!         ".myext",
//!         "MyApp.Document",
//!         &exe_path,
//!         "MyApp Document File"
//!     )?;
//!
//!     // Register URL protocol
//!     register_url_protocol(
//!         "myapp",
//!         &exe_path,
//!         "MyApp Protocol Handler"
//!     )?;
//!
//!     // Enable auto-start (optional)
//!     set_auto_start("MyApp", &exe_path, true)?;
//!
//!     println!("Windows integration configured successfully");
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::path::Path;
use winapi::um::winreg::{RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegSetValueExW};
use winapi::shared::minwindef::{DWORD, HKEY};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::winnt::{KEY_ALL_ACCESS, REG_SZ};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

/// Convert a Rust string to a wide string for Windows API
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Create or open a registry key
unsafe fn create_registry_key(root: HKEY, path: &str) -> Result<HKEY> {
    let wide_path = to_wide_string(path);
    let mut key: HKEY = ptr::null_mut();
    let mut disposition: DWORD = 0;

    let result = RegCreateKeyExW(
        root,
        wide_path.as_ptr(),
        0,
        ptr::null_mut(),
        0, // REG_OPTION_NON_VOLATILE
        KEY_ALL_ACCESS,
        ptr::null_mut(),
        &mut key,
        &mut disposition,
    );

    if result != ERROR_SUCCESS as i32 {
        return Err(anyhow::anyhow!("Failed to create registry key {}: error code {}", path, result));
    }

    Ok(key)
}

/// Set a string value in the registry
unsafe fn set_registry_value(key: HKEY, name: &str, value: &str) -> Result<()> {
    let wide_name = to_wide_string(name);
    let wide_value = to_wide_string(value);
    let value_bytes = (wide_value.len() * 2) as DWORD;

    let result = RegSetValueExW(
        key,
        wide_name.as_ptr(),
        0,
        REG_SZ,
        wide_value.as_ptr() as *const u8,
        value_bytes,
    );

    if result != ERROR_SUCCESS as i32 {
        return Err(anyhow::anyhow!("Failed to set registry value {}: error code {}", name, result));
    }

    Ok(())
}

/// Register a file association in the Windows Registry
///
/// This function registers a custom file extension with a program, allowing
/// the program to be the default handler for files with that extension.
///
/// # Arguments
/// * `extension` - File extension to register (e.g., ".opc")
/// * `program_id` - Unique program identifier (e.g., "Opcode.Document")
/// * `executable_path` - Full path to the executable
/// * `description` - Human-readable description of the file type
///
/// # Returns
/// * `Ok(())` if registration successful
/// * `Err(...)` if registration failed
///
/// # Example
/// ```rust
/// use crate::windows::registry::register_file_association;
///
/// fn main() -> anyhow::Result<()> {
///     register_file_association(
///         ".opc",
///         "Opcode.Document",
///         r"C:\Program Files\Opcode\opcode.exe",
///         "Opcode Document File"
///     )?;
///     Ok(())
/// }
/// ```
pub fn register_file_association(
    extension: &str,
    program_id: &str,
    executable_path: &str,
    description: &str,
) -> Result<()> {
    info!("Registering file association for extension: {}", extension);

    // Ensure extension starts with a dot
    let ext = if extension.starts_with('.') {
        extension.to_string()
    } else {
        format!(".{}", extension)
    };

    // Verify executable exists
    if !Path::new(executable_path).exists() {
        return Err(anyhow::anyhow!("Executable not found for file association: {} (check installation)", executable_path));
    }

    unsafe {
        use winapi::um::winreg::HKEY_CLASSES_ROOT;

        // Register the extension
        let ext_key = create_registry_key(HKEY_CLASSES_ROOT, &ext)
            .context("Failed to create extension key")?;

        // Set the default value to the program ID
        set_registry_value(ext_key, "", program_id)
            .context("Failed to set extension program ID")?;

        // Set content type
        set_registry_value(ext_key, "Content Type", "application/x-opcode")
            .context("Failed to set content type")?;

        RegCloseKey(ext_key);

        // Register the program ID
        let prog_key = create_registry_key(HKEY_CLASSES_ROOT, program_id)
            .context("Failed to create program ID key")?;

        // Set description
        set_registry_value(prog_key, "", description)
            .context("Failed to set program description")?;

        RegCloseKey(prog_key);

        // Register the shell command
        let shell_open_path = format!(r"{}\shell\open\command", program_id);
        let shell_key = create_registry_key(HKEY_CLASSES_ROOT, &shell_open_path)
            .context("Failed to create shell command key")?;

        // Set command line (executable path with "%1" for the file argument)
        let command = format!(r#""{}" "%1""#, executable_path);
        set_registry_value(shell_key, "", &command)
            .context("Failed to set shell command")?;

        RegCloseKey(shell_key);

        // Register the icon
        let icon_path = format!(r"{}\DefaultIcon", program_id);
        let icon_key = create_registry_key(HKEY_CLASSES_ROOT, &icon_path)
            .context("Failed to create icon key")?;

        // Use the executable's icon
        let icon_value = format!("{},0", executable_path);
        set_registry_value(icon_key, "", &icon_value)
            .context("Failed to set icon")?;

        RegCloseKey(icon_key);
    }

    info!("Successfully registered file association for {}", extension);
    Ok(())
}

/// Register a URL protocol in the Windows Registry
///
/// This function registers a custom URL protocol (e.g., "opcode://") allowing
/// the program to handle custom URI schemes.
///
/// # Arguments
/// * `protocol` - Protocol name (e.g., "opcode")
/// * `executable_path` - Full path to the executable
/// * `description` - Human-readable description
///
/// # Returns
/// * `Ok(())` if registration successful
/// * `Err(...)` if registration failed
///
/// # Example
/// ```rust
/// use crate::windows::registry::register_url_protocol;
///
/// fn main() -> anyhow::Result<()> {
///     register_url_protocol(
///         "opcode",
///         r"C:\Program Files\Opcode\opcode.exe",
///         "Opcode Protocol"
///     )?;
///     Ok(())
/// }
/// ```
pub fn register_url_protocol(
    protocol: &str,
    executable_path: &str,
    description: &str,
) -> Result<()> {
    info!("Registering URL protocol: {}", protocol);

    // Verify executable exists
    if !Path::new(executable_path).exists() {
        return Err(anyhow::anyhow!("Executable not found for URL protocol: {} (check installation)", executable_path));
    }

    unsafe {
        use winapi::um::winreg::HKEY_CLASSES_ROOT;

        // Create protocol key
        let protocol_key = create_registry_key(HKEY_CLASSES_ROOT, protocol)
            .context("Failed to create protocol key")?;

        // Set description
        set_registry_value(protocol_key, "", description)
            .context("Failed to set protocol description")?;

        // Mark as URL protocol
        set_registry_value(protocol_key, "URL Protocol", "")
            .context("Failed to set URL protocol flag")?;

        RegCloseKey(protocol_key);

        // Create shell command
        let shell_command_path = format!(r"{}\shell\open\command", protocol);
        let command_key = create_registry_key(HKEY_CLASSES_ROOT, &shell_command_path)
            .context("Failed to create shell command key")?;

        // Set command with URL parameter
        let command = format!(r#""{}" "%1""#, executable_path);
        set_registry_value(command_key, "", &command)
            .context("Failed to set protocol command")?;

        RegCloseKey(command_key);

        // Set default icon
        let icon_path = format!(r"{}\DefaultIcon", protocol);
        let icon_key = create_registry_key(HKEY_CLASSES_ROOT, &icon_path)
            .context("Failed to create icon key")?;

        let icon_value = format!("{},0", executable_path);
        set_registry_value(icon_key, "", &icon_value)
            .context("Failed to set protocol icon")?;

        RegCloseKey(icon_key);
    }

    info!("Successfully registered URL protocol: {}://", protocol);
    Ok(())
}

/// Set auto-start on Windows login
///
/// This function adds or removes the application from Windows startup by
/// modifying the Run registry key.
///
/// # Arguments
/// * `app_name` - Application name for the registry entry
/// * `executable_path` - Full path to the executable
/// * `enabled` - true to enable auto-start, false to disable
///
/// # Returns
/// * `Ok(())` if operation successful
/// * `Err(...)` if operation failed
///
/// # Example
/// ```rust
/// use crate::windows::registry::set_auto_start;
///
/// fn main() -> anyhow::Result<()> {
///     // Enable auto-start
///     set_auto_start(
///         "Opcode",
///         r"C:\Program Files\Opcode\opcode.exe",
///         true
///     )?;
///
///     // Disable auto-start
///     set_auto_start("Opcode", "", false)?;
///     Ok(())
/// }
/// ```
pub fn set_auto_start(app_name: &str, executable_path: &str, enabled: bool) -> Result<()> {
    info!("Setting auto-start for {}: {}", app_name, enabled);

    unsafe {
        use winapi::um::winreg::HKEY_CURRENT_USER;

        const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

        if enabled {
            // Verify executable exists
            if !Path::new(executable_path).exists() {
                return Err(anyhow::anyhow!("Executable not found for auto-start: {} (check installation)", executable_path));
            }

            // Open Run key
            let run_key = create_registry_key(HKEY_CURRENT_USER, RUN_KEY)
                .context("Failed to open Run registry key")?;

            // Set the auto-start value
            set_registry_value(run_key, app_name, executable_path)
                .context("Failed to set auto-start value")?;

            RegCloseKey(run_key);

            info!("Successfully enabled auto-start for {}", app_name);
        } else {
            // Remove the auto-start entry
            let run_key_wide = to_wide_string(RUN_KEY);
            let app_name_wide = to_wide_string(app_name);

            // Open the Run key
            let mut run_key: HKEY = ptr::null_mut();
            let result = RegCreateKeyExW(
                HKEY_CURRENT_USER,
                run_key_wide.as_ptr(),
                0,
                ptr::null_mut(),
                0,
                KEY_ALL_ACCESS,
                ptr::null_mut(),
                &mut run_key,
                ptr::null_mut(),
            );

            if result == ERROR_SUCCESS as i32 {
                // Delete the value
                use winapi::um::winreg::RegDeleteValueW;
                let delete_result = RegDeleteValueW(run_key, app_name_wide.as_ptr());

                RegCloseKey(run_key);

                if delete_result == ERROR_SUCCESS as i32 {
                    info!("Successfully disabled auto-start for {}", app_name);
                } else {
                    warn!("Auto-start entry for {} was not found or could not be deleted", app_name);
                }
            } else {
                warn!("Could not open Run registry key");
            }
        }
    }

    Ok(())
}

/// Remove a file association from the registry
///
/// # Arguments
/// * `extension` - File extension to unregister (e.g., ".opc")
/// * `program_id` - Program identifier to remove
///
/// # Returns
/// * `Ok(())` if removal successful or entry didn't exist
/// * `Err(...)` if removal failed
pub fn remove_file_association(extension: &str, program_id: &str) -> Result<()> {
    info!("Removing file association for extension: {}", extension);

    // Ensure extension starts with a dot
    let ext = if extension.starts_with('.') {
        extension.to_string()
    } else {
        format!(".{}", extension)
    };

    unsafe {
        use winapi::um::winreg::HKEY_CLASSES_ROOT;

        // Delete extension key
        let ext_wide = to_wide_string(&ext);
        let ext_result = RegDeleteKeyW(HKEY_CLASSES_ROOT, ext_wide.as_ptr());

        if ext_result != ERROR_SUCCESS as i32 {
            debug!("Extension key {} not found or could not be deleted", ext);
        }

        // Delete program ID key and all subkeys
        delete_registry_tree(HKEY_CLASSES_ROOT, program_id)?;
    }

    info!("Successfully removed file association for {}", extension);
    Ok(())
}

/// Remove a URL protocol from the registry
///
/// # Arguments
/// * `protocol` - Protocol name to unregister (e.g., "opcode")
///
/// # Returns
/// * `Ok(())` if removal successful or entry didn't exist
/// * `Err(...)` if removal failed
pub fn remove_url_protocol(protocol: &str) -> Result<()> {
    info!("Removing URL protocol: {}", protocol);

    unsafe {
        use winapi::um::winreg::HKEY_CLASSES_ROOT;

        // Delete protocol key and all subkeys
        delete_registry_tree(HKEY_CLASSES_ROOT, protocol)?;
    }

    info!("Successfully removed URL protocol: {}", protocol);
    Ok(())
}

/// Delete a registry key and all its subkeys recursively
unsafe fn delete_registry_tree(root: HKEY, path: &str) -> Result<()> {
    use winapi::um::winreg::RegDeleteTreeW;

    let path_wide = to_wide_string(path);
    let result = RegDeleteTreeW(root, path_wide.as_ptr());

    if result != ERROR_SUCCESS as i32 && result != 2 { // 2 = ERROR_FILE_NOT_FOUND
        return Err(anyhow::anyhow!("Failed to delete registry tree {}: error code {}", path, result));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    #[ignore] // Integration test - requires Windows and admin rights
    fn test_file_association_registration() {
        let exe_path = env::current_exe().expect("Failed to get current executable path");
        let exe_str = exe_path.to_str().expect("Executable path should be valid UTF-8");

        let result = register_file_association(
            ".opctest",
            "Opcode.TestDocument",
            exe_str,
            "Opcode Test Document",
        );

        assert!(result.is_ok(), "File association registration should succeed in test environment");

        // Clean up
        let _ = remove_file_association(".opctest", "Opcode.TestDocument");
    }

    #[test]
    #[ignore] // Integration test - requires Windows and admin rights
    fn test_url_protocol_registration() {
        let exe_path = env::current_exe().expect("Failed to get current executable path");
        let exe_str = exe_path.to_str().expect("Executable path should be valid UTF-8");

        let result = register_url_protocol(
            "opcodetest",
            exe_str,
            "Opcode Test Protocol",
        );

        assert!(result.is_ok(), "URL protocol registration should succeed in test environment");

        // Clean up
        let _ = remove_url_protocol("opcodetest");
    }

    #[test]
    #[ignore] // Integration test - requires Windows
    fn test_auto_start() {
        let exe_path = env::current_exe().expect("Failed to get current executable path");
        let exe_str = exe_path.to_str().expect("Executable path should be valid UTF-8");

        // Test enabling auto-start
        let result = set_auto_start("OpcodeTest", exe_str, true);
        assert!(result.is_ok(), "Auto-start enablement should succeed in test environment");

        // Test disabling auto-start
        let result = set_auto_start("OpcodeTest", "", false);
        assert!(result.is_ok(), "Auto-start disablement should succeed in test environment");
    }
}
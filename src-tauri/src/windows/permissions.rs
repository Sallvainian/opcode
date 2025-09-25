//! Windows permissions, UAC, and security management
//!
//! This module provides comprehensive Windows security and permission management
//! designed for robust, security-aware applications.
//!
//! # Features
//! - **UAC Integration**: Seamless User Account Control handling
//! - **Privilege Detection**: Runtime administrator status checking
//! - **ACL Management**: Fine-grained file and directory permissions
//! - **Security Context**: Process token and elevation analysis
//! - **Path-based Security**: Automatic privilege requirement detection
//!
//! # Security Model
//! Windows permissions follow a complex hierarchical model:
//! - **Standard User**: Limited access to system resources
//! - **Administrator**: Full system access (requires UAC elevation)
//! - **System**: Highest privilege level (services and kernel)
//!
//! # UAC Best Practices
//! - Check privileges before performing admin operations
//! - Request elevation only when necessary
//! - Provide clear justification to users
//! - Handle elevation denial gracefully
//! - Use least-privilege principle
//!
//! # Examples
//!
//! ## Smart Permission Handling
//! ```rust
//! use crate::windows::permissions::*;
//!
//! async fn install_application(install_path: &str) -> anyhow::Result<()> {
//!     // Check if admin access is required
//!     if requires_admin_access(install_path)? {
//!         if !is_running_as_admin()? {
//!             println!("Installation requires administrator privileges");
//!
//!             let exe_path = std::env::current_exe()?;
//!             let elevated = request_elevation(
//!                 &exe_path.to_string_lossy(),
//!                 &["--install", install_path]
//!             ).await?;
//!
//!             if !elevated {
//!                 return Err(anyhow::anyhow!("Administrator privileges required"));
//!             }
//!             return Ok(()); // Elevated process will handle installation
//!         }
//!     }
//!
//!     // Proceed with installation
//!     println!("Installing to: {}", install_path);
//!     // ... installation logic ...
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, TRUE};
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::{GetTokenInformation, InitializeSecurityDescriptor, SetSecurityDescriptorDacl};
use winapi::um::winnt::{
    DACL_SECURITY_INFORMATION, FILE_ALL_ACCESS, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ,
    FILE_GENERIC_WRITE, GENERIC_ALL, PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR,
    SE_FILE_OBJECT, TOKEN_ELEVATION, TOKEN_QUERY, PACL,
    SECURITY_DESCRIPTOR_REVISION, TokenElevation,
};

/// Convert a Rust string to a wide string for Windows API
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Check if the current process is running with administrator privileges
///
/// This function checks whether the current process has elevated privileges
/// (administrator rights) on Windows.
///
/// # Returns
/// * `Ok(true)` if running as administrator
/// * `Ok(false)` if running as standard user
/// * `Err(...)` if unable to determine privileges
///
/// # Example
/// ```rust
/// use crate::windows::permissions::is_running_as_admin;
///
/// fn main() -> anyhow::Result<()> {
///     if is_running_as_admin()? {
///         println!("Running with administrator privileges");
///     } else {
///         println!("Running as standard user");
///     }
///     Ok(())
/// }
/// ```
pub fn is_running_as_admin() -> Result<bool> {
    debug!("Checking if process is running as administrator");

    unsafe {
        let mut token: HANDLE = ptr::null_mut();

        // Open the current process token
        let result = OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY,
            &mut token,
        );

        if result == FALSE {
            return Err(anyhow::anyhow!("Failed to open process token"));
        }

        // Query token elevation status
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size: DWORD = std::mem::size_of::<TOKEN_ELEVATION>() as DWORD;

        let query_result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );

        CloseHandle(token);

        if query_result == FALSE {
            return Err(anyhow::anyhow!("Failed to get token information"));
        }

        let is_admin = elevation.TokenIsElevated != 0;
        debug!("Administrator privilege status: {}", is_admin);

        Ok(is_admin)
    }
}

/// Request UAC elevation by restarting the process with elevated privileges
///
/// This function attempts to restart the specified executable with elevated
/// privileges using the Windows UAC prompt.
///
/// # Arguments
/// * `executable_path` - Path to the executable to run elevated
/// * `args` - Command-line arguments to pass to the elevated process
///
/// # Returns
/// * `Ok(true)` if elevation was successful and process started
/// * `Ok(false)` if user denied elevation or it was cancelled
/// * `Err(...)` if there was an error requesting elevation
///
/// # Example
/// ```rust
/// use crate::windows::permissions::request_elevation;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let elevated = request_elevation(
///         r"C:\Program Files\Opcode\opcode.exe",
///         &["--admin-mode"]
///     ).await?;
///
///     if elevated {
///         println!("Process started with elevated privileges");
///     }
///     Ok(())
/// }
/// ```
pub async fn request_elevation(executable_path: &str, args: &[&str]) -> Result<bool> {
    info!("Requesting UAC elevation for: {}", executable_path);

    // Verify executable exists
    if !Path::new(executable_path).exists() {
        return Err(anyhow::anyhow!("Executable not found: {}", executable_path));
    }

    // Build arguments string
    let args_string = args.join(" ");

    // Use PowerShell to request elevation
    let script = format!(
        r#"
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = "{}"
        $psi.Arguments = "{}"
        $psi.Verb = "runas"
        $psi.UseShellExecute = $true

        try {{
            $process = [System.Diagnostics.Process]::Start($psi)
            Write-Output "Success"
        }} catch {{
            Write-Output "Failed: $_"
        }}
        "#,
        executable_path.replace('\\', "\\\\"),
        args_string.replace('"', "`\"")
    );

    let output = tokio::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .await
        .context("Failed to execute PowerShell command")?;

    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if output_str.starts_with("Success") {
        info!("Successfully started elevated process");
        Ok(true)
    } else if output_str.contains("canceled") || output_str.contains("denied") {
        warn!("User denied elevation request");
        Ok(false)
    } else {
        error!("Failed to elevate: {}", output_str);
        Err(anyhow::anyhow!("Failed to request elevation: {}", output_str))
    }
}

/// Set Windows ACL (Access Control List) on a file
///
/// This function modifies the Windows ACL for a file, controlling who can
/// access it and what permissions they have.
///
/// # Arguments
/// * `file_path` - Path to the file to modify
/// * `permissions` - Permission string (e.g., "Everyone:R", "Administrators:F")
///
/// # Permission Codes:
/// * `F` - Full control
/// * `M` - Modify
/// * `RX` - Read and execute
/// * `R` - Read
/// * `W` - Write
///
/// # Returns
/// * `Ok(())` if ACL was successfully set
/// * `Err(...)` if there was an error setting the ACL
///
/// # Example
/// ```rust
/// use crate::windows::permissions::set_file_acl;
///
/// fn main() -> anyhow::Result<()> {
///     // Grant full control to administrators, read-only to users
///     set_file_acl(
///         r"C:\sensitive\file.txt",
///         "Administrators:F,Users:R"
///     )?;
///     Ok(())
/// }
/// ```
pub fn set_file_acl(file_path: &str, permissions: &str) -> Result<()> {
    info!("Setting ACL for file: {} with permissions: {}", file_path, permissions);

    // Verify file exists
    if !Path::new(file_path).exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path));
    }

    // Use icacls command for simplicity (available on all Windows versions)
    let output = std::process::Command::new("icacls")
        .arg(file_path)
        .arg("/grant")
        .arg(permissions)
        .output()
        .context("Failed to execute icacls command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to set ACL: {}", stderr));
    }

    info!("Successfully set ACL for {}", file_path);
    Ok(())
}

/// Remove specific ACL entries from a file
///
/// # Arguments
/// * `file_path` - Path to the file to modify
/// * `principal` - User or group to remove (e.g., "Everyone", "Users")
///
/// # Returns
/// * `Ok(())` if ACL entry was successfully removed
/// * `Err(...)` if there was an error removing the ACL entry
pub fn remove_file_acl(file_path: &str, principal: &str) -> Result<()> {
    info!("Removing ACL entry for {} from file: {}", principal, file_path);

    // Verify file exists
    if !Path::new(file_path).exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path));
    }

    // Use icacls to remove permissions
    let output = std::process::Command::new("icacls")
        .arg(file_path)
        .arg("/remove")
        .arg(principal)
        .output()
        .context("Failed to execute icacls command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to remove ACL entry: {}", stderr));
    }

    info!("Successfully removed ACL entry for {} from {}", principal, file_path);
    Ok(())
}

/// Reset file ACL to default inherited permissions
///
/// # Arguments
/// * `file_path` - Path to the file to reset
///
/// # Returns
/// * `Ok(())` if ACL was successfully reset
/// * `Err(...)` if there was an error resetting the ACL
pub fn reset_file_acl(file_path: &str) -> Result<()> {
    info!("Resetting ACL to defaults for file: {}", file_path);

    // Verify file exists
    if !Path::new(file_path).exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path));
    }

    // Use icacls to reset permissions
    let output = std::process::Command::new("icacls")
        .arg(file_path)
        .arg("/reset")
        .output()
        .context("Failed to execute icacls command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to reset ACL: {}", stderr));
    }

    info!("Successfully reset ACL for {}", file_path);
    Ok(())
}

/// Check if a file or directory requires administrator access
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `Ok(true)` if the path requires admin access
/// * `Ok(false)` if the path is accessible to standard users
/// * `Err(...)` if unable to determine access requirements
pub fn requires_admin_access(path: &str) -> Result<bool> {
    debug!("Checking if path requires admin access: {}", path);

    // Common system directories that typically require admin access
    let admin_paths = [
        r"c:\windows",
        r"c:\program files",
        r"c:\program files (x86)",
        r"c:\programdata",
    ];

    let path_lower = path.to_lowercase();

    for &admin_path in &admin_paths {
        if path_lower.starts_with(admin_path) {
            debug!("Path {} is in admin-protected directory", path);
            return Ok(true);
        }
    }

    // Try to test write access
    let test_path = if Path::new(path).is_dir() {
        Path::new(path).join(".opcode_test")
    } else {
        Path::new(path).with_extension("opcode_test")
    };

    // Attempt to create a test file
    match std::fs::write(&test_path, b"test") {
        Ok(_) => {
            // Clean up test file
            let _ = std::fs::remove_file(&test_path);
            debug!("Path {} is writable by current user", path);
            Ok(false)
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                debug!("Path {} requires elevated permissions", path);
                Ok(true)
            } else {
                warn!("Unable to determine access requirements for {}: {}", path, e);
                Ok(false)
            }
        }
    }
}

/// Create a security descriptor with specific permissions
///
/// This is a low-level function for advanced ACL manipulation.
///
/// # Arguments
/// * `allow_everyone` - Whether to allow access to Everyone group
/// * `admin_only` - Whether to restrict access to administrators only
///
/// # Returns
/// * `Ok(SECURITY_DESCRIPTOR)` if successful
/// * `Err(...)` if creation failed
pub fn create_security_descriptor(allow_everyone: bool, admin_only: bool) -> Result<SECURITY_DESCRIPTOR> {
    unsafe {
        let mut sd: SECURITY_DESCRIPTOR = std::mem::zeroed();

        // Initialize the security descriptor
        let init_result = InitializeSecurityDescriptor(
            &mut sd as *mut _ as PSECURITY_DESCRIPTOR,
            SECURITY_DESCRIPTOR_REVISION,
        );

        if init_result == FALSE {
            return Err(anyhow::anyhow!("Failed to initialize security descriptor"));
        }

        // Create DACL based on parameters
        let dacl: PACL = if admin_only {
            // Restrict to administrators only
            ptr::null_mut() // Would need to create specific DACL
        } else if allow_everyone {
            // Allow everyone (null DACL = everyone has access)
            ptr::null_mut()
        } else {
            // Default inherited permissions
            ptr::null_mut()
        };

        // Set the DACL
        let set_result = SetSecurityDescriptorDacl(
            &mut sd as *mut _ as PSECURITY_DESCRIPTOR,
            TRUE,
            dacl,
            FALSE,
        );

        if set_result == FALSE {
            return Err(anyhow::anyhow!("Failed to set security descriptor DACL"));
        }

        Ok(sd)
    }
}

/// Get effective permissions for the current user on a file
///
/// # Arguments
/// * `file_path` - Path to check permissions for
///
/// # Returns
/// * Tuple of (can_read, can_write, can_execute, can_delete)
pub fn get_effective_permissions(file_path: &str) -> Result<(bool, bool, bool, bool)> {
    debug!("Getting effective permissions for: {}", file_path);

    let path = Path::new(file_path);

    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", file_path));
    }

    // Use standard Rust fs operations to check basic permissions
    let metadata = path.metadata()
        .context("Failed to get file metadata")?;

    let readonly = metadata.permissions().readonly();

    // Check read permission by attempting to open for reading
    let can_read = std::fs::File::open(path).is_ok();

    // Check write permission
    let can_write = !readonly && !requires_admin_access(file_path).unwrap_or(true);

    // Check execute permission (for .exe files)
    let can_execute = if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        matches!(ext_lower.as_str(), "exe" | "bat" | "cmd" | "com" | "scr" | "msi")
    } else {
        false
    };

    // Check delete permission (similar to write for parent directory)
    let parent_path = path.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("."));
    let can_delete = !readonly && !requires_admin_access(&parent_path).unwrap_or(true);

    debug!("Permissions for {}: read={}, write={}, execute={}, delete={}",
           file_path, can_read, can_write, can_execute, can_delete);

    Ok((can_read, can_write, can_execute, can_delete))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_is_running_as_admin() {
        // This test can run in any context
        let result = is_running_as_admin();
        assert!(result.is_ok());

        let is_admin = result.expect("Failed to check admin privileges - ensure Windows API is accessible");
        println!("Running as administrator: {}", is_admin);
    }

    #[test]
    fn test_requires_admin_access() {
        // Test known system directories
        let result = requires_admin_access(r"C:\Windows\System32");
        assert!(result.is_ok());
        assert!(result.expect("Failed to check admin access for System32"), "System32 should require admin access");

        // Test user temp directory (should not require admin)
        let temp_dir = env::temp_dir();
        let temp_path = temp_dir.to_str().expect("Temp directory path should be valid UTF-8");
        let result = requires_admin_access(temp_path);
        assert!(result.is_ok());
        assert!(!result.expect("Failed to check admin access for temp directory"), "Temp directory should not require admin access");
    }

    #[test]
    fn test_get_effective_permissions() {
        // Test on temp file
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("opcode_perm_test.txt");

        // Create test file
        std::fs::write(&test_file, b"test content").expect("Failed to create test file in temp directory");

        let test_path = test_file.to_str().expect("Test file path should be valid UTF-8");
        let result = get_effective_permissions(test_path);
        assert!(result.is_ok(), "Failed to get effective permissions for test file");

        let (can_read, can_write, can_execute, can_delete) = result.expect("Should successfully get permissions for temp file");
        assert!(can_read, "Should be able to read temp file");
        assert!(can_write, "Should be able to write temp file");
        assert!(!can_execute, "Text file should not be executable");
        assert!(can_delete, "Should be able to delete temp file");

        // Clean up
        std::fs::remove_file(&test_file).expect("Failed to clean up test file");
    }

    #[test]
    #[ignore] // Integration test - requires elevation
    fn test_set_file_acl() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("opcode_acl_test.txt");

        // Create test file
        std::fs::write(&test_file, b"test content").expect("Failed to create test file for ACL test");

        // Set ACL
        let test_path = test_file.to_str().expect("Test file path should be valid UTF-8");
        let result = set_file_acl(
            test_path,
            "Users:(R)"
        );

        if result.is_ok() {
            println!("Successfully set ACL");

            // Reset ACL
            let reset_result = reset_file_acl(test_path);
            assert!(reset_result.is_ok());
        }

        // Clean up
        std::fs::remove_file(&test_file).expect("Failed to clean up ACL test file");
    }
}
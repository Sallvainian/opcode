//! Windows-specific runtime features for Opcode
//!
//! This module provides Windows-specific functionality including:
//! - Process management with kill process tree functionality
//! - Registry operations for file associations and URL protocols
//! - Permissions management including UAC and admin privilege checking
//! - Windows ACL and security descriptor handling
//!
//! All functionality is only available when compiled for Windows and follows
//! the project's error handling patterns with anyhow::Result<T>.

#[cfg(target_os = "windows")]
pub mod process;

#[cfg(target_os = "windows")]
pub mod registry;

#[cfg(target_os = "windows")]
pub mod permissions;

// Re-export all Windows functionality
#[cfg(target_os = "windows")]
pub use process::*;

#[cfg(target_os = "windows")]
pub use registry::*;

#[cfg(target_os = "windows")]
pub use permissions::*;

// No-op implementations for non-Windows platforms to maintain API compatibility
#[cfg(not(target_os = "windows"))]
pub mod process {
    use anyhow::Result;

    /// Kill a process tree by PID (no-op on non-Windows)
    pub async fn kill_process_tree(_pid: u32) -> Result<bool> {
        Ok(false)
    }

    /// List processes by name (no-op on non-Windows)
    pub async fn list_processes_by_name(_name: &str) -> Result<Vec<u32>> {
        Ok(vec![])
    }

    /// Check if process is elevated (no-op on non-Windows)
    pub async fn is_process_elevated(_pid: u32) -> Result<bool> {
        Ok(false)
    }
}

#[cfg(not(target_os = "windows"))]
pub mod registry {
    use anyhow::Result;

    /// Register file association (no-op on non-Windows)
    pub fn register_file_association(_extension: &str, _program_id: &str, _executable_path: &str, _description: &str) -> Result<()> {
        Ok(())
    }

    /// Register URL protocol (no-op on non-Windows)
    pub fn register_url_protocol(_protocol: &str, _executable_path: &str, _description: &str) -> Result<()> {
        Ok(())
    }

    /// Set auto-start on login (no-op on non-Windows)
    pub fn set_auto_start(_app_name: &str, _executable_path: &str, _enabled: bool) -> Result<()> {
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub mod permissions {
    use anyhow::Result;

    /// Check if running with admin privileges (no-op on non-Windows)
    pub fn is_running_as_admin() -> Result<bool> {
        Ok(false)
    }

    /// Request UAC elevation (no-op on non-Windows)
    pub async fn request_elevation(_executable_path: &str, _args: &[&str]) -> Result<bool> {
        Ok(false)
    }

    /// Set Windows ACL on file (no-op on non-Windows)
    pub fn set_file_acl(_file_path: &str, _permissions: &str) -> Result<()> {
        Ok(())
    }
}
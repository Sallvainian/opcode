# Windows Implementation Guide

**Complete technical documentation for Opcode's Windows platform support**

## Table of Contents
- [Architecture Overview](#architecture-overview)
- [Windows Modules](#windows-modules)
- [Process Management](#process-management)
- [Registry Operations](#registry-operations)
- [Permissions & UAC](#permissions--uac)
- [Integration with Tauri](#integration-with-tauri)
- [API Reference](#api-reference)
- [Code Examples](#code-examples)
- [Troubleshooting](#troubleshooting)
- [Performance Considerations](#performance-considerations)

---

## Architecture Overview

Opcode's Windows implementation provides native Windows functionality while maintaining cross-platform compatibility. The architecture follows these principles:

### Design Philosophy
- **Windows-First**: Native Windows APIs for optimal performance
- **Cross-Platform Compatibility**: No-op implementations for non-Windows platforms
- **Error Resilience**: Comprehensive error handling with anyhow::Result
- **Security Aware**: UAC and permission-conscious operations
- **Production Ready**: Logging, testing, and validation throughout

### Module Structure
```
src-tauri/src/windows/
├── mod.rs           # Module exports and cross-platform shims
├── process.rs       # Process management and control
├── registry.rs      # Windows Registry operations
└── permissions.rs   # UAC and ACL management
```

### Cross-Platform Strategy
All Windows modules provide identical APIs across platforms:
- **Windows**: Full native implementation using Windows APIs
- **Non-Windows**: No-op implementations that return safe defaults
- **Result Types**: Consistent anyhow::Result<T> return types
- **Logging**: Platform-aware debug and info messages

---

## Windows Modules

### Core Dependencies
```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = [
    "winnt", "winreg", "processthreadsapi", "handleapi",
    "securitybaseapi", "minwindef", "winerror"
]}
anyhow = "1.0"
log = "0.4"
tokio = { version = "1.0", features = ["process", "rt"] }
```

### Module Exports
```rust
// All Windows functionality accessible via:
use crate::windows::{
    // Process management
    kill_process_tree, list_processes_by_name, is_process_elevated,
    get_process_info, is_current_process_elevated,

    // Registry operations
    register_file_association, register_url_protocol, set_auto_start,
    remove_file_association, remove_url_protocol,

    // Permissions management
    is_running_as_admin, request_elevation, set_file_acl,
    remove_file_acl, reset_file_acl, requires_admin_access,
    get_effective_permissions, create_security_descriptor,
};
```

---

## Process Management

### Overview
The process management module (`process.rs`) provides Windows-specific process control capabilities using native Windows commands and APIs.

### Key Features
- **Process Tree Termination**: Kill process and all child processes recursively
- **Process Discovery**: List processes by name with filtering
- **Privilege Detection**: Check if processes run with elevated privileges
- **Process Information**: Detailed process metadata and relationships

### Core Functions

#### Kill Process Tree
```rust
pub async fn kill_process_tree(pid: u32) -> Result<bool>
```

Terminates a process and all its child processes recursively. Uses bottom-up approach to ensure clean termination.

**Algorithm**:
1. Build complete child process map using `wmic process get ProcessId,ParentProcessId`
2. Recursively find all descendant processes
3. Terminate child processes first (bottom-up)
4. Finally terminate root process
5. Attempt graceful termination before forced termination

**Error Handling**:
- Returns `Ok(false)` if process already terminated
- Returns `Err(...)` for permission or system errors
- Continues termination even if some child processes fail

#### List Processes by Name
```rust
pub async fn list_processes_by_name(name: &str) -> Result<Vec<u32>>
```

Finds all running processes matching a specific name.

**Implementation**:
- Uses `tasklist /FI "IMAGENAME eq {name}" /FO CSV /NH`
- Parses CSV output to extract process IDs
- Case-insensitive matching
- Returns empty vector if no matches found

#### Process Elevation Check
```rust
pub async fn is_process_elevated(pid: u32) -> Result<bool>
```

Determines if a specific process is running with administrator privileges.

**Method**:
- Uses PowerShell with WindowsIdentity and WindowsPrincipal
- Checks process token elevation status
- Returns `false` for non-existent processes
- Handles access denied gracefully

### Process Information Structure
```rust
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub parent_pid: Option<u32>,
    pub is_elevated: bool,
}
```

### Usage Examples

#### Basic Process Termination
```rust
use crate::windows::process::kill_process_tree;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let success = kill_process_tree(1234).await?;
    if success {
        println!("Process tree terminated successfully");
    } else {
        println!("Process was already terminated or not found");
    }
    Ok(())
}
```

#### Find and Terminate Processes by Name
```rust
use crate::windows::process::{list_processes_by_name, kill_process_tree};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pids = list_processes_by_name("notepad.exe").await?;

    for pid in pids {
        println!("Terminating notepad.exe with PID {}", pid);
        kill_process_tree(pid).await?;
    }

    Ok(())
}
```

#### Check Process Privileges
```rust
use crate::windows::process::{get_process_info, list_processes_by_name};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pids = list_processes_by_name("svchost.exe").await?;
    let processes = get_process_info(&pids).await?;

    for process in processes {
        println!("PID: {}, Name: {}, Elevated: {}",
                 process.pid, process.name, process.is_elevated);
    }

    Ok(())
}
```

---

## Registry Operations

### Overview
The registry module (`registry.rs`) provides Windows Registry manipulation for file associations, URL protocols, and auto-start configuration.

### Key Features
- **File Associations**: Register custom file extensions
- **URL Protocols**: Handle custom URI schemes
- **Auto-Start Management**: Configure Windows startup behavior
- **Registry Safety**: Proper key creation and cleanup

### Core Functions

#### File Association Registration
```rust
pub fn register_file_association(
    extension: &str,
    program_id: &str,
    executable_path: &str,
    description: &str,
) -> Result<()>
```

Registers a file extension with the application, making it the default handler.

**Registry Keys Created**:
```
HKEY_CLASSES_ROOT\.ext -> program_id
HKEY_CLASSES_ROOT\program_id -> description
HKEY_CLASSES_ROOT\program_id\shell\open\command -> "path\to\exe" "%1"
HKEY_CLASSES_ROOT\program_id\DefaultIcon -> path\to\exe,0
```

**Features**:
- Validates executable path exists
- Sets proper content type
- Configures shell command with file parameter
- Uses executable icon as default icon

#### URL Protocol Registration
```rust
pub fn register_url_protocol(
    protocol: &str,
    executable_path: &str,
    description: &str,
) -> Result<()>
```

Registers a custom URL protocol (e.g., `opcode://`) for the application.

**Registry Keys Created**:
```
HKEY_CLASSES_ROOT\protocol -> description
HKEY_CLASSES_ROOT\protocol\URL Protocol -> ""
HKEY_CLASSES_ROOT\protocol\shell\open\command -> "path\to\exe" "%1"
HKEY_CLASSES_ROOT\protocol\DefaultIcon -> path\to\exe,0
```

#### Auto-Start Configuration
```rust
pub fn set_auto_start(app_name: &str, executable_path: &str, enabled: bool) -> Result<()>
```

Manages Windows startup behavior via the Registry Run key.

**Registry Location**:
```
HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run
```

**Operations**:
- **Enable**: Sets registry value to executable path
- **Disable**: Removes registry value
- **Validation**: Confirms executable exists before enabling

### Windows API Integration

The module uses direct Windows API calls for registry operations:

```rust
// Core API functions used:
RegCreateKeyExW()    // Create/open registry keys
RegSetValueExW()     // Set registry values
RegDeleteKeyW()      // Delete registry keys
RegDeleteTreeW()     // Recursively delete registry trees
RegDeleteValueW()    // Delete specific registry values
```

### Usage Examples

#### Register File Association
```rust
use crate::windows::registry::register_file_association;

fn main() -> anyhow::Result<()> {
    register_file_association(
        ".opc",                           // Extension
        "Opcode.Document",                // Program ID
        r"C:\Program Files\Opcode\opcode.exe", // Executable
        "Opcode Document File"            // Description
    )?;

    println!("File association registered successfully");
    Ok(())
}
```

#### Register URL Protocol
```rust
use crate::windows::registry::register_url_protocol;

fn main() -> anyhow::Result<()> {
    register_url_protocol(
        "opcode",                         // Protocol name
        r"C:\Program Files\Opcode\opcode.exe", // Executable
        "Opcode Protocol Handler"         // Description
    )?;

    // Now opcode:// URLs will open the application
    println!("URL protocol registered successfully");
    Ok(())
}
```

#### Configure Auto-Start
```rust
use crate::windows::registry::set_auto_start;

fn main() -> anyhow::Result<()> {
    // Enable auto-start
    set_auto_start(
        "Opcode",                         // App name in registry
        r"C:\Program Files\Opcode\opcode.exe", // Executable path
        true                              // Enable
    )?;

    // Later, to disable auto-start
    set_auto_start("Opcode", "", false)?;

    Ok(())
}
```

#### Complete Application Registration
```rust
use crate::windows::registry::{
    register_file_association, register_url_protocol, set_auto_start
};

fn setup_windows_integration() -> anyhow::Result<()> {
    let exe_path = std::env::current_exe()?.to_string_lossy().to_string();

    // Register file association
    register_file_association(
        ".opc",
        "Opcode.Document",
        &exe_path,
        "Opcode Document"
    )?;

    // Register URL protocol
    register_url_protocol(
        "opcode",
        &exe_path,
        "Opcode Protocol"
    )?;

    // Enable auto-start (optional)
    set_auto_start("Opcode", &exe_path, true)?;

    println!("Windows integration configured successfully");
    Ok(())
}
```

---

## Permissions & UAC

### Overview
The permissions module (`permissions.rs`) handles Windows User Account Control (UAC), administrator privilege detection, and Access Control List (ACL) management.

### Key Features
- **UAC Integration**: Detect and request administrator privileges
- **Permission Analysis**: Check file/directory access permissions
- **ACL Management**: Modify Windows file permissions programmatically
- **Security Awareness**: Handle security contexts appropriately

### Core Functions

#### Administrator Privilege Detection
```rust
pub fn is_running_as_admin() -> Result<bool>
```

Checks if the current process has administrator privileges using Windows security tokens.

**Implementation Details**:
- Opens current process token with `TOKEN_QUERY` access
- Queries `TokenElevation` information
- Checks `TokenIsElevated` flag
- Properly handles and closes token handles

#### UAC Elevation Request
```rust
pub async fn request_elevation(executable_path: &str, args: &[&str]) -> Result<bool>
```

Requests UAC elevation by launching a new process with the "runas" verb.

**Process**:
1. Validates executable path exists
2. Constructs PowerShell script with ProcessStartInfo
3. Sets `Verb = "runas"` to trigger UAC prompt
4. Launches process and handles user response
5. Returns success/failure/cancellation status

**User Experience**:
- Shows standard Windows UAC prompt
- Allows user to approve or deny elevation
- Gracefully handles cancellation

#### ACL Management
```rust
pub fn set_file_acl(file_path: &str, permissions: &str) -> Result<()>
pub fn remove_file_acl(file_path: &str, principal: &str) -> Result<()>
pub fn reset_file_acl(file_path: &str) -> Result<()>
```

Manages Windows Access Control Lists using the `icacls` command-line utility.

**Permission Syntax**:
- `F` - Full control
- `M` - Modify access
- `RX` - Read and execute
- `R` - Read-only access
- `W` - Write access

**Examples**:
- `"Administrators:F"` - Full control for administrators
- `"Users:(R)"` - Read-only for users
- `"Everyone:(RX)"` - Read and execute for everyone

#### Access Requirements Analysis
```rust
pub fn requires_admin_access(path: &str) -> Result<bool>
```

Determines if a path requires administrator access to modify.

**Detection Methods**:
1. **Path-based**: Check against known system directories
2. **Permission Test**: Attempt to create test file
3. **Error Analysis**: Analyze permission denied errors

**System Directories Requiring Admin**:
- `C:\Windows`
- `C:\Program Files`
- `C:\Program Files (x86)`
- `C:\ProgramData`
- `C:\Windows\System32`

### Security Context Handling

#### Effective Permissions
```rust
pub fn get_effective_permissions(file_path: &str) -> Result<(bool, bool, bool, bool)>
```

Returns tuple of `(can_read, can_write, can_execute, can_delete)` permissions.

**Analysis Method**:
- Uses file metadata for basic permissions
- Checks readonly flag
- Determines executable status by extension
- Tests write access requirements
- Evaluates parent directory permissions for deletion

### Usage Examples

#### Check Administrator Status
```rust
use crate::windows::permissions::is_running_as_admin;

fn main() -> anyhow::Result<()> {
    if is_running_as_admin()? {
        println!("Running with administrator privileges");
        // Perform admin operations
    } else {
        println!("Running as standard user");
        // Request elevation if needed
    }
    Ok(())
}
```

#### Request Elevation
```rust
use crate::windows::permissions::request_elevation;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let elevated = request_elevation(
        &std::env::current_exe()?.to_string_lossy(),
        &["--admin-mode", "--operation", "install"]
    ).await?;

    if elevated {
        println!("Successfully started elevated process");
    } else {
        println!("User denied elevation or it failed");
    }

    Ok(())
}
```

#### Manage File Permissions
```rust
use crate::windows::permissions::{set_file_acl, get_effective_permissions};

fn secure_file(file_path: &str) -> anyhow::Result<()> {
    // Check current permissions
    let (can_read, can_write, can_execute, can_delete) =
        get_effective_permissions(file_path)?;

    println!("Current permissions: R={}, W={}, X={}, D={}",
             can_read, can_write, can_execute, can_delete);

    // Restrict to administrators only
    set_file_acl(file_path, "Administrators:F,Users:R")?;

    println!("File permissions updated");
    Ok(())
}
```

#### Smart Permission Handling
```rust
use crate::windows::permissions::{requires_admin_access, is_running_as_admin, request_elevation};

async fn install_application(install_path: &str) -> anyhow::Result<()> {
    if requires_admin_access(install_path)? {
        if !is_running_as_admin()? {
            println!("Installation requires administrator privileges");

            let exe_path = std::env::current_exe()?;
            let elevated = request_elevation(
                &exe_path.to_string_lossy(),
                &["--install", install_path]
            ).await?;

            if !elevated {
                return Err(anyhow::anyhow!("Administrator privileges required"));
            }
        }
    }

    // Proceed with installation
    println!("Installing to: {}", install_path);
    Ok(())
}
```

---

## Integration with Tauri

### Command Registration

Windows functions are exposed to the frontend through Tauri commands:

```rust
// In src-tauri/src/lib.rs
use crate::windows;

#[tauri::command]
async fn kill_process_tree(pid: u32) -> Result<bool, String> {
    windows::kill_process_tree(pid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_processes_by_name(name: String) -> Result<Vec<u32>, String> {
    windows::list_processes_by_name(&name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn is_running_as_admin() -> Result<bool, String> {
    windows::is_running_as_admin()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn request_elevation(exe_path: String, args: Vec<String>) -> Result<bool, String> {
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    windows::request_elevation(&exe_path, &str_args)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn register_file_association(
    extension: String,
    program_id: String,
    executable_path: String,
    description: String,
) -> Result<(), String> {
    windows::register_file_association(&extension, &program_id, &executable_path, &description)
        .map_err(|e| e.to_string())
}

// Register commands with Tauri
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            kill_process_tree,
            list_processes_by_name,
            is_running_as_admin,
            request_elevation,
            register_file_association,
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend Integration

TypeScript/JavaScript interfaces for frontend usage:

```typescript
// src/lib/windows-api.ts
import { invoke } from '@tauri-apps/api/tauri';

export interface ProcessInfo {
    pid: number;
    name: string;
    parent_pid?: number;
    is_elevated: boolean;
}

export class WindowsAPI {
    // Process management
    static async killProcessTree(pid: number): Promise<boolean> {
        return await invoke('kill_process_tree', { pid });
    }

    static async listProcessesByName(name: string): Promise<number[]> {
        return await invoke('list_processes_by_name', { name });
    }

    static async isProcessElevated(pid: number): Promise<boolean> {
        return await invoke('is_process_elevated', { pid });
    }

    // Permission management
    static async isRunningAsAdmin(): Promise<boolean> {
        return await invoke('is_running_as_admin');
    }

    static async requestElevation(exePath: string, args: string[]): Promise<boolean> {
        return await invoke('request_elevation', { exePath, args });
    }

    // Registry operations
    static async registerFileAssociation(
        extension: string,
        programId: string,
        executablePath: string,
        description: string
    ): Promise<void> {
        return await invoke('register_file_association', {
            extension,
            programId,
            executablePath,
            description
        });
    }

    static async registerUrlProtocol(
        protocol: string,
        executablePath: string,
        description: string
    ): Promise<void> {
        return await invoke('register_url_protocol', {
            protocol,
            executablePath,
            description
        });
    }

    static async setAutoStart(
        appName: string,
        executablePath: string,
        enabled: boolean
    ): Promise<void> {
        return await invoke('set_auto_start', {
            appName,
            executablePath,
            enabled
        });
    }
}
```

### React Component Example

```typescript
// src/components/WindowsIntegration.tsx
import { useState, useEffect } from 'react';
import { WindowsAPI } from '../lib/windows-api';

export function WindowsIntegration() {
    const [isAdmin, setIsAdmin] = useState(false);
    const [processes, setProcesses] = useState<number[]>([]);

    useEffect(() => {
        checkAdminStatus();
    }, []);

    const checkAdminStatus = async () => {
        try {
            const adminStatus = await WindowsAPI.isRunningAsAdmin();
            setIsAdmin(adminStatus);
        } catch (error) {
            console.error('Failed to check admin status:', error);
        }
    };

    const requestElevation = async () => {
        try {
            const elevated = await WindowsAPI.requestElevation(
                await getCurrentExecutablePath(),
                ['--admin-mode']
            );

            if (elevated) {
                // Process will restart with elevation
                console.log('Elevation granted');
            }
        } catch (error) {
            console.error('Elevation failed:', error);
        }
    };

    const killNotepadProcesses = async () => {
        try {
            const pids = await WindowsAPI.listProcessesByName('notepad.exe');
            setProcesses(pids);

            for (const pid of pids) {
                await WindowsAPI.killProcessTree(pid);
            }

            setProcesses([]);
        } catch (error) {
            console.error('Failed to kill processes:', error);
        }
    };

    const registerFileAssociation = async () => {
        try {
            await WindowsAPI.registerFileAssociation(
                '.opc',
                'Opcode.Document',
                await getCurrentExecutablePath(),
                'Opcode Document File'
            );

            console.log('File association registered');
        } catch (error) {
            console.error('Failed to register file association:', error);
        }
    };

    return (
        <div className="windows-integration">
            <h2>Windows Integration</h2>

            <div className="admin-status">
                <p>Administrator Status: {isAdmin ? 'Elevated' : 'Standard User'}</p>
                {!isAdmin && (
                    <button onClick={requestElevation}>
                        Request Elevation
                    </button>
                )}
            </div>

            <div className="process-management">
                <h3>Process Management</h3>
                <button onClick={killNotepadProcesses}>
                    Kill Notepad Processes
                </button>
                {processes.length > 0 && (
                    <p>Found {processes.length} notepad processes</p>
                )}
            </div>

            <div className="registry-operations">
                <h3>Registry Integration</h3>
                <button onClick={registerFileAssociation}>
                    Register .opc Files
                </button>
            </div>
        </div>
    );
}
```

---

## API Reference

### Process Management API

#### `kill_process_tree(pid: u32) -> Result<bool>`
- **Purpose**: Terminate process and all child processes
- **Parameters**: `pid` - Process ID to terminate
- **Returns**: `true` if successful, `false` if process not found
- **Errors**: System errors, permission denied

#### `list_processes_by_name(name: &str) -> Result<Vec<u32>>`
- **Purpose**: Find all processes with specific name
- **Parameters**: `name` - Process name (e.g., "notepad.exe")
- **Returns**: Vector of process IDs
- **Errors**: Command execution failures

#### `is_process_elevated(pid: u32) -> Result<bool>`
- **Purpose**: Check if process has administrator privileges
- **Parameters**: `pid` - Process ID to check
- **Returns**: `true` if elevated, `false` otherwise
- **Errors**: Process not found, access denied

#### `get_process_info(pids: &[u32]) -> Result<Vec<ProcessInfo>>`
- **Purpose**: Get detailed information for multiple processes
- **Parameters**: `pids` - Array of process IDs
- **Returns**: Vector of ProcessInfo structures
- **Errors**: Command execution failures

### Registry API

#### `register_file_association(extension: &str, program_id: &str, executable_path: &str, description: &str) -> Result<()>`
- **Purpose**: Register file extension handler
- **Parameters**:
  - `extension` - File extension (e.g., ".opc")
  - `program_id` - Unique program identifier
  - `executable_path` - Path to executable
  - `description` - Human-readable description
- **Errors**: Registry access denied, invalid path

#### `register_url_protocol(protocol: &str, executable_path: &str, description: &str) -> Result<()>`
- **Purpose**: Register custom URL protocol handler
- **Parameters**:
  - `protocol` - Protocol name (e.g., "opcode")
  - `executable_path` - Path to executable
  - `description` - Protocol description
- **Errors**: Registry access denied, invalid path

#### `set_auto_start(app_name: &str, executable_path: &str, enabled: bool) -> Result<()>`
- **Purpose**: Configure Windows startup behavior
- **Parameters**:
  - `app_name` - Application name in registry
  - `executable_path` - Path to executable (ignored when disabling)
  - `enabled` - `true` to enable, `false` to disable
- **Errors**: Registry access denied, invalid path

### Permissions API

#### `is_running_as_admin() -> Result<bool>`
- **Purpose**: Check current process administrator status
- **Returns**: `true` if running as administrator
- **Errors**: Token access failures

#### `request_elevation(executable_path: &str, args: &[&str]) -> Result<bool>`
- **Purpose**: Request UAC elevation for new process
- **Parameters**:
  - `executable_path` - Path to executable to run elevated
  - `args` - Command-line arguments
- **Returns**: `true` if elevated process started, `false` if denied
- **Errors**: PowerShell execution failures, invalid path

#### `set_file_acl(file_path: &str, permissions: &str) -> Result<()>`
- **Purpose**: Set Windows ACL permissions on file
- **Parameters**:
  - `file_path` - Path to file
  - `permissions` - Permission string (e.g., "Administrators:F,Users:R")
- **Errors**: File not found, permission denied

#### `requires_admin_access(path: &str) -> Result<bool>`
- **Purpose**: Check if path requires administrator access
- **Parameters**: `path` - File or directory path
- **Returns**: `true` if admin access required
- **Errors**: Path access failures

---

## Code Examples

### Complete Application Integration

```rust
use crate::windows::{
    process::{kill_process_tree, list_processes_by_name},
    registry::{register_file_association, register_url_protocol},
    permissions::{is_running_as_admin, request_elevation},
};
use anyhow::Result;

// Complete Windows integration setup
pub async fn setup_windows_integration() -> Result<()> {
    println!("Setting up Windows integration...");

    // Check if we need elevation for registry operations
    if !is_running_as_admin()? {
        println!("Requesting administrator privileges for setup...");
        let exe_path = std::env::current_exe()?;
        let elevated = request_elevation(
            &exe_path.to_string_lossy(),
            &["--setup-windows-integration"]
        ).await?;

        if !elevated {
            return Err(anyhow::anyhow!("Administrator privileges required for Windows integration"));
        }

        // The elevated process will handle the setup
        return Ok(());
    }

    // We're running as admin, proceed with setup
    let exe_path = std::env::current_exe()?.to_string_lossy().to_string();

    // Register file association
    register_file_association(
        ".opc",
        "Opcode.Document",
        &exe_path,
        "Opcode Document File"
    )?;

    // Register URL protocol
    register_url_protocol(
        "opcode",
        &exe_path,
        "Opcode Protocol Handler"
    )?;

    println!("Windows integration setup completed successfully");
    Ok(())
}

// Smart process management with error handling
pub async fn manage_child_processes(parent_name: &str) -> Result<Vec<u32>> {
    let mut terminated_pids = Vec::new();

    // Find all processes with the given name
    let pids = list_processes_by_name(parent_name).await?;

    if pids.is_empty() {
        println!("No processes found with name: {}", parent_name);
        return Ok(terminated_pids);
    }

    println!("Found {} processes named '{}'", pids.len(), parent_name);

    // Terminate each process tree
    for pid in pids {
        match kill_process_tree(pid).await {
            Ok(true) => {
                println!("Successfully terminated process tree starting from PID {}", pid);
                terminated_pids.push(pid);
            }
            Ok(false) => {
                println!("Process {} was already terminated or not found", pid);
            }
            Err(e) => {
                eprintln!("Failed to terminate process {}: {}", pid, e);
                // Continue with other processes
            }
        }
    }

    Ok(terminated_pids)
}

// Comprehensive file permission management
pub fn secure_application_files(app_dir: &str) -> Result<()> {
    use crate::windows::permissions::{
        set_file_acl, get_effective_permissions, requires_admin_access
    };
    use std::fs;
    use std::path::Path;

    let app_path = Path::new(app_dir);

    if !app_path.exists() {
        return Err(anyhow::anyhow!("Application directory does not exist: {}", app_dir));
    }

    // Check if we need admin access
    if requires_admin_access(app_dir)? && !is_running_as_admin()? {
        return Err(anyhow::anyhow!("Administrator privileges required to modify {}", app_dir));
    }

    // Process all files in the application directory
    for entry in fs::read_dir(app_path)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_string_lossy();

        if path.is_file() {
            // Get current permissions
            let (can_read, can_write, can_execute, can_delete) =
                get_effective_permissions(&path_str)?;

            println!("File: {} - R:{} W:{} X:{} D:{}",
                     path_str, can_read, can_write, can_execute, can_delete);

            // Set secure permissions based on file type
            let permissions = if path.extension()
                .map(|ext| ext.to_string_lossy().to_lowercase())
                .as_deref() == Some("exe")
            {
                // Executable files: Administrators full control, Users read/execute
                "Administrators:F,Users:(RX)"
            } else {
                // Data files: Administrators full control, Users read only
                "Administrators:F,Users:R"
            };

            match set_file_acl(&path_str, permissions) {
                Ok(()) => println!("Updated permissions for {}", path_str),
                Err(e) => eprintln!("Failed to update permissions for {}: {}", path_str, e),
            }
        }
    }

    println!("File security update completed");
    Ok(())
}
```

### Error Handling Best Practices

```rust
use anyhow::{Context, Result};
use log::{error, warn, info};

// Robust error handling with logging and recovery
pub async fn robust_process_management(target_name: &str) -> Result<()> {
    info!("Starting process management for: {}", target_name);

    // Step 1: Find processes with detailed error context
    let pids = list_processes_by_name(target_name).await
        .with_context(|| format!("Failed to list processes named '{}'", target_name))?;

    if pids.is_empty() {
        info!("No processes found with name: {}", target_name);
        return Ok(());
    }

    info!("Found {} processes to manage", pids.len());

    let mut success_count = 0;
    let mut failure_count = 0;

    // Step 2: Process each PID with individual error handling
    for pid in pids {
        match kill_process_tree(pid).await {
            Ok(true) => {
                success_count += 1;
                info!("Successfully terminated process tree for PID {}", pid);
            }
            Ok(false) => {
                warn!("Process {} was already terminated or not found", pid);
                // Count as success since the goal is achieved
                success_count += 1;
            }
            Err(e) => {
                failure_count += 1;
                error!("Failed to terminate process {}: {:#}", pid, e);

                // Attempt alternative termination method
                warn!("Attempting alternative termination for PID {}", pid);
                match kill_single_process_alternative(pid).await {
                    Ok(true) => {
                        success_count += 1;
                        info!("Alternative termination succeeded for PID {}", pid);
                    }
                    Ok(false) => {
                        warn!("Process {} not found during alternative termination", pid);
                        success_count += 1;
                    }
                    Err(alt_e) => {
                        error!("Alternative termination also failed for PID {}: {:#}", pid, alt_e);
                    }
                }
            }
        }
    }

    // Step 3: Report results
    info!("Process management completed: {} succeeded, {} failed",
          success_count, failure_count);

    if failure_count > 0 {
        warn!("Some processes could not be terminated");
    }

    Ok(())
}

// Alternative process termination using different method
async fn kill_single_process_alternative(pid: u32) -> Result<bool> {
    use tokio::process::Command;

    // Try using PowerShell Stop-Process cmdlet
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("Stop-Process -Id {} -Force -ErrorAction SilentlyContinue", pid)
        ])
        .output()
        .await
        .context("Failed to execute PowerShell Stop-Process")?;

    // PowerShell returns success even if process not found
    Ok(output.status.success())
}

// Registry operation with rollback capability
pub fn register_with_rollback(
    extension: &str,
    program_id: &str,
    executable_path: &str,
    description: &str,
) -> Result<()> {
    use crate::windows::registry::{register_file_association, remove_file_association};

    info!("Registering file association with rollback capability");

    // Attempt registration
    match register_file_association(extension, program_id, executable_path, description) {
        Ok(()) => {
            info!("File association registered successfully");

            // Verify registration worked by attempting to use it
            match verify_file_association(extension) {
                Ok(true) => {
                    info!("File association verification successful");
                    Ok(())
                }
                Ok(false) => {
                    warn!("File association verification failed, rolling back");
                    let _ = remove_file_association(extension, program_id);
                    Err(anyhow::anyhow!("File association verification failed"))
                }
                Err(e) => {
                    warn!("File association verification error: {}, rolling back", e);
                    let _ = remove_file_association(extension, program_id);
                    Err(e).context("File association verification failed")
                }
            }
        }
        Err(e) => {
            error!("File association registration failed: {:#}", e);
            Err(e).context("Failed to register file association")
        }
    }
}

// Verify file association was registered correctly
fn verify_file_association(extension: &str) -> Result<bool> {
    // Implementation would check registry keys exist and are correct
    // This is a placeholder for the actual verification logic
    Ok(true)
}
```

---

## Troubleshooting

### Common Issues and Solutions

#### Issue: "Access Denied" Registry Errors

**Symptoms**:
```
Failed to create registry key: error code 5
```

**Cause**: Insufficient privileges to modify registry

**Solutions**:
1. **Check Administrator Status**:
   ```rust
   if !is_running_as_admin()? {
       println!("Registry operations require administrator privileges");
       // Request elevation or inform user
   }
   ```

2. **Request Elevation**:
   ```rust
   let elevated = request_elevation(&exe_path, &["--registry-setup"]).await?;
   ```

3. **Use HKEY_CURRENT_USER** (when possible):
   - Modify registry operations to use user-specific keys
   - Avoids need for administrator privileges

#### Issue: Process Termination Failures

**Symptoms**:
```
Failed to terminate process: The process cannot be terminated
```

**Causes**:
- Protected system processes
- Processes running with higher privileges
- Process already terminated

**Solutions**:
1. **Check Process Status First**:
   ```rust
   let pids = list_processes_by_name("target.exe").await?;
   if pids.is_empty() {
       println!("Process already terminated");
       return Ok(true);
   }
   ```

2. **Handle Permission Errors**:
   ```rust
   match kill_process_tree(pid).await {
       Err(e) if e.to_string().contains("Access is denied") => {
           warn!("Insufficient privileges to terminate PID {}", pid);
           Ok(false)
       }
       other => other,
   }
   ```

3. **Use Alternative Methods**:
   ```rust
   // Try PowerShell as fallback
   let output = Command::new("powershell")
       .args(["-Command", &format!("Stop-Process -Id {} -Force", pid)])
       .output().await?;
   ```

#### Issue: UAC Elevation Failures

**Symptoms**:
```
Failed to request elevation: User denied
```

**Causes**:
- User clicked "No" on UAC prompt
- UAC disabled in system
- PowerShell execution policy restrictions

**Solutions**:
1. **Handle User Denial Gracefully**:
   ```rust
   match request_elevation(&exe_path, &args).await {
       Ok(true) => println!("Elevation granted"),
       Ok(false) => {
           println!("Elevation denied by user");
           // Provide alternative options or limited functionality
       }
       Err(e) => println!("Elevation error: {}", e),
   }
   ```

2. **Check PowerShell Execution Policy**:
   ```rust
   let policy_check = Command::new("powershell")
       .args(["-Command", "Get-ExecutionPolicy"])
       .output().await?;
   ```

3. **Provide Manual Instructions**:
   ```rust
   fn show_manual_elevation_instructions() {
       println!("To run with administrator privileges:");
       println!("1. Right-click the application");
       println!("2. Select 'Run as administrator'");
       println!("3. Click 'Yes' on the UAC prompt");
   }
   ```

#### Issue: File Association Not Working

**Symptoms**:
- File association registered but files don't open with application
- Wrong icon displayed for associated files

**Diagnosis Steps**:
1. **Check Registry Keys**:
   ```bash
   reg query HKCR\.ext
   reg query HKCR\ProgramId\shell\open\command
   ```

2. **Verify Executable Path**:
   ```rust
   if !Path::new(executable_path).exists() {
       return Err(anyhow::anyhow!("Executable not found: {}", executable_path));
   }
   ```

3. **Test Command Line**:
   ```cmd
   "C:\Path\To\App.exe" "C:\Test\File.ext"
   ```

**Solutions**:
1. **Use Absolute Paths**:
   ```rust
   let exe_path = std::env::current_exe()?.canonicalize()?;
   register_file_association(ext, prog_id, &exe_path.to_string_lossy(), desc)?;
   ```

2. **Refresh File Associations**:
   ```rust
   use std::process::Command;

   // Notify Windows of association changes
   Command::new("cmd")
       .args(["/c", "assoc", &format!("{}=Opcode.Document", extension)])
       .output()?;
   ```

3. **Re-register After Updates**:
   ```rust
   // Always re-register after application updates
   pub fn update_file_associations() -> Result<()> {
       let exe_path = std::env::current_exe()?;
       register_file_association(".opc", "Opcode.Document",
                                &exe_path.to_string_lossy(),
                                "Opcode Document")?;
       Ok(())
   }
   ```

### Debugging Techniques

#### Enable Detailed Logging
```rust
use log::{debug, info, warn, error};
use env_logger;

fn init_windows_logging() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("debug")
    ).init();

    info!("Windows module logging initialized");
}
```

#### PowerShell Debugging
```rust
// Add verbose output to PowerShell commands
let script = format!(r#"
    $VerbosePreference = 'Continue'
    Write-Verbose "Checking process elevation for PID {}"
    # ... rest of script
"#, pid);
```

#### Registry Debugging
```rust
use winapi::um::winreg::RegQueryValueExW;

// Helper function to read registry values for debugging
unsafe fn debug_registry_value(key: HKEY, value_name: &str) -> Result<String> {
    let value_name_wide = to_wide_string(value_name);
    let mut buffer: [u16; 1024] = [0; 1024];
    let mut buffer_size: DWORD = (buffer.len() * 2) as DWORD;

    let result = RegQueryValueExW(
        key,
        value_name_wide.as_ptr(),
        ptr::null_mut(),
        ptr::null_mut(),
        buffer.as_mut_ptr() as *mut u8,
        &mut buffer_size,
    );

    if result == ERROR_SUCCESS as i32 {
        Ok(String::from_utf16_lossy(&buffer[..(buffer_size as usize / 2)]))
    } else {
        Err(anyhow::anyhow!("Failed to read registry value: {}", result))
    }
}
```

### Performance Considerations

#### Batch Operations
```rust
// Instead of individual registry operations
for extension in extensions {
    register_file_association(extension, prog_id, exe_path, desc)?;
}

// Use batch registry operations
fn register_multiple_associations(associations: &[(String, String, String, String)]) -> Result<()> {
    // Single registry transaction for all associations
    unsafe {
        let key = create_registry_key(HKEY_CLASSES_ROOT, "")?;

        for (ext, prog_id, exe_path, desc) in associations {
            // Perform all operations within single key context
        }

        RegCloseKey(key);
    }

    Ok(())
}
```

#### Process Queries Optimization
```rust
// Cache process list for multiple operations
pub struct ProcessManager {
    process_cache: HashMap<String, Vec<u32>>,
    cache_time: std::time::Instant,
    cache_duration: std::time::Duration,
}

impl ProcessManager {
    pub async fn get_processes_cached(&mut self, name: &str) -> Result<Vec<u32>> {
        if self.cache_time.elapsed() > self.cache_duration {
            self.process_cache.clear();
        }

        if let Some(cached) = self.process_cache.get(name) {
            return Ok(cached.clone());
        }

        let processes = list_processes_by_name(name).await?;
        self.process_cache.insert(name.to_string(), processes.clone());
        self.cache_time = std::time::Instant::now();

        Ok(processes)
    }
}
```

#### Memory Management
```rust
// Proper cleanup of Windows handles
pub struct RegistryKeyGuard {
    key: HKEY,
}

impl RegistryKeyGuard {
    pub fn new(key: HKEY) -> Self {
        Self { key }
    }
}

impl Drop for RegistryKeyGuard {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.key);
        }
    }
}

// Usage
unsafe {
    let key = create_registry_key(HKEY_CLASSES_ROOT, "SomeKey")?;
    let _guard = RegistryKeyGuard::new(key);

    // Operations with key
    set_registry_value(key, "Value", "Data")?;

    // Automatic cleanup when guard goes out of scope
}
```

---

*This completes the comprehensive Windows implementation documentation. The implementation provides full Windows native functionality while maintaining cross-platform compatibility and production-ready error handling, logging, and testing capabilities.*
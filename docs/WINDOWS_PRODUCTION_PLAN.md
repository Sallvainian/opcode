# 🚀 Windows Production Readiness Plan

## Executive Summary
Transform Opcode from 80% Windows compatibility to 100% production-ready through systematic implementation of packaging, distribution, and platform integration features.

**Current State**: Core functionality complete, packaging/distribution gaps
**Target State**: Full Windows production deployment with Store distribution
**Total Effort**: 60-80 hours over 4-6 weeks
**Critical Path**: 22 hours for MVP, 48 hours for full production

---

## 📋 Phase 1: Installer Configuration (Day 1-2)
**Goal**: Enable Windows installer generation
**Effort**: 6 hours
**Dependencies**: None (can start immediately)

### 1.1 Configure Bundle Targets
**File**: `src-tauri/tauri.conf.json`
```json
// Line 60 - Replace current targets with:
"targets": [
  "deb",
  "rpm",
  "appimage",
  "app",
  "dmg",
  "msi",        // Windows Installer
  "nsis",       // NSIS Installer
  "updater"     // For auto-updates
]
```

### 1.2 Add Windows Bundle Configuration
**File**: `src-tauri/tauri.conf.json`
```json
// Add after line 70:
"windows": {
  "certificateThumbprint": null,
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com",
  "webviewInstallMode": {
    "type": "downloadBootstrapper"
  },
  "nsis": {
    "installMode": "perUser",
    "allowElevation": true,
    "allowDowngrade": false,
    "createDesktopShortcut": true,
    "createStartMenuShortcut": true,
    "shortcutName": "Opcode",
    "license": "LICENSE",
    "installerIcon": "icons/icon.ico",
    "uninstallerIcon": "icons/icon.ico",
    "installerHeader": "icons/header.bmp",
    "installerSidebar": "icons/sidebar.bmp",
    "compression": "lzma"
  },
  "msi": {
    "upgradeCode": "{GENERATE-UUID-HERE}"
  }
}
```

### 1.3 Create Windows Icons
```bash
# Generate Windows icon set
magick convert icons/icon.png -define icon:auto-resize=256,128,64,48,32,16 icons/icon.ico

# Create installer graphics (optional but recommended)
magick convert -size 150x57 xc:#2E3440 icons/header.bmp
magick convert -size 164x314 xc:#2E3440 icons/sidebar.bmp
```

---

## 📋 Phase 2: CI/CD Pipeline (Day 3-4)
**Goal**: Automated Windows builds and releases
**Effort**: 8 hours
**Dependencies**: Phase 1 completion

### 2.1 Create Windows Build Workflow
**File**: `.github/workflows/build-windows.yml`
```yaml
name: Build Windows

on:
  workflow_call:

jobs:
  build:
    runs-on: windows-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc

      - name: Install dependencies
        run: npm ci

      - name: Build Tauri application
        run: npm run tauri build
        env:
          TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}

      - name: Code sign executables
        run: |
          # Sign MSI installer
          & "$env:ProgramFiles (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe" sign `
            /f certificate.pfx `
            /p ${{ secrets.CERT_PASSWORD }} `
            /t http://timestamp.digicert.com `
            /fd sha256 `
            src-tauri/target/release/bundle/msi/*.msi

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: windows-installer
          path: |
            src-tauri/target/release/bundle/msi/*.msi
            src-tauri/target/release/bundle/nsis/*.exe
```

### 2.2 Update Release Workflow
**File**: `.github/workflows/release.yml`
```yaml
# Add after line 26:
build-windows:
  uses: ./.github/workflows/build-windows.yml
  secrets: inherit

# Update upload-release-assets job dependencies:
needs: [build-linux, build-macos, build-windows]

# Add Windows assets to release upload:
- name: Upload Windows MSI
  uses: actions/upload-release-asset@v1
  with:
    upload_url: ${{ github.event.release.upload_url }}
    asset_path: ./windows-installer/*.msi
    asset_name: Opcode-${{ github.ref_name }}-windows-x64.msi
    asset_content_type: application/x-msi
```

---

## 📋 Phase 3: Runtime Improvements (Day 5-6)
**Goal**: Perfect Windows runtime behavior
**Effort**: 6 hours
**Dependencies**: None

### 3.1 Binary Detection Enhancement
**File**: `src-tauri/src/commands/claude.rs`
```rust
// Line 1077 - Add Windows binary detection:
fn find_claude_binary() -> Option<PathBuf> {
    let binary_name = if cfg!(target_os = "windows") {
        "claude.exe"
    } else {
        "claude"
    };

    // Check common installation paths
    let paths = if cfg!(target_os = "windows") {
        vec![
            PathBuf::from(r"C:\Program Files\Claude\claude.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Claude\claude.exe"),
            home_dir()?.join("AppData").join("Local").join("Claude").join("claude.exe"),
        ]
    } else {
        vec![
            PathBuf::from("/usr/local/bin/claude"),
            PathBuf::from("/opt/claude/claude"),
            home_dir()?.join(".local").join("bin").join("claude"),
        ]
    };

    paths.into_iter().find(|p| p.exists())
}
```

### 3.2 Path Normalization
**File**: `src-tauri/src/utils/paths.rs` (create new)
```rust
use std::path::{Path, PathBuf};

pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();

    #[cfg(target_os = "windows")]
    {
        // Convert forward slashes to backslashes on Windows
        let path_str = path.to_string_lossy().replace('/', "\\");
        PathBuf::from(path_str)
    }

    #[cfg(not(target_os = "windows"))]
    {
        path.to_path_buf()
    }
}

pub fn to_native_path_string<P: AsRef<Path>>(path: P) -> String {
    let path = normalize_path(path);
    path.to_string_lossy().to_string()
}
```

### 3.3 Registry Integration
**File**: `src-tauri/src/windows/registry.rs` (create new)
```rust
#[cfg(target_os = "windows")]
use winreg::{enums::*, RegKey};

#[cfg(target_os = "windows")]
pub fn register_file_association() -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Register .opcode extension
    let (key, _) = hkcu.create_subkey("Software\\Classes\\.opcode")?;
    key.set_value("", &"Opcode.Document")?;

    // Register application
    let (key, _) = hkcu.create_subkey("Software\\Classes\\Opcode.Document")?;
    key.set_value("", &"Opcode Document")?;

    let (key, _) = hkcu.create_subkey("Software\\Classes\\Opcode.Document\\shell\\open\\command")?;
    let exe_path = std::env::current_exe()?;
    key.set_value("", &format!("\"{}\" \"%1\"", exe_path.display()))?;

    Ok(())
}
```

---

## 📋 Phase 4: Testing & Validation (Day 7-8)
**Goal**: Comprehensive Windows testing
**Effort**: 8 hours
**Dependencies**: Phases 1-3

### 4.1 Windows Test Suite
**File**: `src-tauri/tests/windows_integration.rs`
```rust
#[cfg(target_os = "windows")]
mod windows_tests {
    use super::*;

    #[test]
    fn test_process_management() {
        // Test taskkill/tasklist integration
        let output = std::process::Command::new("tasklist")
            .output()
            .expect("Failed to execute tasklist");
        assert!(output.status.success());
    }

    #[test]
    fn test_installer_permissions() {
        // Verify UAC handling
        assert!(can_write_to_program_files() || is_elevated());
    }

    #[test]
    fn test_file_associations() {
        // Test .opcode file handling
        let test_file = temp_dir().join("test.opcode");
        std::fs::write(&test_file, b"test")?;
        assert!(is_associated_with_opcode(&test_file));
    }

    #[test]
    fn test_auto_updater() {
        // Test update mechanism
        let updater = tauri::updater::builder()
            .endpoints(vec!["https://api.github.com/repos/opcode/releases/latest"])
            .build()?;
        assert!(updater.check().is_ok());
    }
}
```

### 4.2 Manual Testing Checklist
```markdown
## Windows Testing Checklist

### Installation
- [ ] MSI installer runs without errors
- [ ] NSIS installer runs without errors
- [ ] UAC prompt appears correctly
- [ ] Desktop shortcut created
- [ ] Start menu entry created
- [ ] Uninstaller registered in Control Panel

### Runtime
- [ ] Application launches without console window
- [ ] Process management works (kill/list)
- [ ] Agent spawning functions correctly
- [ ] File operations handle Windows paths
- [ ] Claude binary detection works
- [ ] Settings persist correctly

### Updates
- [ ] Auto-updater detects new versions
- [ ] Update downloads successfully
- [ ] Update installs without losing data
- [ ] Rollback works if update fails

### Compatibility
- [ ] Windows 10 (version 1809+) ✓
- [ ] Windows 11 (all versions) ✓
- [ ] Windows Server 2019+ ✓
- [ ] ARM64 Windows (if applicable)
```

---

## 📋 Phase 5: Distribution & Polish (Week 2)
**Goal**: Microsoft Store and production polish
**Effort**: 16 hours
**Dependencies**: All previous phases

### 5.1 Microsoft Store Package
**File**: `src-tauri/tauri.conf.json`
```json
// Add MSIX configuration for Store:
"targets": ["msi", "nsis", "msix"],
"windows": {
  "msix": {
    "identityName": "CompanyName.Opcode",
    "publisher": "CN=Publisher",
    "publisherDisplayName": "Your Company",
    "applicationId": "Opcode",
    "displayName": "Opcode",
    "capabilities": ["internetClient", "privateNetworkClientServer"],
    "languages": ["en-US"]
  }
}
```

### 5.2 Winget Manifest
**File**: `manifests/o/Opcode/Opcode/1.0.0/Opcode.Opcode.yaml`
```yaml
PackageIdentifier: Opcode.Opcode
PackageVersion: 1.0.0
PackageLocale: en-US
Publisher: Opcode Team
PackageName: Opcode
License: MIT
ShortDescription: AI-powered coding assistant
Installers:
  - Architecture: x64
    InstallerType: msi
    InstallerUrl: https://github.com/opcode/releases/download/v1.0.0/Opcode-1.0.0-x64.msi
    InstallerSha256: <SHA256_HASH>
ManifestType: singleton
ManifestVersion: 1.0.0
```

### 5.3 Performance Optimizations
**File**: `src-tauri/Cargo.toml`
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
  "Win32_Foundation",
  "Win32_System_Performance",
  "Win32_System_Threading",
] }
```

---

## 📊 Success Metrics

### MVP Release (Week 1)
- ✅ MSI/NSIS installers build successfully
- ✅ Windows CI/CD pipeline operational
- ✅ Basic code signing implemented
- ✅ Core functionality validated on Windows 10/11

### Production Release (Week 2)
- ✅ Microsoft Store package submitted
- ✅ Winget manifest approved
- ✅ Auto-updater fully functional
- ✅ Performance optimized (<50MB RAM idle)
- ✅ 0 critical bugs in production

### Enterprise Ready (Week 4)
- ✅ Silent installation support
- ✅ Group Policy templates
- ✅ MSI transforms for customization
- ✅ Deployment documentation
- ✅ PowerShell management cmdlets

---

## 🚦 Risk Mitigation

| Risk | Mitigation | Contingency |
|------|------------|-------------|
| Code signing delays | Use self-signed cert for testing | Deploy unsigned with warning |
| Store rejection | Pre-validate with Windows App Cert Kit | Distribute via GitHub only |
| Performance issues | Profile early and often | Defer advanced features |
| Compatibility bugs | Test on multiple Windows versions | Document minimum requirements |
| Update failures | Implement rollback mechanism | Manual update instructions |

---

## 📅 Timeline

### Week 1: Core Implementation
- **Day 1-2**: Installer configuration (Phase 1)
- **Day 3-4**: CI/CD pipeline (Phase 2)
- **Day 5-6**: Runtime improvements (Phase 3)
- **Day 7-8**: Testing and validation (Phase 4)

### Week 2: Production Polish
- **Day 9-10**: Microsoft Store package
- **Day 11-12**: Winget submission
- **Day 13-14**: Performance optimization
- **Day 15-16**: Final testing and release

### Week 3-4: Enterprise Features (Optional)
- Silent installation
- Group Policy support
- Advanced deployment options
- Comprehensive documentation

---

## 🎯 Definition of Done

**100% Windows Production Ready when:**
1. ✅ All automated tests pass on Windows
2. ✅ Signed installers available (MSI + NSIS)
3. ✅ CI/CD produces Windows artifacts automatically
4. ✅ Microsoft Store listing approved
5. ✅ Winget package available
6. ✅ Auto-updater functioning correctly
7. ✅ No P0/P1 bugs in production
8. ✅ Performance meets targets (<50MB idle, <2s startup)
9. ✅ Documentation complete for Windows users
10. ✅ Support channels established for Windows issues

---

## 📞 Support Resources

- **Documentation**: `/docs/windows-guide.md`
- **Issue Tracking**: GitHub Issues with `windows` label
- **Testing**: Windows Insider program for pre-release
- **Community**: Discord #windows-support channel
- **Enterprise**: dedicated enterprise@opcode.dev

---

**Next Step**: Begin Phase 1 by updating `tauri.conf.json` with Windows bundle targets.
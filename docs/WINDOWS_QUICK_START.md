# 🚀 Windows Compatibility Quick Start Guide

## Immediate Actions (30 minutes)

### 1️⃣ Run Setup Script
```powershell
# Execute from project root
.\scripts\windows-setup.ps1
```
This will:
- ✅ Configure Windows build targets
- ✅ Create CI/CD workflow
- ✅ Generate Windows icons
- ✅ Validate build environment

### 2️⃣ Manual Configuration (if script fails)

#### Update `src-tauri/tauri.conf.json`:
```json
// Line 60 - Add Windows targets
"targets": ["deb", "rpm", "appimage", "app", "dmg", "msi", "nsis"]

// After line 70 - Add Windows config
"windows": {
  "nsis": {
    "installMode": "perUser",
    "createDesktopShortcut": true,
    "createStartMenuShortcut": true
  }
}
```

#### Create `.github/workflows/build-windows.yml`:
```yaml
name: Build Windows
on:
  workflow_call:
jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: npm ci
      - run: npm run tauri build
      - uses: actions/upload-artifact@v4
        with:
          name: windows-installer
          path: src-tauri/target/release/bundle/**/*.{msi,exe}
```

---

## Critical Fixes Needed

### 🔴 HIGH PRIORITY (Blocks Release)

| Issue | Fix | File | Time |
|-------|-----|------|------|
| No Windows installers | Add `"msi", "nsis"` to targets | `tauri.conf.json:60` | 5 min |
| No CI/CD workflow | Create `build-windows.yml` | `.github/workflows/` | 15 min |
| Release excludes Windows | Add Windows to release.yml | `release.yml:26` | 10 min |

### 🟡 MEDIUM PRIORITY (Core Features)

| Issue | Fix | File | Time |
|-------|-----|------|------|
| Claude.exe not detected | Add `.exe` suffix check | `claude.rs:1077` | 30 min |
| File permissions missing | Implement Windows ACLs | `manager.rs:422` | 1 hour |
| Path separators | Normalize to backslash | Create `utils/paths.rs` | 30 min |

### 🟢 LOW PRIORITY (Polish)

| Issue | Fix | File | Time |
|-------|-----|------|------|
| No Windows icons | Generate .ico file | `icons/icon.ico` | 15 min |
| No registry integration | Add file associations | New `windows/registry.rs` | 2 hours |
| No Store package | Configure MSIX | `tauri.conf.json` | 4 hours |

---

## Test Commands

```powershell
# Build Windows installers
npm run tauri build

# Test process management
tasklist | findstr opcode
taskkill /F /PID <pid>

# Verify installer
msiexec /i "src-tauri\target\release\bundle\msi\*.msi" /quiet

# Check Windows compatibility
npm run tauri build -- --target x86_64-pc-windows-msvc
```

---

## Validation Checklist

### Before First Release
- [ ] MSI installer builds successfully
- [ ] NSIS installer builds successfully
- [ ] Application launches without console
- [ ] Process kill/list commands work
- [ ] Settings persist after restart

### Before Store Submission
- [ ] Code signed with valid certificate
- [ ] Auto-updater configured and tested
- [ ] Windows 10/11 compatibility verified
- [ ] Performance <50MB RAM idle
- [ ] Uninstaller removes all traces

---

## Quick Fixes

### If build fails:
```powershell
# Clear cache and rebuild
cargo clean
rm -r node_modules
npm ci
npm run tauri build
```

### If installer fails:
```powershell
# Check Windows SDK
winget install Microsoft.WindowsSDK

# Install WiX Toolset (for MSI)
winget install WixToolset.WixToolset
```

### If signing fails:
```powershell
# Use self-signed cert for testing
New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=Opcode Dev"
```

---

## Resources

- **Full Plan**: [WINDOWS_PRODUCTION_PLAN.md](./WINDOWS_PRODUCTION_PLAN.md)
- **Tauri Docs**: https://tauri.app/v1/guides/building/windows
- **Code Signing**: https://docs.microsoft.com/windows/win32/seccrypto/cryptography-tools
- **Store Submission**: https://docs.microsoft.com/windows/apps/publish/

---

## Support Channels

- **GitHub Issues**: Label with `windows`
- **Discord**: #windows-support
- **Direct**: windows@opcode.dev

---

**Next Step**: Run `.\scripts\windows-setup.ps1` to begin!
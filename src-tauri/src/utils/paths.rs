use std::path::{Path, PathBuf, MAIN_SEPARATOR};

/// Normalize a path for the current platform
/// On Windows, converts forward slashes to backslashes
/// On Unix, maintains forward slashes
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();

    // Convert to string for manipulation
    let path_str = path.to_string_lossy();

    // Platform-specific normalization
    #[cfg(target_os = "windows")]
    {
        // Replace forward slashes with backslashes on Windows
        let normalized = path_str.replace('/', "\\");
        // Also handle double backslashes (except for UNC paths)
        let normalized = if normalized.starts_with("\\\\") {
            // Preserve UNC path prefix
            format!("\\\\{}", normalized[2..].replace("\\\\", "\\"))
        } else {
            normalized.replace("\\\\", "\\")
        };
        PathBuf::from(normalized)
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix systems, just return the path as-is
        path.to_path_buf()
    }
}

/// Convert a path to a native string representation
/// This ensures the path uses the correct separators for the current platform
pub fn to_native_path_string<P: AsRef<Path>>(path: P) -> String {
    let normalized = normalize_path(path);
    normalized.to_string_lossy().to_string()
}

/// Join paths with proper platform-specific separator
pub fn join_paths<P: AsRef<Path>, Q: AsRef<Path>>(base: P, path: Q) -> PathBuf {
    let mut result = normalize_path(base);
    result.push(normalize_path(path));
    result
}

/// Check if a path is absolute for the current platform
pub fn is_absolute_path<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    #[cfg(target_os = "windows")]
    {
        // Windows absolute paths start with drive letter (C:\) or UNC (\\server\)
        let path_str = path.to_string_lossy();
        (path_str.len() >= 3 && path_str.chars().nth(1) == Some(':'))
            || path_str.starts_with("\\\\")
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix absolute paths start with /
        path.is_absolute()
    }
}

/// Get the platform-specific path separator
pub fn get_separator() -> char {
    MAIN_SEPARATOR
}

/// Ensure a path ends with the platform separator
pub fn ensure_trailing_separator<P: AsRef<Path>>(path: P) -> String {
    let mut path_str = to_native_path_string(path);
    let sep = get_separator();

    if !path_str.ends_with(sep) {
        path_str.push(sep);
    }

    path_str
}

/// Convert a Unix-style path to the current platform's style
pub fn from_unix_path(path: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Convert forward slashes to backslashes on Windows
        PathBuf::from(path.replace('/', "\\"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from(path)
    }
}

/// Convert a Windows-style path to the current platform's style
pub fn from_windows_path(path: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(path)
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Convert backslashes to forward slashes on Unix
        PathBuf::from(path.replace('\\', "/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        #[cfg(target_os = "windows")]
        {
            assert_eq!(
                normalize_path("C:/Users/test/file.txt").to_str().unwrap(),
                "C:\\Users\\test\\file.txt"
            );
            assert_eq!(
                normalize_path("C:\\Users\\test\\file.txt").to_str().unwrap(),
                "C:\\Users\\test\\file.txt"
            );
            assert_eq!(
                normalize_path("\\\\server\\share\\file.txt").to_str().unwrap(),
                "\\\\server\\share\\file.txt"
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(
                normalize_path("/home/user/file.txt").to_str().unwrap(),
                "/home/user/file.txt"
            );
        }
    }

    #[test]
    fn test_is_absolute_path() {
        #[cfg(target_os = "windows")]
        {
            assert!(is_absolute_path("C:\\Users\\test"));
            assert!(is_absolute_path("D:/Projects"));
            assert!(is_absolute_path("\\\\server\\share"));
            assert!(!is_absolute_path("relative\\path"));
            assert!(!is_absolute_path("./file.txt"));
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert!(is_absolute_path("/home/user"));
            assert!(!is_absolute_path("relative/path"));
            assert!(!is_absolute_path("./file.txt"));
        }
    }

    #[test]
    fn test_join_paths() {
        #[cfg(target_os = "windows")]
        {
            let result = join_paths("C:\\Users", "test\\file.txt");
            assert_eq!(result.to_str().unwrap(), "C:\\Users\\test\\file.txt");
        }

        #[cfg(not(target_os = "windows"))]
        {
            let result = join_paths("/home", "user/file.txt");
            assert_eq!(result.to_str().unwrap(), "/home/user/file.txt");
        }
    }
}
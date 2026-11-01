//! File access for the engine. `write_text` writes atomically (temporary file, then rename) and is the only way user files should be written.

use std::fs;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::engine::{ErrorCode, Result};

pub fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|err| {
        ErrorCode::IoFailure
            .error()
            .with_context("operation", "load_file")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })
}

/// Write a file so that it is never seen half-written.
///
/// The content goes to a temporary file in the same directory, is flushed to
/// disk, and is then renamed over the target. A crash, a full disk or an
/// interrupt therefore leaves either the old file or the new one, never a
/// truncated mix. The original's permissions are kept, and a symlinked file is
/// written through the link instead of being replaced by a regular file.
pub fn write_text(path: &Path, content: &str) -> Result<()> {
    let failed = |operation: &'static str, err: &dyn std::fmt::Display| {
        ErrorCode::IoFailure
            .error()
            .with_context("operation", operation)
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    };

    // Replace the file a symlink points at, not the link itself.
    let target = match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            fs::canonicalize(path).map_err(|err| failed("resolve_symlink", &err))?
        }
        _ => path.to_path_buf(),
    };

    let parent = match target.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => std::path::PathBuf::from("."),
    };
    create_dir_all(&parent)?;

    let mut temp =
        tempfile::NamedTempFile::new_in(&parent).map_err(|err| failed("create_temp_file", &err))?;
    {
        use std::io::Write;
        temp.write_all(content.as_bytes())
            .map_err(|err| failed("write_temp_file", &err))?;
        temp.as_file()
            .sync_all()
            .map_err(|err| failed("flush_temp_file", &err))?;
    }

    // Keep the mode of the file being replaced (the temp file starts private).
    if let Ok(existing) = fs::metadata(&target) {
        fs::set_permissions(temp.path(), existing.permissions())
            .map_err(|err| failed("copy_permissions", &err))?;
    }

    temp.persist(&target)
        .map_err(|err| failed("replace_file", &err.error))?;
    Ok(())
}

pub fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|err| {
        ErrorCode::IoFailure
            .error()
            .with_context("operation", "create_dir_all")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })
}

pub fn current_dir() -> Result<std::path::PathBuf> {
    std::env::current_dir().map_err(|err| {
        ErrorCode::IoFailure
            .error()
            .with_context("operation", "current_dir")
            .with_context("error", err.to_string())
    })
}
pub fn load_json<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let raw = read_text(path)?;

    serde_json::from_str(&raw).map_err(|err| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "load_json")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })
}

pub fn save_json<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let raw = serde_json::to_string_pretty(value).map_err(|err| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "save_json")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })?;

    write_text(path, &raw)
}

pub fn load_toml<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let raw = read_text(path)?;

    toml::from_str(&raw).map_err(|err| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "load_toml")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })
}

pub fn save_toml<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let raw = toml::to_string_pretty(value).map_err(|err| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "save_toml")
            .with_context("path", path.display().to_string())
            .with_context("error", err.to_string())
    })?;

    write_text(path, &raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_new_files_and_creates_missing_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a/b/config.toml");

        write_text(&path, "x = 1\n").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "x = 1\n");
    }

    #[test]
    fn replaces_existing_content_completely() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "a long first version of the file\n".repeat(50)).unwrap();

        write_text(&path, "short\n").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "short\n");
    }

    #[test]
    fn leaves_no_temporary_files_behind() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        write_text(&path, "one\n").unwrap();
        write_text(&path, "two\n").unwrap();

        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["config.toml"]);
    }

    #[test]
    fn a_failed_replace_keeps_the_original_and_cleans_up() {
        // The target is a directory, so the final rename cannot succeed.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::create_dir(&path).unwrap();
        fs::write(path.join("keep"), "precious").unwrap();

        assert!(write_text(&path, "new\n").is_err());

        assert_eq!(fs::read_to_string(path.join("keep")).unwrap(), "precious");
        let leftovers = fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(leftovers, 1, "a temporary file was left behind");
    }

    #[cfg(unix)]
    #[test]
    fn keeps_the_permissions_of_the_file_it_replaces() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "old\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

        write_text(&path, "new\n").unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o640);
    }

    #[cfg(unix)]
    #[test]
    fn writes_through_a_symlink_instead_of_replacing_it() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("dotfiles-config.toml");
        let link = dir.path().join("config.toml");
        fs::write(&real, "old\n").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();

        write_text(&link, "new\n").unwrap();

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_to_string(&real).unwrap(), "new\n");
    }
}

use crate::errors::MoldXError2;
use crate::types::Entity;
use anyhow::{bail, Result};
use std::fs;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Reads a directory and returns its entries sorted by file name.
///
/// # Arguments
///
/// * `path` - The directory to read.
///
/// # Returns
///
/// The directory entries sorted lexicographically by file name.
///
/// # Errors
///
/// Returns an error if the directory cannot be read or an entry cannot be
/// accessed.
pub fn sorted_read_dir(path: &Path) -> Result<Vec<std::fs::DirEntry>> {
    let mut entries = std::fs::read_dir(path)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|a| a.file_name());
    Ok(entries)
}

/// Returns whether a path refers to a shell script.
///
/// # Arguments
///
/// * `path` - The path to test.
///
/// # Returns
///
/// `true` if the path has a `.sh` extension.
pub fn is_shell_script(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("sh")
}

/// Collects relative names of all visible files contained in a directory.
///
/// Hidden files and directories are skipped. Relative paths preserve nested
/// template structure. When the given path is a file, a set containing only
/// its own name is returned.
///
/// # Arguments
///
/// * `root` - The directory (or file) to inspect.
///
/// # Returns
///
/// The sorted set of visible file names.
///
/// # Errors
///
/// Returns an error if the directory cannot be read.
pub fn file_names_for_dir(root: &Path) -> Result<BTreeSet<String>> {
    if root.is_file() {
        let mut names = BTreeSet::new();
        if let Some(name) = root.file_name().and_then(|name| name.to_str()) {
            names.insert(name.to_string());
        }
        return Ok(names);
    }

    let mut names = BTreeSet::new();
    for entry in WalkDir::new(root).min_depth(1).into_iter().flatten() {
        let path = entry.path();
        let hidden = path
            .strip_prefix(root)
            .ok()
            .is_some_and(|relative| {
                relative.components().any(|component| {
                    component.as_os_str().to_string_lossy().starts_with('.')
                })
            });
        if !path.is_file() || hidden {
            continue;
        }

        if let Ok(relative) = path.strip_prefix(root) {
            names.insert(relative.to_string_lossy().replace('\\', "/"));
        }
    }

    Ok(names)
}

/// Extracts a human-readable name from a path.
///
/// For directories the file name is used; for files the file stem is used.
/// The special names `.` and `..` are rejected.
///
/// # Arguments
///
/// * `path` - The path to extract a name from.
/// * `entity` - The entity kind, used for error context.
///
/// # Returns
///
/// The extracted name as a string.
///
/// # Errors
///
/// Returns [`MoldXError2::NameResolutionFailed`] when the name cannot be
/// determined, or [`MoldXError2::InvalidName`] for `.` or `..`.
pub fn resolve_name(path: &Path, entity: Entity) -> Result<String> {
    let name = if path.is_dir() {
        path.file_name()
    } else if path.is_file() {
        path.file_stem()
    } else {
        None
    }
        .and_then(|name| name.to_str())
        .ok_or_else(|| MoldXError2::NameResolutionFailed {
            path: path.to_path_buf(),
            entity,
        })?
        .to_owned();

    if name == "." || name == ".." {
        bail!(MoldXError2::InvalidName { name, entity });
    }

    Ok(name)
}

/// Searches the filesystem for the first path satisfying a predicate.
///
/// When `traverse_upward` is set, the search starts at `start` and walks up
/// the directory tree, checking each level and its direct children, before
/// falling back to a downward scan rooted at `start`.
///
/// # Arguments
///
/// * `start` - The path where the search begins.
/// * `predicate` - The match function applied to candidate paths.
/// * `max_depth` - Maximum number of upward steps and maximum depth of the
///   downward walk.
/// * `traverse_upward` - Whether to search upward before scanning downward.
///
/// # Returns
///
/// The first matching path.
///
/// # Errors
///
/// Returns [`MoldXError2::PathNotFound`] if no path satisfies the predicate.
pub fn discover_path<F>(
    start: impl Into<PathBuf>,
    predicate: F,
    max_depth: usize,
    traverse_upward: bool,
) -> Result<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    let start = start.into();

    if traverse_upward {
        let mut current = start.clone();

        for _ in 0..=max_depth {
            if predicate(&current) {
                return Ok(current);
            }

            if let Ok(entries) = fs::read_dir(&current)
                && let Some(entry) = entries.flatten().find(|e| predicate(&e.path()))
            {
                return Ok(entry.path());
            }

            if !current.pop() {
                break;
            }
        }
    }

    for entry in WalkDir::new(&start)
        .max_depth(max_depth)
        .into_iter()
        .flatten()
    {
        if predicate(entry.path()) {
            return Ok(entry.into_path());
        }
    }

    bail!(MoldXError2::DiscoveryFailed { start, kind: ".moldx" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_sorted_read_dir() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("c.txt"), "").unwrap();
        fs::write(dir.path().join("a.txt"), "").unwrap();
        fs::write(dir.path().join("b.txt"), "").unwrap();
        let entries = sorted_read_dir(dir.path()).unwrap();
        let names: Vec<String> = entries
            .iter()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn test_sorted_read_dir_empty() {
        let dir = tempdir().unwrap();
        let entries = sorted_read_dir(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_sorted_read_dir_nonexistent() {
        let result = sorted_read_dir(Path::new("/nonexistent_path_12345"));
        assert!(result.is_err());
    }

    #[test]
    fn test_is_shell_script() {
        assert!(is_shell_script(Path::new("script.sh")));
        assert!(is_shell_script(Path::new("/path/to/run.sh")));
        assert!(!is_shell_script(Path::new("script.py")));
        assert!(!is_shell_script(Path::new("Makefile")));
        assert!(!is_shell_script(Path::new("noext")));
    }

    #[test]
    fn test_file_names_for_dir_with_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "").unwrap();
        fs::write(dir.path().join("b.rs"), "").unwrap();
        fs::write(dir.path().join(".hidden"), "").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        let names = file_names_for_dir(dir.path()).unwrap();
        assert_eq!(names.len(), 2);
        assert!(names.contains("a.rs"));
        assert!(names.contains("b.rs"));
        assert!(!names.contains(".hidden"));
        assert!(!names.contains("subdir"));
    }

    #[test]
    fn test_file_names_for_nested_files() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("nested/config")).unwrap();
        fs::write(dir.path().join("nested/config/app.toml"), "").unwrap();
        let names = file_names_for_dir(dir.path()).unwrap();
        assert!(names.contains("nested/config/app.toml"));
    }

    #[test]
    fn test_file_names_for_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("standalone.txt");
        fs::write(&file, "").unwrap();
        let names = file_names_for_dir(&file).unwrap();
        assert_eq!(names.len(), 1);
        assert!(names.contains("standalone.txt"));
    }

    #[test]
    fn test_file_names_for_empty_dir() {
        let dir = tempdir().unwrap();
        let names = file_names_for_dir(dir.path()).unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn test_discover_path_downward_finds_file() {
        let root = tempdir().unwrap();
        let sub = root.path().join("a").join("b").join("c");
        fs::create_dir_all(&sub).unwrap();
        let target = sub.join("target.txt");
        fs::write(&target, "").unwrap();
        let found = discover_path(
            root.path(),
            |p| p.file_name() == Some(OsStr::new("target.txt")),
            10,
            false,
        );
        assert!(found.is_ok());
        assert_eq!(found.unwrap(), target);
    }

    #[test]
    fn test_discover_path_not_found() {
        let root = tempdir().unwrap();
        let result = discover_path(
            root.path(),
            |p| p.file_name() == Some(OsStr::new("nope.txt")),
            2,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_path_upward_finds_dir() {
        let root = tempdir().unwrap();
        let sub = root.path().join("level1").join("level2");
        fs::create_dir_all(&sub).unwrap();
        fs::create_dir(root.path().join(".moldx")).unwrap();
        let found = discover_path(
            &sub,
            |p| p.file_name() == Some(OsStr::new(".moldx")),
            10,
            true,
        );
        assert!(found.is_ok());
    }

    #[test]
    fn test_resolve_name_dir() {
        let root = tempdir().unwrap();
        let dir = root.path().join("my-profile");
        fs::create_dir(&dir).unwrap();
        assert_eq!(resolve_name(&dir, Entity::Profile).unwrap(), "my-profile");
    }

    #[test]
    fn test_resolve_name_file_stem() {
        let root = tempdir().unwrap();
        let file = root.path().join("build.sh");
        fs::write(&file, "").unwrap();
        assert_eq!(resolve_name(&file, Entity::Command).unwrap(), "build");
    }

    #[test]
    fn test_resolve_name_nonexistent() {
        assert!(resolve_name(Path::new("/nonexistent/xx"), Entity::Module).is_err());
    }
}

use std::{collections::BTreeSet, path::Path};
use anyhow::Result;

pub fn sorted_read_dir(path: &Path) -> Result<Vec<std::fs::DirEntry>> {
    let mut entries = std::fs::read_dir(path)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(entries)
}

pub fn is_ignored_name(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules"
}

pub fn is_shell_script(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("sh")
}

pub fn file_names_for_dir(root: &Path) -> Result<BTreeSet<String>> {
    if root.is_file() {
        let mut names = BTreeSet::new();
        if let Some(name) = root.file_name().and_then(|name| name.to_str()) {
            names.insert(name.to_string());
        }
        return Ok(names);
    }

    let mut names = BTreeSet::new();
    for entry in sorted_read_dir(root)? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            if name.starts_with('.') {
                continue;
            }
            names.insert(name.to_string());
        }
    }

    Ok(names)
}

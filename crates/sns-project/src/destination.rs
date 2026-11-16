//! Destination identity for the host file adapters.
use std::path::PathBuf;

pub fn resolve_destination(path: &std::path::Path) -> Result<PathBuf, String> {
    // Resolve the nearest existing ancestor, including symlinks, before appending
    // missing directories. Preference storage may create its own parent folders.
    let mut cursor = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let mut missing = Vec::new();
    loop {
        match std::fs::symlink_metadata(&cursor) {
            Ok(_) => {
                let mut resolved = cursor.canonicalize().map_err(|e| e.to_string())?;
                for part in missing.into_iter().rev() {
                    if part == ".." {
                        resolved.pop();
                    } else {
                        resolved.push(part);
                        // A previously missing suffix can return to an existing
                        // symlink after `..`; resolve it before comparing owners.
                        match std::fs::symlink_metadata(&resolved) {
                            Ok(_) => {
                                resolved = resolved.canonicalize().map_err(|e| e.to_string())?
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                            Err(error) => return Err(error.to_string()),
                        }
                    }
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let name = match cursor.components().next_back() {
                    Some(std::path::Component::Normal(name)) => name.to_os_string(),
                    Some(std::path::Component::ParentDir) => std::ffi::OsString::from(".."),
                    _ => return Err("session.protected_path".into()),
                };
                missing.push(name);
                if !cursor.pop() {
                    return Err("session.protected_path".into());
                }
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}
pub fn same_destination(a: &std::path::Path, b: &std::path::Path) -> bool {
    resolve_destination(a)
        .ok()
        .zip(resolve_destination(b).ok())
        .is_some_and(|(a, b)| a == b)
}

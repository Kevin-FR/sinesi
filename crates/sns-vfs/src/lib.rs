//! Isolated in-memory filesystem. No path here is ever passed to the host filesystem.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Serialize;
use sns_model::{Diagnostic, VirtualFile};
use std::{collections::BTreeMap, sync::Arc};
pub const DIRECTORY_MEDIA_TYPE: &str = "inode/directory";
pub const MAX_ENTRIES: usize = 4096;
pub const MAX_HOST_BYTES: usize = 1_048_576;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
enum Node {
    Directory,
    File {
        media_type: String,
        bytes: Arc<[u8]>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Vfs {
    nodes: BTreeMap<String, Node>,
}
impl Default for Vfs {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::from([("/".into(), Node::Directory)]),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub path: String,
    pub directory: bool,
    pub media_type: String,
    pub bytes: usize,
}
fn error(code: &str, path: &str) -> Diagnostic {
    Diagnostic::new(code, path)
}
/// Resolve '.', '..' and relative paths within the virtual root. Host/URL syntax is rejected.
pub fn resolve(cwd: &str, input: &str) -> Result<String, Diagnostic> {
    if !sns_model::virtual_path_valid(cwd)
        || cwd.len() > 4096
        || input.is_empty()
        || input.len() > 4096
        || input.contains(['\\', ':', '\0', '%', '#'])
        || input.starts_with("//")
    {
        return Err(error("fs.invalid_path", input));
    }
    let combined = if input.starts_with('/') {
        input.to_owned()
    } else {
        format!("{cwd}/{input}")
    };
    let mut parts = Vec::new();
    for part in combined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    if parts.len() > 128 {
        return Err(error("resource.limit_exceeded", input));
    }
    let path = format!("/{}", parts.join("/"));
    if path.len() > 4096 {
        return Err(error("resource.limit_exceeded", input));
    }
    Ok(path)
}
fn canonical(path: &str) -> Result<String, Diagnostic> {
    let normalized = resolve("/", path)?;
    if !path.starts_with('/') || normalized != path {
        return Err(error("fs.invalid_path", path));
    }
    Ok(normalized)
}
fn parent(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(p, _)| if p.is_empty() { "/" } else { p })
        .unwrap_or("/")
}
impl Vfs {
    pub fn from_files(files: &[VirtualFile]) -> Result<Self, Diagnostic> {
        let mut vfs = Self::default();
        let mut explicit = std::collections::BTreeSet::new();
        for file in files {
            let path = canonical(&file.path)?;
            if path == "/" || !explicit.insert(path.clone()) {
                return Err(error("fs.already_exists", &path));
            }
            if file.media_type.len() > 4096
                || file.data_base64.len() > MAX_HOST_BYTES.div_ceil(3) * 4
            {
                return Err(error("resource.limit_exceeded", &path));
            }
            let data = STANDARD
                .decode(&file.data_base64)
                .map_err(|_| error("validation.invalid_base64", &path))?;
            let node = if file.media_type == DIRECTORY_MEDIA_TYPE {
                if !data.is_empty() {
                    return Err(error("fs.invalid_directory", &path));
                }
                Node::Directory
            } else {
                Node::File {
                    media_type: file.media_type.clone(),
                    bytes: data.into(),
                }
            };
            let mut missing = Vec::new();
            let mut directory = parent(&path);
            while !vfs.nodes.contains_key(directory) {
                missing.push(directory.to_owned());
                directory = parent(directory);
            }
            if vfs.nodes.get(directory) != Some(&Node::Directory) {
                return Err(error("fs.not_directory", directory));
            }
            for dir in missing.into_iter().rev() {
                vfs.nodes.insert(dir, Node::Directory);
            }
            if matches!(vfs.nodes.get(&path), Some(Node::Directory)) && node != Node::Directory {
                return Err(error("fs.is_directory", &path));
            }
            vfs.nodes.insert(path, node);
            vfs.validate()?;
        }
        Ok(vfs)
    }
    pub fn to_files(&self) -> Vec<VirtualFile> {
        self.nodes
            .iter()
            .filter(|(p, _)| p.as_str() != "/")
            .map(|(path, node)| match node {
                Node::Directory => VirtualFile {
                    path: path.clone(),
                    media_type: DIRECTORY_MEDIA_TYPE.into(),
                    data_base64: String::new(),
                },
                Node::File { media_type, bytes } => VirtualFile {
                    path: path.clone(),
                    media_type: media_type.clone(),
                    data_base64: STANDARD.encode(bytes),
                },
            })
            .collect()
    }
    pub fn is_directory(&self, path: &str) -> bool {
        self.nodes.get(path) == Some(&Node::Directory)
    }
    pub fn exists(&self, path: &str) -> bool {
        self.nodes.contains_key(path)
    }
    pub fn read(&self, path: &str) -> Result<&[u8], Diagnostic> {
        canonical(path)?;
        match self.nodes.get(path) {
            Some(Node::File { bytes, .. }) => Ok(bytes),
            Some(Node::Directory) => Err(error("fs.is_directory", path)),
            None => Err(error("fs.not_found", path)),
        }
    }
    pub fn read_text(&self, path: &str) -> Result<&str, Diagnostic> {
        std::str::from_utf8(self.read(path)?).map_err(|_| error("fs.not_text", path))
    }
    pub fn media_type(&self, path: &str) -> Result<&str, Diagnostic> {
        match self.nodes.get(path) {
            Some(Node::File { media_type, .. }) => Ok(media_type),
            Some(Node::Directory) => Ok(DIRECTORY_MEDIA_TYPE),
            None => Err(error("fs.not_found", path)),
        }
    }
    pub fn list(&self, path: &str) -> Result<Vec<Entry>, Diagnostic> {
        canonical(path)?;
        if !self.is_directory(path) {
            return Err(error(
                if self.exists(path) {
                    "fs.not_directory"
                } else {
                    "fs.not_found"
                },
                path,
            ));
        }
        Ok(self
            .nodes
            .iter()
            .filter(|(p, _)| p.as_str() != path && parent(p) == path)
            .map(|(path, node)| match node {
                Node::Directory => Entry {
                    path: path.clone(),
                    directory: true,
                    media_type: DIRECTORY_MEDIA_TYPE.into(),
                    bytes: 0,
                },
                Node::File { media_type, bytes } => Entry {
                    path: path.clone(),
                    directory: false,
                    media_type: media_type.clone(),
                    bytes: bytes.len(),
                },
            })
            .collect())
    }
    pub fn mkdir(&mut self, path: &str) -> Result<(), Diagnostic> {
        self.transaction(|v| {
            let path = canonical(path)?;
            v.require_new(&path)?;
            v.require_parent(&path)?;
            v.nodes.insert(path, Node::Directory);
            Ok(())
        })
    }
    pub fn write(
        &mut self,
        path: &str,
        data: &[u8],
        media_type: &str,
        overwrite: bool,
    ) -> Result<(), Diagnostic> {
        if data.len() > MAX_HOST_BYTES
            || media_type.len() > 4096
            || media_type == DIRECTORY_MEDIA_TYPE
        {
            return Err(error("resource.limit_exceeded", path));
        }
        self.transaction(|v| {
            let path = canonical(path)?;
            v.require_parent(&path)?;
            if v.is_directory(&path) {
                return Err(error("fs.is_directory", &path));
            }
            if !overwrite {
                v.require_new(&path)?;
            }
            v.nodes.insert(
                path,
                Node::File {
                    media_type: media_type.into(),
                    bytes: Arc::from(data),
                },
            );
            Ok(())
        })
    }
    pub fn copy(&mut self, source: &str, destination: &str) -> Result<(), Diagnostic> {
        self.transaction(|v| v.copy_inner(source, destination).map(|_| ()))
    }
    pub fn move_entry(&mut self, source: &str, destination: &str) -> Result<(), Diagnostic> {
        self.transaction(|v| {
            v.copy_inner(source, destination)?;
            v.remove_inner(source, true)
        })
    }
    pub fn remove(&mut self, path: &str, recursive: bool) -> Result<(), Diagnostic> {
        self.transaction(|v| v.remove_inner(path, recursive))
    }
    fn copy_inner(&mut self, source: &str, destination: &str) -> Result<String, Diagnostic> {
        canonical(source)?;
        canonical(destination)?;
        if source == "/" {
            return Err(error("fs.root_protected", source));
        }
        if !self.exists(source) {
            return Err(error("fs.not_found", source));
        }
        let destination = if self.is_directory(destination) {
            format!(
                "{}/{}",
                destination.trim_end_matches('/'),
                source.rsplit('/').next().unwrap()
            )
        } else {
            destination.to_owned()
        };
        if destination == source || destination.starts_with(&format!("{source}/")) {
            return Err(error("fs.invalid_destination", &destination));
        }
        self.require_new(&destination)?;
        self.require_parent(&destination)?;
        let entries: Vec<_> = self
            .nodes
            .iter()
            .filter(|(p, _)| p.as_str() == source || p.starts_with(&format!("{source}/")))
            .map(|(p, n)| (format!("{destination}{}", &p[source.len()..]), n.clone()))
            .collect();
        for (path, node) in entries {
            canonical(&path)?;
            self.require_new(&path)?;
            self.nodes.insert(path, node);
        }
        Ok(destination)
    }
    fn remove_inner(&mut self, path: &str, recursive: bool) -> Result<(), Diagnostic> {
        canonical(path)?;
        if path == "/" {
            return Err(error("fs.root_protected", path));
        }
        if !self.exists(path) {
            return Err(error("fs.not_found", path));
        }
        let prefix = format!("{path}/");
        if !recursive && self.nodes.keys().any(|p| p.starts_with(&prefix)) {
            return Err(error("fs.directory_not_empty", path));
        }
        self.nodes
            .retain(|p, _| p != path && !p.starts_with(&prefix));
        Ok(())
    }
    fn require_new(&self, path: &str) -> Result<(), Diagnostic> {
        if self.exists(path) {
            Err(error("fs.already_exists", path))
        } else {
            Ok(())
        }
    }
    fn require_parent(&self, path: &str) -> Result<(), Diagnostic> {
        if self.is_directory(parent(path)) {
            Ok(())
        } else {
            Err(error("fs.not_directory", parent(path)))
        }
    }
    fn validate(&self) -> Result<(), Diagnostic> {
        if self.nodes.len() - 1 > MAX_ENTRIES
            || self
                .nodes
                .values()
                .map(|n| match n {
                    Node::Directory => 0,
                    Node::File { bytes, .. } => bytes.len(),
                })
                .sum::<usize>()
                > MAX_HOST_BYTES
        {
            return Err(error("resource.limit_exceeded", "filesystem"));
        }
        Ok(())
    }
    fn transaction(
        &mut self,
        action: impl FnOnce(&mut Self) -> Result<(), Diagnostic>,
    ) -> Result<(), Diagnostic> {
        let mut next = self.clone();
        action(&mut next)?;
        next.validate()?;
        *self = next;
        Ok(())
    }
}

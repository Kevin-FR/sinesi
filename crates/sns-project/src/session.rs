//! Recovery documents never replace a user project. A companion OS file lock
//! protects each live instance; the JSON file itself is replaced atomically.
use super::*;
use serde::{Deserialize, Serialize};
use sns_model::{Decimal, Id};
use std::{
    collections::BTreeSet,
    fs::TryLockError,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
pub const MAX_EDIT_STATE_BYTES: usize = 4 * 1_024 * 1_024;
pub const MAX_RECOVERY_BYTES: u64 = MAX_DOCUMENT_BYTES + MAX_EDIT_STATE_BYTES as u64 + 65_536;
pub const MAX_RECOVERY_ENTRIES: usize = 32;
pub const MAX_SESSION_FILES: usize = 128;
pub const MAX_SCAN_FILES: usize = 256;
static NEXT_SESSION: AtomicU64 = AtomicU64::new(0);
/// Explicit release also handles an open file description briefly inherited by
/// a concurrently spawning child before its close-on-exec descriptors close.
struct LockedFile(File);
impl Drop for LockedFile {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionView {
    pub diagram: bool,
    pub zoom: f32,
    pub pan: [f32; 2],
    pub selected: Vec<Id>,
}
impl Default for SessionView {
    fn default() -> Self {
        Self {
            diagram: false,
            zoom: 1.,
            pan: [0., 0.],
            selected: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRecord {
    pub session_version: u32,
    pub session_id: Id,
    pub saved_at_unix_ms: Decimal,
    pub revision: Decimal,
    /// Display only. Recovery always creates an unsaved document; never open or write this path.
    pub origin_display: Option<String>,
    pub project: Project,
    pub view: SessionView,
    /// Versioned, inactive editor buffers. They are never executed or applied by this adapter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_state: Option<serde_json::Value>,
}
impl SessionRecord {
    pub fn validate(&self) -> Result<(), Error> {
        self.project.validate()?;
        if let Some(state) = &self.edit_state
            && (!state.is_object() || serde_json::to_vec(state)?.len() > MAX_EDIT_STATE_BYTES)
        {
            return Err(Diagnostic::new("resource.limit_exceeded", "session_edit_state").into());
        }
        if self.session_version != 1 {
            return Err(Diagnostic::new("session.unsupported_version", "session").into());
        }
        if !valid_session_id(self.session_id.as_str())
            || self.origin_display.as_ref().is_some_and(|p| p.len() > 4096)
            || !self.view.zoom.is_finite()
            || !(0.25..=3.).contains(&self.view.zoom)
            || self
                .view
                .pan
                .iter()
                .any(|p| !p.is_finite() || p.abs() > 10_000_000.)
            || self.view.selected.len()
                > self.project.devices.len()
                    + self.project.links.len()
                    + self.project.annotations.len()
        {
            return Err(Diagnostic::new("session.invalid_document", "session").into());
        }
        let mut seen = BTreeSet::new();
        for id in &self.view.selected {
            if !seen.insert(id)
                || !(self.project.device(id).is_some()
                    || self.project.links.iter().any(|l| &l.id == id)
                    || self.project.annotations.iter().any(|a| &a.id == id))
            {
                return Err(Diagnostic::new("session.invalid_document", id.as_str()).into());
            }
        }
        limit(&serde_json::to_vec(&self.project)?)?;
        Ok(())
    }
}
fn valid_session_id(id: &str) -> bool {
    id.starts_with("session-")
        && id.len() <= 100
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
fn regular(path: &Path) -> Result<bool, Error> {
    match fs::symlink_metadata(path) {
        Ok(m) => Ok(m.file_type().is_file()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}
fn directory_lock(root: &Path) -> Result<LockedFile, Error> {
    let path = root.join(".allocation.lock");
    if path.exists() && !regular(&path)? {
        return Err(Diagnostic::new("session.invalid_document", "allocation_lock").into());
    }
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    for _ in 0..50 {
        match file.try_lock() {
            Ok(()) => return Ok(LockedFile(file)),
            Err(TryLockError::WouldBlock) => {
                std::thread::sleep(std::time::Duration::from_millis(5))
            }
            Err(e) => return Err(lock_error(e)),
        }
    }
    Err(Diagnostic::new("session.in_use", "allocation").into())
}
pub fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

pub struct SessionLease {
    id: Id,
    root: PathBuf,
    _lock: LockedFile,
}
impl SessionLease {
    pub fn create(root: &Path) -> Result<Self, Error> {
        fs::create_dir_all(root)?;
        let root = root.canonicalize()?;
        let _allocation = directory_lock(&root)?;
        // A finite directory scan refuses pathological caches; no project is deleted.
        if fs::read_dir(&root)?.take(MAX_SESSION_FILES + 1).count() + 2 > MAX_SESSION_FILES {
            return Err(Diagnostic::new("resource.limit_exceeded", "session_directory").into());
        }
        for _ in 0..32 {
            let serial = NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let id = Id::new(format!("session-{}-{nanos}-{serial}", std::process::id()))?;
            let path = root.join(format!("{}.lock", id.as_str()));
            match File::create_new(&path) {
                Ok(lock) => {
                    lock.try_lock().map_err(lock_error)?;
                    match File::create_new(root.join(format!("{}.json", id.as_str()))) {
                        Ok(reservation) => drop(reservation),
                        Err(e) => {
                            drop(lock);
                            let _ = fs::remove_file(&path);
                            if e.kind() == std::io::ErrorKind::AlreadyExists {
                                continue;
                            }
                            return Err(e.into());
                        }
                    }
                    return Ok(Self {
                        id,
                        root,
                        _lock: LockedFile(lock),
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Err(Diagnostic::new("resource.limit_exceeded", "session_identity").into())
    }
    pub fn id(&self) -> &Id {
        &self.id
    }
    pub fn path(&self) -> PathBuf {
        self.root.join(format!("{}.json", self.id.as_str()))
    }
    pub fn acquire(root: &Path, id: &Id) -> Result<Self, Error> {
        if !valid_session_id(id.as_str()) {
            return Err(Diagnostic::new("session.invalid_document", "session_id").into());
        }
        let root = root.canonicalize()?;
        let path = root.join(format!("{}.lock", id.as_str()));
        if !regular(&path)? {
            return Err(Diagnostic::new("session.invalid_document", "session_lock").into());
        }
        let lock = File::options().read(true).write(true).open(path)?;
        lock.try_lock().map_err(lock_error)?;
        Ok(Self {
            id: id.clone(),
            root,
            _lock: LockedFile(lock),
        })
    }
    pub fn save(&self, record: &SessionRecord) -> Result<(), Error> {
        record.validate()?;
        if record.session_id != self.id {
            return Err(Diagnostic::new("session.invalid_document", "session_id").into());
        }
        let bytes = serde_json::to_vec(record)?;
        if bytes.len() as u64 > MAX_RECOVERY_BYTES {
            return Err(Diagnostic::new("resource.limit_exceeded", "session_document").into());
        }
        write_atomic(&self.path(), &bytes)
    }
    pub fn read(&self) -> Result<SessionRecord, Error> {
        let path = self.path();
        if !regular(&path)? {
            return Err(
                Diagnostic::new("session.invalid_document", "regular_file_required").into(),
            );
        }
        let bytes = read_with_limit(&path, MAX_RECOVERY_BYTES)?;
        let record: SessionRecord = serde_json::from_slice(&bytes)?;
        record.validate()?;
        if record.session_id != self.id {
            return Err(Diagnostic::new("session.invalid_document", "session_id").into());
        }
        Ok(record)
    }
    /// The lease is held while discarding; active instances cannot be selected here.
    pub fn discard(self) -> Result<(), Error> {
        match fs::remove_file(self.path()) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        let lock = self.root.join(format!("{}.lock", self.id.as_str()));
        drop(self);
        match fs::remove_file(lock) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
fn lock_error(e: TryLockError) -> Error {
    match e {
        TryLockError::WouldBlock => Diagnostic::new("session.in_use", "session").into(),
        TryLockError::Error(e) => e.into(),
    }
}

pub struct Recovery {
    pub lease: SessionLease,
    pub record: SessionRecord,
}
#[derive(Clone, Debug)]
pub struct SessionSummary {
    pub id: Id,
    pub name: String,
    pub origin_display: Option<String>,
    pub saved_at_unix_ms: u64,
}
pub fn summaries(root: &Path) -> Result<(Vec<SessionSummary>, Vec<String>), Error> {
    let mut summaries = Vec::new();
    let mut diagnostics = visit(root, |lease, record| {
        summaries.push(SessionSummary {
            id: lease.id().clone(),
            name: record.project.name,
            origin_display: record.origin_display,
            saved_at_unix_ms: record.saved_at_unix_ms.0,
        });
    })?;
    summaries.sort_by(|a, b| {
        b.saved_at_unix_ms
            .cmp(&a.saved_at_unix_ms)
            .then_with(|| a.id.cmp(&b.id))
    });
    if summaries.len() > MAX_RECOVERY_ENTRIES {
        diagnostics.push("session.more_available".into());
        summaries.truncate(MAX_RECOVERY_ENTRIES);
    }
    Ok((summaries, diagnostics))
}
#[derive(Default)]
pub struct RecoveryScan {
    pub recoveries: Vec<Recovery>,
    pub diagnostics: Vec<String>,
}
/// Candidates are exclusively leased until the caller recovers, discards or drops them.
pub fn scan(root: &Path) -> Result<RecoveryScan, Error> {
    let mut scan = RecoveryScan::default();
    scan.diagnostics = visit(root, |lease, record| {
        scan.recoveries.push(Recovery { lease, record })
    })?;
    scan.recoveries.sort_by(|a, b| {
        b.record
            .saved_at_unix_ms
            .0
            .cmp(&a.record.saved_at_unix_ms.0)
            .then_with(|| a.record.session_id.cmp(&b.record.session_id))
    });
    if scan.recoveries.len() > MAX_RECOVERY_ENTRIES {
        scan.diagnostics.push("session.more_available".into());
        scan.recoveries.truncate(MAX_RECOVERY_ENTRIES);
    }
    Ok(scan)
}
fn visit(
    root: &Path,
    mut accept: impl FnMut(SessionLease, SessionRecord),
) -> Result<Vec<String>, Error> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let root = root.canonicalize()?;
    let mut diagnostics = Vec::new();
    let mut entries = fs::read_dir(&root)?
        .take(MAX_SCAN_FILES + 1)
        .collect::<Result<Vec<_>, _>>()?;
    if entries.len() > MAX_SCAN_FILES {
        return Err(Diagnostic::new("resource.limit_exceeded", "session_directory").into());
    }
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        // Empty companion locks without data are abandoned initializations, not snapshots.
        if path.extension().and_then(|s| s.to_str()) == Some("lock")
            && regular(&path)?
            && let Some(name) = path
                .file_stem()
                .and_then(|s| s.to_str())
                .filter(|s| valid_session_id(s))
            && fs::metadata(&path)?.len() == 0
            && !root.join(format!("{name}.json")).exists()
        {
            if let Ok(_allocation) = directory_lock(&root)
                && !root.join(format!("{name}.json")).exists()
                && let Ok(lease) = SessionLease::acquire(&root, &Id::new(name)?)
            {
                lease.discard()?;
            }
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("json") || !regular(&path)? {
            continue;
        }
        let Some(name) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| valid_session_id(s))
        else {
            continue;
        };
        let lock_path = root.join(format!("{name}.lock"));
        if !regular(&lock_path)? {
            diagnostics.push("session.invalid_document".into());
            continue;
        }
        let lock = match File::options().read(true).write(true).open(&lock_path) {
            Ok(f) => f,
            Err(e) => {
                diagnostics.push(e.to_string());
                continue;
            }
        };
        match lock.try_lock() {
            Err(TryLockError::WouldBlock) => continue,
            Err(e) => {
                diagnostics.push(lock_error(e).to_string());
                continue;
            }
            Ok(()) => {}
        }
        let lease = SessionLease {
            id: Id::new(name)?,
            root: root.clone(),
            _lock: LockedFile(lock),
        };
        if fs::metadata(&path)?.len() == 0 {
            if let Ok(_allocation) = directory_lock(&root) {
                lease.discard()?;
            }
            continue;
        }
        match lease.read() {
            Ok(record) => accept(lease, record),
            Err(e) => diagnostics.push(e.to_string()),
        }
    }
    Ok(diagnostics)
}

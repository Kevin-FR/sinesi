//! Bounded technical events shared by the launcher and workbench.
use serde::Serialize;
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Write},
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

const CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Startup,
    ConfigurationResolved,
    ConfigurationFallback,
    ProjectReady,
    ProjectLoadFailed,
    ProjectSaveFailed,
    ImportFailed,
    RuntimeFileFailed,
    SimulationFailed,
    PreferencesWriteFailed,
    ExportFailed,
    DiagnosticExported,
    WindowReady,
    WindowFailed,
    ReportFailed,
    Exit,
}

impl Event {
    fn debug(self) -> bool {
        matches!(
            self,
            Self::ConfigurationResolved | Self::ProjectReady | Self::WindowReady
        )
    }
}

#[derive(Clone, Copy, Serialize)]
struct Entry {
    sequence: u64,
    event: Event,
}

#[derive(Default)]
struct State {
    sequence: u64,
    entries: VecDeque<Entry>,
    verbose: bool,
    output: Option<mpsc::SyncSender<Entry>>,
}

#[derive(Clone, Default)]
pub struct Journal(Arc<Mutex<State>>);

impl Journal {
    pub fn record(&self, event: Event) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if event.debug() && !state.verbose {
            return;
        }
        state.sequence = state.sequence.saturating_add(1);
        let entry = Entry {
            sequence: state.sequence,
            event,
        };
        if state.entries.len() == CAPACITY {
            state.entries.pop_front();
        }
        state.entries.push_back(entry);
        if entry.sequence <= CAPACITY as u64
            && let Some(output) = &state.output
        {
            // At most CAPACITY entries are ever queued. Disk writes stay off the UI thread.
            let _ = output.try_send(entry);
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        Snapshot {
            diagnostic_version: 1,
            discarded: state.sequence.saturating_sub(state.entries.len() as u64),
            entries: state.entries.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct Snapshot {
    diagnostic_version: u8,
    discarded: u64,
    entries: VecDeque<Entry>,
}

impl Snapshot {
    pub fn write_to(&self, path: &std::path::Path) -> Result<(), sns_project::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        sns_project::atomic_write(path, &bytes)
    }
}

pub struct Logging {
    journal: Journal,
    writer: Option<JoinHandle<io::Result<()>>>,
}

impl Logging {
    pub fn start(verbose: bool, mut file: Option<File>) -> io::Result<Self> {
        let journal = Journal::default();
        let writer = if verbose || file.is_some() {
            let (sender, receiver) = mpsc::sync_channel::<Entry>(CAPACITY);
            let mut state = journal.0.lock().unwrap_or_else(|e| e.into_inner());
            state.verbose = verbose;
            state.output = Some(sender);
            Some(
                thread::Builder::new()
                    .name("sinesi-diagnostics".into())
                    .spawn(move || {
                        for entry in receiver {
                            let mut line = serde_json::to_vec(&entry)?;
                            line.push(b'\n');
                            if let Some(file) = &mut file {
                                file.write_all(&line)?;
                            }
                            if verbose {
                                // stderr is optional, including for GUI launches without a console.
                                let _ = io::stderr().lock().write_all(&line);
                            }
                        }
                        if let Some(file) = file {
                            file.sync_all()?;
                        }
                        Ok(())
                    })?,
            )
        } else {
            None
        };
        Ok(Self { journal, writer })
    }

    pub fn journal(&self) -> Journal {
        self.journal.clone()
    }

    pub fn finish(self) -> io::Result<()> {
        self.journal
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .output
            .take();
        if let Some(writer) = self.writer {
            writer
                .join()
                .map_err(|_| io::Error::other("diagnostic writer stopped"))??;
        }
        Ok(())
    }
}

/// Claim a new file and reject aliases of other launch destinations.
pub fn open_log(
    path: &std::path::Path,
    protected: &[&std::path::Path],
    session_root: Option<&std::path::Path>,
) -> Result<(std::path::PathBuf, File), &'static str> {
    use sns_project::destination::resolve_destination;
    let resolve = |path: &std::path::Path| {
        resolve_destination(path).map_err(|_| "diagnostic.log_open_failed")
    };
    let target = resolve(path)?;
    for path in protected {
        if target == resolve(path)? {
            return Err("diagnostic.log_conflict");
        }
    }
    if let Some(root) = session_root
        && target.starts_with(resolve(root)?)
    {
        return Err("diagnostic.log_conflict");
    }
    let file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| "diagnostic.log_open_failed")?;
    Ok((target, file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_recent_events_even_when_disk_reaches_its_limit() {
        let root = tempfile::tempdir().unwrap();
        let log = root.path().join("technical.jsonl");
        let (path, file) = open_log(&log, &[], None).unwrap();
        let logging = Logging::start(false, Some(file)).unwrap();
        let journal = logging.journal();
        for _ in 0..300 {
            journal.record(Event::Startup);
        }
        journal.record(Event::RuntimeFileFailed);
        logging.finish().unwrap();
        let output = std::fs::read_to_string(path).unwrap();
        assert_eq!(output.lines().count(), CAPACITY);
        let captured = serde_json::to_value(journal.snapshot()).unwrap();
        let schema: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../contracts/diagnostic.schema.json"))
                .unwrap();
        let contract = jsonschema::validator_for(&schema).unwrap();
        contract.validate(&captured).unwrap();
        assert_eq!(captured["discarded"], 45);
        assert_eq!(captured["entries"][0]["sequence"], 46);
        assert_eq!(captured["entries"][255]["event"], "runtime_file_failed");
    }
}

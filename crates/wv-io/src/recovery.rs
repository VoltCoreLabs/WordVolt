//! Crash recovery: unsaved documents are written to a folder every now and then and offered
//! again at the next start.
//!
//! Every running instance of the app has an id and touches a heartbeat file; a document file
//! (`<session>.wvd` plus `<session>.meta` with the instance id and the original path) belongs to
//! the instance that wrote it. Documents of an instance whose heartbeat is fresh are not touched
//! (another window is still running); the others are what a crash left behind.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use wv_core::Document;

use crate::IoError;

/// A document that was recovered.
pub struct Recovered {
    pub session: u64,
    /// File the document came from; `None` for a new, never saved document.
    pub path: Option<PathBuf>,
    pub doc: Document,
}

/// `<local data>/WordVolt/recovery` (LOCALAPPDATA on Windows, Application Support on macOS, XDG data
/// on Linux).
pub fn default_dir() -> PathBuf {
    let home = || std::env::var_os("HOME").map(PathBuf::from);
    let base = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        home().map(|h| h.join("Library").join("Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home().map(|h| h.join(".local").join("share")))
    };
    base.unwrap_or_else(std::env::temp_dir)
        .join("WordVolt")
        .join("recovery")
}

/// A fresh id (for an instance or a document session).
pub fn new_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    nanos
        .rotate_left(17)
        .wrapping_add(u64::from(std::process::id()) << 32)
        .wrapping_add(COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn session_file(dir: &Path, session: u64, ext: &str) -> PathBuf {
    dir.join(format!("{session:016x}.{ext}"))
}

fn heartbeat_file(dir: &Path, instance: u64) -> PathBuf {
    dir.join(format!("{instance:016x}.alive"))
}

/// Shows that this instance is running.
pub fn heartbeat(dir: &Path, instance: u64) {
    if fs::create_dir_all(dir).is_ok() {
        let _ = fs::write(heartbeat_file(dir, instance), b"");
    }
}

/// Writes the document of `session` (replacing the earlier copy).
pub fn write(
    dir: &Path,
    instance: u64,
    session: u64,
    path: Option<&Path>,
    doc: &Document,
) -> Result<(), IoError> {
    fs::create_dir_all(dir)?;
    let meta = format!(
        "{instance:016x}\n{}",
        path.map(|p| p.display().to_string()).unwrap_or_default()
    );
    fs::write(session_file(dir, session, "meta"), meta)?;
    let bytes = crate::wvd::encode(doc)?;
    let tmp = session_file(dir, session, "tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, session_file(dir, session, "wvd"))?;
    Ok(())
}

/// Forgets the recovery copy of `session` (the document was saved or closed on purpose).
pub fn remove(dir: &Path, session: u64) {
    for ext in ["wvd", "meta", "tmp"] {
        let _ = fs::remove_file(session_file(dir, session, ext));
    }
}

/// The instance ends normally.
pub fn end_instance(dir: &Path, instance: u64) {
    let _ = fs::remove_file(heartbeat_file(dir, instance));
}

fn is_fresh(path: &Path, max_age: Duration) -> bool {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < max_age)
}

/// Documents left by instances that are no longer running (no heartbeat for `stale_after`).
/// Unreadable copies are deleted.
pub fn scan(dir: &Path, stale_after: Duration) -> Vec<Recovered> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        // Marks of windows that ended long ago (killed, crashed) are litter.
        if ext == Some("alive") && !is_fresh(&path, Duration::from_secs(24 * 3600)) {
            let _ = fs::remove_file(&path);
        }
        if ext != Some("wvd") {
            continue;
        }
        let Some(session) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| u64::from_str_radix(s, 16).ok())
        else {
            continue;
        };
        let meta = fs::read_to_string(session_file(dir, session, "meta")).unwrap_or_default();
        let mut lines = meta.lines();
        let instance = lines.next().and_then(|l| u64::from_str_radix(l, 16).ok());
        if instance.is_some_and(|i| is_fresh(&heartbeat_file(dir, i), stale_after)) {
            continue;
        }
        let original = lines.next().filter(|l| !l.is_empty()).map(PathBuf::from);
        match fs::read(&path)
            .map_err(IoError::from)
            .and_then(|b| crate::wvd::decode(&b))
        {
            Ok(doc) => found.push(Recovered {
                session,
                path: original,
                doc,
            }),
            Err(_) => remove(dir, session),
        }
    }
    found.sort_by_key(|r| r.session);
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wv-recovery-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn an_unfinished_session_comes_back_with_its_path() {
        let dir = temp("back");
        let doc = Document::from_text("Привет, мир");
        write(&dir, 1, 7, Some(Path::new("C:/docs/a.docx")), &doc).unwrap();
        write(&dir, 1, 8, None, &Document::from_text("new")).unwrap();
        let found = scan(&dir, Duration::from_secs(30));
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].path.as_deref(), Some(Path::new("C:/docs/a.docx")));
        assert_eq!(found[0].doc.text(), "Привет, мир");
        assert!(found[1].path.is_none());
        remove(&dir, 7);
        assert_eq!(scan(&dir, Duration::from_secs(30)).len(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn documents_of_a_running_instance_are_left_alone() {
        let dir = temp("alive");
        write(&dir, 5, 9, None, &Document::from_text("x")).unwrap();
        heartbeat(&dir, 5);
        assert!(scan(&dir, Duration::from_secs(30)).is_empty());
        // No heartbeat any more (it ended or crashed): the copy is offered.
        end_instance(&dir, 5);
        assert_eq!(scan(&dir, Duration::from_secs(30)).len(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_broken_copy_is_dropped() {
        let dir = temp("broken");
        fs::create_dir_all(&dir).unwrap();
        fs::write(session_file(&dir, 3, "wvd"), b"not json").unwrap();
        assert!(scan(&dir, Duration::from_secs(30)).is_empty());
        assert!(!session_file(&dir, 3, "wvd").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn old_heartbeat_marks_are_cleaned_up() {
        let dir = temp("litter");
        heartbeat(&dir, 1);
        heartbeat(&dir, 2);
        let old = SystemTime::now() - Duration::from_secs(3 * 24 * 3600);
        let file = fs::File::options()
            .write(true)
            .open(heartbeat_file(&dir, 1))
            .unwrap();
        file.set_modified(old).unwrap();
        drop(file);
        scan(&dir, Duration::from_secs(30));
        assert!(!heartbeat_file(&dir, 1).exists());
        assert!(heartbeat_file(&dir, 2).exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn ids_differ() {
        assert_ne!(new_id(), new_id());
    }
}

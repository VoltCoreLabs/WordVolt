use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// User settings stored next to the other per-user app data.
#[derive(Serialize, Deserialize)]
struct Settings {
    #[serde(default)]
    recent_colors: Vec<u32>,
    /// Pages sit side by side while they fit (false: always one page per row).
    #[serde(default = "yes")]
    multi_page: bool,
    /// Misspelled words are underlined.
    #[serde(default = "yes")]
    spellcheck: bool,
}

fn yes() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            recent_colors: Vec::new(),
            multi_page: true,
            spellcheck: true,
        }
    }
}

/// `<config dir>/WordVolt/settings.json` (APPDATA on Windows, Application Support on macOS, XDG on Linux).
fn file() -> Option<PathBuf> {
    let home = || std::env::var_os("HOME").map(PathBuf::from);
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        home().map(|h| h.join("Library").join("Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| home().map(|h| h.join(".config")))
    };
    base.map(|b| b.join("WordVolt").join("settings.json"))
}

pub fn load_recent_colors() -> Vec<u32> {
    file().map(|p| read(&p).recent_colors).unwrap_or_default()
}

pub fn save_recent_colors(colors: &[u32]) {
    if let Some(path) = file() {
        let _ = update(&path, |s| s.recent_colors = colors.to_vec());
    }
}

pub fn load_multi_page() -> bool {
    file().is_none_or(|p| read(&p).multi_page)
}

pub fn save_multi_page(on: bool) {
    if let Some(path) = file() {
        let _ = update(&path, |s| s.multi_page = on);
    }
}

pub fn load_spellcheck() -> bool {
    file().is_none_or(|p| read(&p).spellcheck)
}

pub fn save_spellcheck(on: bool) {
    if let Some(path) = file() {
        let _ = update(&path, |s| s.spellcheck = on);
    }
}

/// `<config dir>/WordVolt/userdict.txt`: the words the user added, one per line.
fn user_dict_file() -> Option<PathBuf> {
    file().map(|p| p.with_file_name("userdict.txt"))
}

pub fn load_user_words() -> Vec<String> {
    user_dict_file()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|text| {
            text.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

pub fn append_user_word(word: &str) {
    use std::io::Write;
    if let Some(path) = user_dict_file() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "{word}");
        }
    }
}

fn read(path: &Path) -> Settings {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Reads the settings, changes them and writes them back (other settings stay as they were).
fn update(path: &Path, change: impl FnOnce(&mut Settings)) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut settings = read(path);
    change(&mut settings);
    let json = serde_json::to_vec_pretty(&settings).map_err(io::Error::other)?;
    let temp = path.with_extension("tmp");
    std::fs::write(&temp, json)?;
    std::fs::rename(&temp, path)
}

#[cfg(test)]
mod tests {
    use super::{read, update};

    #[test]
    fn settings_round_trip_keep_each_other_and_bad_files_are_ignored() {
        let dir = std::env::temp_dir().join(format!("wv-settings-{}", std::process::id()));
        let path = dir.join("sub").join("settings.json");
        assert!(read(&path).recent_colors.is_empty() && read(&path).multi_page);
        update(&path, |s| s.recent_colors = vec![0xFF0000, 0x2563EB]).unwrap();
        update(&path, |s| s.multi_page = false).unwrap();
        let now = read(&path);
        assert_eq!(now.recent_colors, vec![0xFF0000, 0x2563EB]);
        assert!(!now.multi_page);
        std::fs::write(&path, "not json").unwrap();
        assert!(read(&path).recent_colors.is_empty() && read(&path).multi_page);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

/// Name of the signed-in user (author of comments and tracked changes): `USERNAME` on Windows,
/// `USER` elsewhere.
pub fn user_name() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default()
}

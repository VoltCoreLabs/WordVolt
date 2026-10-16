use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// User settings stored next to the other per-user app data.
#[derive(Serialize, Deserialize, Default)]
struct Settings {
    #[serde(default)]
    recent_colors: Vec<u32>,
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
    file().map(|p| read_recent(&p)).unwrap_or_default()
}

pub fn save_recent_colors(colors: &[u32]) {
    if let Some(path) = file() {
        let _ = write_recent(&path, colors);
    }
}

fn read_recent(path: &Path) -> Vec<u32> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    serde_json::from_slice::<Settings>(&bytes)
        .map(|s| s.recent_colors)
        .unwrap_or_default()
}

fn write_recent(path: &Path, colors: &[u32]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_vec_pretty(&Settings {
        recent_colors: colors.to_vec(),
    })
    .map_err(io::Error::other)?;
    let temp = path.with_extension("tmp");
    std::fs::write(&temp, json)?;
    std::fs::rename(&temp, path)
}

#[cfg(test)]
mod tests {
    use super::{read_recent, write_recent};

    #[test]
    fn recent_colors_round_trip_and_bad_files_are_ignored() {
        let dir = std::env::temp_dir().join(format!("wv-settings-{}", std::process::id()));
        let path = dir.join("sub").join("settings.json");
        assert!(read_recent(&path).is_empty());
        write_recent(&path, &[0xFF0000, 0x2563EB]).unwrap();
        assert_eq!(read_recent(&path), vec![0xFF0000, 0x2563EB]);
        std::fs::write(&path, "not json").unwrap();
        assert!(read_recent(&path).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

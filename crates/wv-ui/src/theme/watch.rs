use gpui_kit::App;

/// Dev only: reloads `assets/theme/light.toml` on save, without a rebuild.
#[cfg(debug_assertions)]
pub fn watch_theme_file(cx: &mut App) {
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::time::Duration;

    use notify::{RecursiveMode, Watcher};

    use super::Theme;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/theme/light.toml");
    let (tx, rx) = mpsc::channel();
    let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if res.is_ok() {
            let _ = tx.send(());
        }
    });
    let Ok(mut watcher) = watcher else { return };
    if watcher.watch(&path, RecursiveMode::NonRecursive).is_err() {
        return;
    }
    cx.spawn(async move |cx| {
        let _keep_alive = watcher;
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(150))
                .await;
            if rx.try_iter().count() == 0 {
                continue;
            }
            match std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|s| Theme::parse(&s))
            {
                Ok(theme) => {
                    cx.update(|cx| {
                        cx.set_global(theme);
                        cx.refresh_windows();
                    });
                }
                Err(err) => eprintln!("theme reload failed: {err}"),
            }
        }
    })
    .detach();
}

#[cfg(not(debug_assertions))]
pub fn watch_theme_file(_cx: &mut App) {}

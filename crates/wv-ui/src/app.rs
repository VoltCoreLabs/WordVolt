use gpui_kit::*;

use crate::assets::{Assets, register_fonts};
use crate::theme::{Theme, watch_theme_file};
use crate::window::MainWindow;

/// Starts the application and opens the main window.
pub fn run() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        register_fonts(cx);
        cx.set_global(Theme::light());
        crate::ui_kit::init(cx);
        crate::actions::bind_keys(cx);
        watch_theme_file(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1280.), px(800.)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some(wv_i18n::APP_NAME.into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(16.), px(14.))),
            }),
            window_decorations: Some(WindowDecorations::Client),
            window_min_size: Some(size(px(720.), px(480.))),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| {
            let view = cx.new(MainWindow::new);
            // `wordvolt <file>` opens that file.
            if let Some(path) = std::env::args_os().nth(1).map(std::path::PathBuf::from)
                && let Ok(doc) = wv_io::load_file(&path)
            {
                view.update(cx, |this, cx| this.install(doc, Some(path), cx));
            }
            let weak = view.downgrade();
            window.on_window_should_close(cx, move |window, cx| {
                weak.update(cx, |this, cx| this.should_close(window, cx))
                    .unwrap_or(true)
            });
            view
        })
        .expect("failed to open window");
        cx.activate(true);
    });
}

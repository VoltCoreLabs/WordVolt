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
        watch_theme_file(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1280.), px(800.)),
                cx,
            ))),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| MainWindow::new()))
            .expect("failed to open window");
        cx.activate(true);
    });
}

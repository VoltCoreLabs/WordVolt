use gpui_kit::*;

use crate::theme::Theme;
use crate::window::MainWindow;

/// Starts the application and opens the main window.
pub fn run() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        cx.set_global(Theme::light());
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1280.), px(800.)),
                cx,
            ))),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| MainWindow))
            .expect("failed to open window");
        cx.activate(true);
    });
}

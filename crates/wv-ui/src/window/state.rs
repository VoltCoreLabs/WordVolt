use wv_i18n::{APP_NAME, Key, Lang, t};

/// Ribbon tabs.
#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Home,
    Insert,
    Layout,
    View,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Home, Tab::Insert, Tab::Layout, Tab::View];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Home => "Home",
            Tab::Insert => "Insert",
            Tab::Layout => "Layout",
            Tab::View => "View",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Align {
    Left,
    Center,
    Right,
    Justify,
}

/// UI state of the main window (document state lives in wv-core).
pub struct MainWindow {
    /// `None` = document was never saved (shown as "New document").
    pub file_name: Option<String>,
    pub dirty: bool,
    pub lang: Lang,
    pub applied_title: String,
    pub tab: Tab,
    pub ribbon_open: bool,
    pub panel_open: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub align: Align,
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            file_name: None,
            dirty: false,
            lang: Lang::default(),
            applied_title: String::new(),
            tab: Tab::Home,
            ribbon_open: true,
            panel_open: false,
            bold: false,
            italic: false,
            underline: false,
            align: Align::Left,
        }
    }
}

impl MainWindow {
    /// File name, or "New document" if there is none; `•` marks unsaved changes.
    pub fn document_title(&self) -> String {
        let name = self
            .file_name
            .clone()
            .unwrap_or_else(|| t(self.lang, Key::UntitledDocument).to_string());
        if self.dirty {
            format!("{name} •")
        } else {
            name
        }
    }

    /// Title for the OS (taskbar, alt-tab).
    pub fn window_title(&self) -> String {
        format!("{APP_NAME} — {}", self.document_title())
    }
}

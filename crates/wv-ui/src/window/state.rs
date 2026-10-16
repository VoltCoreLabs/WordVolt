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

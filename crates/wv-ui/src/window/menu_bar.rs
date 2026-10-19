use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_i18n::{Key, Lang, t as tr};

use crate::theme::{Theme, radius};
use crate::ui_kit::pop_in;

/// What a menu entry does (handled by `MainWindow`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    New,
    Open,
    Save,
    SaveAs,
    ExportPdf,
    Print,
    Undo,
    Redo,
    Find,
    Replace,
    About,
    Stats,
}

pub struct CommandChosen(pub Command);

/// (label, shortcut, command)
type Entry = (Key, &'static str, Command);

pub(super) const MOD: &str = if cfg!(target_os = "macos") {
    "⌘"
} else {
    "Ctrl+"
};

const FILE: &[Entry] = &[
    (Key::MenuNew, "N", Command::New),
    (Key::MenuOpen, "O", Command::Open),
    (Key::Save, "S", Command::Save),
    (Key::MenuSaveAs, "Shift+S", Command::SaveAs),
    (Key::TipExportPdf, "", Command::ExportPdf),
    (Key::MenuPrint, "P", Command::Print),
];
const EDIT: &[Entry] = &[
    (Key::MenuFind, "F", Command::Find),
    (Key::MenuReplace, "H", Command::Replace),
    (Key::MenuStats, "", Command::Stats),
];
const HISTORY: &[Entry] = &[
    (Key::MenuUndo, "Z", Command::Undo),
    (Key::MenuRedo, "Y", Command::Redo),
];

const HELP: &[Entry] = &[(Key::MenuAbout, "", Command::About)];

/// Title-bar menus: File, Edit and History. Rarely used commands live here, not in the ribbon.
pub struct MenuBar {
    lang: Lang,
    /// Index of the open menu.
    open: Option<usize>,
    /// The open menu was reached by hovering, so the next click on it keeps it open.
    by_hover: bool,
    /// Counts openings so the pop-in animation restarts each time.
    epoch: u64,
}

impl EventEmitter<CommandChosen> for MenuBar {}

impl MenuBar {
    pub fn new(lang: Lang) -> Self {
        Self {
            lang,
            open: None,
            by_hover: false,
            epoch: 0,
        }
    }

    fn show(&mut self, menu: Option<usize>, cx: &mut Context<Self>) {
        if menu.is_none() {
            self.by_hover = false;
        }
        if self.open != menu {
            self.open = menu;
            self.epoch += 1;
            cx.notify();
        }
    }

    fn panel(
        &self,
        index: usize,
        entries: &'static [Entry],
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let lang = self.lang;
        let th = cx.global::<Theme>();
        let (surface, border, hover, muted) =
            (th.surface, th.border, th.surface_alt, th.text_muted);
        let rows: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(row, (label, shortcut, command))| {
                let command = *command;
                let shortcut = if shortcut.is_empty() {
                    String::new()
                } else {
                    format!("{MOD}{shortcut}")
                };
                div()
                    .id(("menu-item", index * 10 + row))
                    .h(px(30.))
                    .px(px(10.))
                    .gap(px(24.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .rounded(radius::sm())
                    .text_sm()
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.show(None, cx);
                            cx.emit(CommandChosen(command));
                        }),
                    )
                    .child(tr(lang, *label))
                    .child(div().text_xs().text_color(muted).child(shortcut))
            })
            .collect();
        div()
            .id(("menu-panel", index))
            .occlude()
            .min_w(px(260.))
            .p(px(4.))
            .flex()
            .flex_col()
            .bg(surface)
            .border_1()
            .border_color(border)
            .rounded(radius::md())
            .shadow_lg()
            .children(rows)
    }
}

impl Render for MenuBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.lang;
        let th = cx.global::<Theme>();
        let (hover, open_bg) = (th.surface_alt, th.accent_subtle);
        let menus: [(Key, &'static [Entry]); 4] = [
            (Key::GroupFile, FILE),
            (Key::GroupEditing, EDIT),
            (Key::GroupHistory, HISTORY),
            (Key::MenuHelp, HELP),
        ];
        let items: Vec<_> = menus
            .into_iter()
            .enumerate()
            .map(|(index, (title, entries))| {
                let is_open = self.open == Some(index);
                let panel = is_open.then(|| {
                    deferred(
                        anchored()
                            .anchor(Anchor::TopLeft)
                            .offset(point(px(0.), px(30.)))
                            .child(pop_in(
                                self.panel(index, entries, cx),
                                format!("menu-pop-{index}-{}", self.epoch),
                            )),
                    )
                    .with_priority(10)
                });
                div()
                    .relative()
                    .child(
                        div()
                            .id(("menu-title", index))
                            .h(px(28.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .rounded(radius::sm())
                            .text_sm()
                            .cursor_pointer()
                            .when(is_open, |d| d.bg(open_bg))
                            .hover(move |s| s.bg(hover))
                            // With a menu open, moving over another title switches to it.
                            .on_hover(cx.listener(move |this, hovered, _, cx| {
                                if *hovered && this.open.is_some() && this.open != Some(index) {
                                    this.show(Some(index), cx);
                                    this.by_hover = true;
                                }
                            }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    let keep = std::mem::take(&mut this.by_hover);
                                    let next = (keep || this.open != Some(index)).then_some(index);
                                    this.show(next, cx);
                                }),
                            )
                            .child(tr(lang, title)),
                    )
                    .children(panel)
            })
            .collect();
        div()
            .flex()
            .items_center()
            .gap(px(2.))
            .on_mouse_down_out(cx.listener(|this, _, _, cx| this.show(None, cx)))
            .children(items)
    }
}

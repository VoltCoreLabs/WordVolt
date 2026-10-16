use gpui_kit::*;

use crate::ui_kit::{ButtonKind, Icon, block_group, button, icon_button};
use crate::window::state::{Align, MainWindow};

/// Home tab: font, paragraph, history, file.
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let font = block_group(
        "Font",
        vec![
            icon_button("bold", Icon::TextB, this.bold, cx)
                .on_click(cx.listener(|t, _, _, cx| {
                    t.bold = !t.bold;
                    cx.notify();
                }))
                .into_any_element(),
            icon_button("italic", Icon::TextItalic, this.italic, cx)
                .on_click(cx.listener(|t, _, _, cx| {
                    t.italic = !t.italic;
                    cx.notify();
                }))
                .into_any_element(),
            icon_button("underline", Icon::TextUnderline, this.underline, cx)
                .on_click(cx.listener(|t, _, _, cx| {
                    t.underline = !t.underline;
                    cx.notify();
                }))
                .into_any_element(),
            icon_button("highlight", Icon::Highlighter, false, cx).into_any_element(),
        ],
        cx,
    );
    let aligns = [
        ("al-left", Icon::AlignLeft, Align::Left),
        ("al-center", Icon::AlignCenter, Align::Center),
        ("al-right", Icon::AlignRight, Align::Right),
        ("al-justify", Icon::AlignJustify, Align::Justify),
    ];
    let mut para: Vec<AnyElement> = aligns
        .into_iter()
        .map(|(id, glyph, a)| {
            icon_button(id, glyph, this.align == a, cx)
                .on_click(cx.listener(move |t, _, _, cx| {
                    t.align = a;
                    cx.notify();
                }))
                .into_any_element()
        })
        .collect();
    para.push(icon_button("bullets", Icon::ListBullets, false, cx).into_any_element());
    para.push(icon_button("numbers", Icon::ListNumbers, false, cx).into_any_element());
    let paragraph = block_group("Paragraph", para, cx);
    let history = block_group(
        "History",
        vec![
            icon_button("undo", Icon::Undo, false, cx).into_any_element(),
            icon_button("redo", Icon::Redo, false, cx).into_any_element(),
        ],
        cx,
    );
    let file = block_group(
        "File",
        vec![
            icon_button("new", Icon::FilePlus, false, cx).into_any_element(),
            icon_button("open", Icon::FolderOpen, false, cx).into_any_element(),
            button("save", "Save", ButtonKind::Primary, cx).into_any_element(),
        ],
        cx,
    );
    vec![
        font.into_any_element(),
        paragraph.into_any_element(),
        history.into_any_element(),
        file.into_any_element(),
    ]
}

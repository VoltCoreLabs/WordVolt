use gpui_kit::*;
use wv_core::Align;
use wv_i18n::{Key, t};

use crate::ui_kit::{ButtonKind, Icon, block_group, button, icon_button};
use crate::window::state::MainWindow;

/// Home tab: font, paragraph, history, file. State comes from the document (caret/selection).
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let lang = this.lang;
    let (style, align) = {
        let view = this.editor.read(cx);
        (view.core.current_style(), view.core.align())
    };
    let size = style.size_pt.unwrap_or(wv_layout::DEFAULT_SIZE_PT);
    let font = block_group(
        t(lang, Key::GroupFont),
        vec![
            button("size-down", "A\u{2212}", ButtonKind::Default, cx)
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.bump_size(-1))))
                .into_any_element(),
            div()
                .w(px(28.))
                .flex()
                .justify_center()
                .child(size.to_string())
                .into_any_element(),
            button("size-up", "A+", ButtonKind::Default, cx)
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.bump_size(1))))
                .into_any_element(),
            icon_button("bold", Icon::TextB, style.bold, cx)
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.toggle_bold())))
                .into_any_element(),
            icon_button("italic", Icon::TextItalic, style.italic, cx)
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.toggle_italic())))
                .into_any_element(),
            icon_button("underline", Icon::TextUnderline, style.underline, cx)
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.toggle_underline())))
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
            icon_button(id, glyph, align == a, cx)
                .on_click(cx.listener(move |t, _, _, cx| t.edit(cx, |c| c.set_align(a))))
                .into_any_element()
        })
        .collect();
    para.push(icon_button("bullets", Icon::ListBullets, false, cx).into_any_element());
    para.push(icon_button("numbers", Icon::ListNumbers, false, cx).into_any_element());
    let paragraph = block_group(t(lang, Key::GroupParagraph), para, cx);
    let history = block_group(
        t(lang, Key::GroupHistory),
        vec![
            icon_button("undo", Icon::Undo, false, cx)
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| _ = c.undo())))
                .into_any_element(),
            icon_button("redo", Icon::Redo, false, cx)
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| _ = c.redo())))
                .into_any_element(),
        ],
        cx,
    );
    let file = block_group(
        t(lang, Key::GroupFile),
        vec![
            icon_button("new", Icon::FilePlus, false, cx)
                .on_click(cx.listener(|t, _, w, cx| t.new_document(w, cx)))
                .into_any_element(),
            icon_button("open", Icon::FolderOpen, false, cx)
                .on_click(cx.listener(|t, _, w, cx| t.open_document(w, cx)))
                .into_any_element(),
            button("save", t(lang, Key::Save), ButtonKind::Primary, cx)
                .on_click(cx.listener(|t, _, w, cx| t.save_document(w, cx)))
                .into_any_element(),
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

use gpui_kit::*;
use wv_core::{Align, Script};
use wv_i18n::{Key, t};

use crate::theme::DEFAULT_FONT;
use crate::ui_kit::{ButtonKind, Icon, block_group, button, icon_button};
use crate::window::state::MainWindow;

/// Home tab: font, text style, paragraph, history, file. State comes from the document (caret/selection).
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let lang = this.lang;
    let (style, align, quote) = {
        let view = this.editor.read(cx);
        (
            view.core.current_style(),
            view.core.align(),
            view.core.quote_active(),
        )
    };
    let size = style.size_pt.unwrap_or(wv_layout::DEFAULT_SIZE_PT);
    let font_name = style
        .font
        .clone()
        .unwrap_or_else(|| DEFAULT_FONT.to_string());
    this.font_combo.update(cx, |c, _| c.set_current(&font_name));
    this.size_combo
        .update(cx, |c, _| c.set_current(&size.to_string()));
    this.text_color
        .update(cx, |p, _| p.set_current(style.color));
    this.highlight
        .update(cx, |p, _| p.set_current(style.highlight));
    this.link_editor
        .update(cx, |p, _| p.set_current(style.link.clone()));

    let font = block_group(
        t(lang, Key::GroupFont),
        vec![
            this.font_combo.clone().into_any_element(),
            this.size_combo.clone().into_any_element(),
            button("size-down", "A\u{2212}", ButtonKind::Default, cx)
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.bump_size(-1))))
                .into_any_element(),
            button("size-up", "A+", ButtonKind::Default, cx)
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.bump_size(1))))
                .into_any_element(),
        ],
        cx,
    );

    let toggles = vec![
        ("bold", Icon::TextB, style.bold),
        ("italic", Icon::TextItalic, style.italic),
        ("underline", Icon::TextUnderline, style.underline),
        ("strike", Icon::Strikethrough, style.strike),
        (
            "superscript",
            Icon::Superscript,
            style.script == Script::Super,
        ),
        ("subscript", Icon::Subscript, style.script == Script::Sub),
        ("caps", Icon::TextAa, style.caps),
        ("mono", Icon::Code, style.mono),
        ("spoiler", Icon::EyeSlash, style.spoiler),
    ];
    let mut text_style: Vec<AnyElement> = toggles
        .into_iter()
        .map(|(id, glyph, active)| {
            icon_button(id, glyph, active, cx)
                .on_click(cx.listener(move |t, _, _, cx| {
                    t.edit(cx, |c| match id {
                        "bold" => c.toggle_bold(),
                        "italic" => c.toggle_italic(),
                        "underline" => c.toggle_underline(),
                        "strike" => c.toggle_strike(),
                        "superscript" => c.toggle_script(Script::Super),
                        "subscript" => c.toggle_script(Script::Sub),
                        "caps" => c.toggle_caps(),
                        "mono" => c.toggle_mono(),
                        _ => c.toggle_spoiler(),
                    });
                }))
                .into_any_element()
        })
        .collect();
    text_style.push(this.link_editor.clone().into_any_element());
    text_style.push(this.text_color.clone().into_any_element());
    text_style.push(this.highlight.clone().into_any_element());
    let text_style = block_group(t(lang, Key::GroupStyle), text_style, cx);

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
    para.push(
        icon_button("quote", Icon::Quotes, quote, cx)
            .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.toggle_quote())))
            .into_any_element(),
    );
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
        text_style.into_any_element(),
        paragraph.into_any_element(),
        history.into_any_element(),
        file.into_any_element(),
    ]
}

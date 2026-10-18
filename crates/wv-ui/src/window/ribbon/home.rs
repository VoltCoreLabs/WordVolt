use gpui_kit::*;
use wv_core::{Align, ListKind, Script};
use wv_i18n::{Key, t};

use crate::theme::DEFAULT_FONT;
use crate::ui_kit::{ButtonKind, Icon, Tip, block_group, button, icon_button};
use crate::window::state::MainWindow;
use crate::window::style_names::style_label;

/// Home tab: font, text style, paragraph, history, file. State comes from the document (caret/selection).
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let lang = this.lang;
    let (style, align, quote, list, named) = {
        let view = this.editor.read(cx);
        (
            view.core.current_style(),
            view.core.align(),
            view.core.quote_active(),
            view.core.list_kind(),
            view.core.named_style(),
        )
    };
    // The size shown includes what the paragraph style supplies (a heading is bigger than 12 pt).
    let mut shown = style.clone();
    named.apply(&mut shown);
    let size = shown.size_pt.unwrap_or(wv_layout::DEFAULT_SIZE_PT);
    this.style_combo
        .update(cx, |c, _| c.set_current(style_label(lang, named)));
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
    this.case_menu.update(cx, |m, _| m.set_active(style.caps));

    let font = block_group(
        t(lang, Key::GroupFont),
        vec![
            this.font_combo.clone().into_any_element(),
            this.size_combo.clone().into_any_element(),
            button("size-down", "A−", ButtonKind::Default, cx)
                .tip(t(lang, Key::TipSizeDown))
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.bump_size(-1))))
                .into_any_element(),
            button("size-up", "A+", ButtonKind::Default, cx)
                .tip(t(lang, Key::TipSizeUp))
                .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.bump_size(1))))
                .into_any_element(),
        ],
        cx,
    );

    let toggles = [
        ("bold", Icon::TextB, style.bold, Key::TipBold),
        ("italic", Icon::TextItalic, style.italic, Key::TipItalic),
        (
            "underline",
            Icon::TextUnderline,
            style.underline,
            Key::TipUnderline,
        ),
        ("strike", Icon::Strikethrough, style.strike, Key::TipStrike),
        (
            "superscript",
            Icon::Superscript,
            style.script == Script::Super,
            Key::TipSuper,
        ),
        (
            "subscript",
            Icon::Subscript,
            style.script == Script::Sub,
            Key::TipSub,
        ),
        ("mono", Icon::Code, style.mono, Key::TipCode),
        ("spoiler", Icon::EyeSlash, style.spoiler, Key::TipSpoiler),
    ];
    let mut text_style: Vec<AnyElement> = toggles
        .into_iter()
        .map(|(id, glyph, active, tip)| {
            icon_button(id, glyph, active, cx)
                .tip(t(lang, tip))
                .on_click(cx.listener(move |t, _, _, cx| {
                    t.edit(cx, |c| match id {
                        "bold" => c.toggle_bold(),
                        "italic" => c.toggle_italic(),
                        "underline" => c.toggle_underline(),
                        "strike" => c.toggle_strike(),
                        "superscript" => c.toggle_script(Script::Super),
                        "subscript" => c.toggle_script(Script::Sub),
                        "mono" => c.toggle_mono(),
                        _ => c.toggle_spoiler(),
                    });
                }))
                .into_any_element()
        })
        .collect();
    text_style.insert(6, this.case_menu.clone().into_any_element());
    text_style.push(this.link_editor.clone().into_any_element());
    text_style.push(this.text_color.clone().into_any_element());
    text_style.push(this.highlight.clone().into_any_element());
    let text_style = block_group(t(lang, Key::GroupStyle), text_style, cx);

    let aligns = [
        ("al-left", Icon::AlignLeft, Align::Left, Key::TipAlignLeft),
        (
            "al-center",
            Icon::AlignCenter,
            Align::Center,
            Key::TipAlignCenter,
        ),
        (
            "al-right",
            Icon::AlignRight,
            Align::Right,
            Key::TipAlignRight,
        ),
        (
            "al-justify",
            Icon::AlignJustify,
            Align::Justify,
            Key::TipAlignJustify,
        ),
    ];
    let mut para: Vec<AnyElement> = aligns
        .into_iter()
        .map(|(id, glyph, a, tip)| {
            icon_button(id, glyph, align == a, cx)
                .tip(t(lang, tip))
                .on_click(cx.listener(move |t, _, _, cx| t.edit(cx, |c| c.set_align(a))))
                .into_any_element()
        })
        .collect();
    para.push(
        icon_button("quote", Icon::Quotes, quote, cx)
            .tip(t(lang, Key::TipQuote))
            .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.toggle_quote())))
            .into_any_element(),
    );
    for (id, glyph, kind, tip) in [
        (
            "bullets",
            Icon::ListBullets,
            ListKind::Bullet,
            Key::TipBullets,
        ),
        (
            "numbers",
            Icon::ListNumbers,
            ListKind::Number,
            Key::TipNumbers,
        ),
    ] {
        para.push(
            icon_button(id, glyph, list == Some(kind), cx)
                .tip(t(lang, tip))
                .on_click(cx.listener(move |t, _, _, cx| t.edit(cx, |c| c.toggle_list(kind))))
                .into_any_element(),
        );
    }
    para.push(
        icon_button("outdent", Icon::Outdent, false, cx)
            .tip(t(lang, Key::TipOutdent))
            .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.change_indent(-1))))
            .into_any_element(),
    );
    para.push(
        icon_button("indent", Icon::Indent, false, cx)
            .tip(t(lang, Key::TipIndent))
            .on_click(cx.listener(|t, _, _, cx| t.edit(cx, |c| c.change_indent(1))))
            .into_any_element(),
    );
    para.push(this.spacing_menu.clone().into_any_element());
    let paragraph = block_group(t(lang, Key::GroupParagraph), para, cx);

    let styles = block_group(
        t(lang, Key::GroupStyles),
        vec![
            this.style_combo.clone().into_any_element(),
            icon_button("styles-panel", Icon::Rows, this.styles_open, cx)
                .tip(t(lang, Key::TipStylesPanel))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.styles_open = !this.styles_open;
                    cx.notify();
                }))
                .into_any_element(),
        ],
        cx,
    );

    vec![
        font.into_any_element(),
        text_style.into_any_element(),
        paragraph.into_any_element(),
        styles.into_any_element(),
    ]
}

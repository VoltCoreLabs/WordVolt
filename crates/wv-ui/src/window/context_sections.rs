use gpui_kit::*;
use wv_core::{Align, ListKind};
use wv_i18n::{Key, t};

use super::context_menu::tool;
use super::context_parts::choice_row;
use super::state::MainWindow;
use crate::theme::Theme;
use crate::ui_kit::Icon;
use wv_core::CharStyle;
use wv_i18n::Lang;

/// Corrections for a misspelled word under the caret (and "add to dictionary" / "skip").
pub(super) fn spelling_rows(
    this: &MainWindow,
    lang: Lang,
    cx: &mut Context<MainWindow>,
) -> Vec<AnyElement> {
    let spelling = crate::spell::active(cx).and_then(|speller| {
        let core = &this.editor.read(cx).core;
        let caret = core.sel.head;
        let text = core.doc.paragraphs[caret.para].text();
        let range = speller
            .misspelled(&text)
            .iter()
            .find(|r| r.start <= caret.offset && caret.offset <= r.end)
            .cloned()?;
        let word = text[range.clone()].to_string();
        Some((caret.para, range, speller.suggest(&word, 5), word))
    });
    let mut spell_rows: Vec<AnyElement> = Vec::new();
    if let Some((para, range, suggestions, word)) = spelling {
        let (accent, border) = {
            let th = cx.global::<Theme>();
            (th.accent, th.border)
        };
        for (i, suggestion) in suggestions.into_iter().enumerate() {
            let (start, end) = (range.start, range.end);
            let replacement = suggestion.clone();
            spell_rows.push(
                choice_row(
                    ("ctx-fix", i),
                    suggestion,
                    Some(accent),
                    cx,
                    move |this, _, cx| {
                        let replacement = replacement.clone();
                        this.edit(cx, move |c| {
                            c.replace_match(
                                wv_core::Position::new(para, start),
                                wv_core::Position::new(para, end),
                                &replacement,
                            )
                        });
                    },
                )
                .into_any_element(),
            );
        }
        for (i, (label, keep)) in [
            (t(lang, Key::CtxAddWord), true),
            (t(lang, Key::CtxSkipWord), false),
        ]
        .into_iter()
        .enumerate()
        {
            let word = word.clone();
            spell_rows.push(
                choice_row(
                    ("ctx-dict", i),
                    label.to_string(),
                    None,
                    cx,
                    move |this, _, cx| this.add_to_dictionary(&word, keep, cx),
                )
                .into_any_element(),
            );
        }
        spell_rows.push(div().h(px(1.)).my(px(4.)).bg(border).into_any_element());
    }
    spell_rows
}

/// Bold / italic / underline / ... toolbar.
pub(super) fn format_tools(style: &CharStyle, lang: Lang, cx: &mut Context<MainWindow>) -> Div {
    div()
        .flex()
        .gap(px(4.))
        .child(tool(
            "ctx-bold",
            Icon::TextB,
            style.bold,
            Key::TipBold,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_bold()),
            cx,
        ))
        .child(tool(
            "ctx-italic",
            Icon::TextItalic,
            style.italic,
            Key::TipItalic,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_italic()),
            cx,
        ))
        .child(tool(
            "ctx-underline",
            Icon::TextUnderline,
            style.underline,
            Key::TipUnderline,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_underline()),
            cx,
        ))
        .child(tool(
            "ctx-strike",
            Icon::Strikethrough,
            style.strike,
            Key::TipStrike,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_strike()),
            cx,
        ))
        .child(tool(
            "ctx-code",
            Icon::Code,
            style.mono,
            Key::TipCode,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_mono()),
            cx,
        ))
        .child(tool(
            "ctx-spoiler",
            Icon::EyeSlash,
            style.spoiler,
            Key::TipSpoiler,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_spoiler()),
            cx,
        ))
}

/// Alignment, list and quote toolbar.
pub(super) fn paragraph_tools(
    align: Align,
    list: Option<ListKind>,
    quote: bool,
    lang: Lang,
    cx: &mut Context<MainWindow>,
) -> Div {
    div()
        .flex()
        .gap(px(4.))
        .child(tool(
            "ctx-left",
            Icon::AlignLeft,
            align == Align::Left,
            Key::TipAlignLeft,
            lang,
            |t, _, cx| t.edit(cx, |c| c.set_align(Align::Left)),
            cx,
        ))
        .child(tool(
            "ctx-center",
            Icon::AlignCenter,
            align == Align::Center,
            Key::TipAlignCenter,
            lang,
            |t, _, cx| t.edit(cx, |c| c.set_align(Align::Center)),
            cx,
        ))
        .child(tool(
            "ctx-right",
            Icon::AlignRight,
            align == Align::Right,
            Key::TipAlignRight,
            lang,
            |t, _, cx| t.edit(cx, |c| c.set_align(Align::Right)),
            cx,
        ))
        .child(tool(
            "ctx-justify",
            Icon::AlignJustify,
            align == Align::Justify,
            Key::TipAlignJustify,
            lang,
            |t, _, cx| t.edit(cx, |c| c.set_align(Align::Justify)),
            cx,
        ))
        .child(tool(
            "ctx-bullets",
            Icon::ListBullets,
            list == Some(ListKind::Bullet),
            Key::TipBullets,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_list(ListKind::Bullet)),
            cx,
        ))
        .child(tool(
            "ctx-numbers",
            Icon::ListNumbers,
            list == Some(ListKind::Number),
            Key::TipNumbers,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_list(ListKind::Number)),
            cx,
        ))
        .child(tool(
            "ctx-quote",
            Icon::Quotes,
            quote,
            Key::TipQuote,
            lang,
            |t, _, cx| t.edit(cx, |c| c.toggle_quote()),
            cx,
        ))
}

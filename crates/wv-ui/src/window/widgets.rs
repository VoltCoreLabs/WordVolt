use super::color_picker::{ColorEvent, ColorPicker};
use super::combo::{Combo, ComboConfig, ComboEvent, ItemLook};
use super::comment_editor::{CommentEditor, CommentEvent};
use super::link_editor::{LinkEditor, LinkEvent};
use super::menu_button::{MenuButton, MenuEvent, MenuItem};
use super::refs_panel::{RefEvent, RefsPanel};
use super::style_namer::{NewStyle, StyleNamer};
use super::style_names::{style_from_label, style_label};
use crate::theme::DEFAULT_FONT;
use crate::ui_kit::Icon;
use gpui_kit::*;
use wv_core::{CaseKind, NamedStyle};
use wv_i18n::{Key, Lang};

use super::state::MainWindow;

/// Drop-down preview of a paragraph style: roughly its size (capped to keep rows compact), weight and colour.
fn style_look(label: &str) -> ItemLook {
    let Some(style) = style_from_label(Lang::default(), label) else {
        return ItemLook::default();
    };
    ItemLook {
        size_px: style
            .size_pt()
            .map(|pt| (f32::from(pt) * 96.0 / 72.0).min(24.0)),
        bold: style.bold(),
        color: style.color(),
        ..ItemLook::default()
    }
}

/// Line spacing choices: label and multiplier.
const SPACINGS: [(&str, f32); 6] = [
    ("1.0", 1.0),
    ("1.15", 1.15),
    ("1.5", 1.5),
    ("2.0", 2.0),
    ("2.5", 2.5),
    ("3.0", 3.0),
];

pub(super) fn make_font_combo(_lang: Lang, cx: &mut Context<MainWindow>) -> Entity<Combo> {
    let font_combo = cx.new(|cx| {
        let config = ComboConfig {
            id: "font",
            width: 168.0,
            default_label: Some(format!("{DEFAULT_FONT} ·")),
            look: Some(|name| ItemLook {
                family: Some(name.to_string()),
                ..ItemLook::default()
            }),
            items: |cx| {
                let mut fonts: Vec<String> = cx
                    .text_system()
                    .all_font_names()
                    .into_iter()
                    .filter(|n| !n.starts_with('.'))
                    .collect();
                fonts.sort_by_key(|n| n.to_lowercase());
                fonts.dedup();
                fonts
            },
        };
        Combo::new(config, DEFAULT_FONT.to_string(), cx)
    });
    cx.subscribe(
        &font_combo,
        |this, _, ComboEvent::Picked(name): &ComboEvent, cx| {
            let name = name.clone();
            this.edit(cx, move |c| c.set_font_family(name));
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    font_combo
}

pub(super) fn make_size_combo(_lang: Lang, cx: &mut Context<MainWindow>) -> Entity<Combo> {
    let size_combo = cx.new(|cx| {
        let config = ComboConfig {
            id: "size",
            width: 64.0,
            default_label: None,
            look: None,
            items: |_| {
                [8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72]
                    .iter()
                    .map(u16::to_string)
                    .collect()
            },
        };
        Combo::new(config, "12".to_string(), cx)
    });
    cx.subscribe(
        &size_combo,
        |this, _, ComboEvent::Picked(text): &ComboEvent, cx| {
            if let Some(size) = text
                .as_deref()
                .and_then(|t| t.parse::<u16>().ok())
                .filter(|s| (1..=999).contains(s))
            {
                this.edit(cx, move |c| c.set_size_pt(size));
            }
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    size_combo
}

pub(super) fn make_text_color(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<ColorPicker> {
    let text_color =
        cx.new(|cx| ColorPicker::new("text-color", Icon::TextColor, Key::TipTextColor, lang, cx));
    cx.subscribe(
        &text_color,
        |this, _, ColorEvent::Picked(color): &ColorEvent, cx| {
            let color = *color;
            this.edit(cx, move |c| c.set_color(color));
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    text_color
}

pub(super) fn make_highlight(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<ColorPicker> {
    let highlight =
        cx.new(|cx| ColorPicker::new("highlight", Icon::Highlighter, Key::TipHighlight, lang, cx));
    cx.subscribe(
        &highlight,
        |this, _, ColorEvent::Picked(color): &ColorEvent, cx| {
            let color = *color;
            this.edit(cx, move |c| c.set_highlight(color));
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    highlight
}

pub(super) fn make_link_editor(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<LinkEditor> {
    let link_editor = cx.new(|cx| LinkEditor::new(lang, cx));
    cx.subscribe(
        &link_editor,
        |this, _, LinkEvent::Set(url): &LinkEvent, cx| {
            let url = url.clone();
            this.edit(cx, move |c| c.set_link(url));
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    link_editor
}

pub(super) fn make_comment_editor(
    lang: Lang,
    cx: &mut Context<MainWindow>,
) -> Entity<CommentEditor> {
    let editor = cx.new(|cx| CommentEditor::new(lang, cx));
    cx.subscribe(&editor, |this, _, event: &CommentEvent, cx| {
        let author = std::env::var("USERNAME").unwrap_or_default();
        let event = event.clone();
        this.edit(cx, move |c| match event {
            CommentEvent::Add(text) => {
                c.add_comment(&author, &text);
            }
            CommentEvent::Edit(id, text) => c.set_comment_text(id, &text),
            CommentEvent::Delete(id) => c.delete_comment(id),
        });
        this.editor.update(cx, |view, cx| view.request_focus(cx));
    })
    .detach();
    editor
}

pub(super) fn make_style_namer(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<StyleNamer> {
    let namer = cx.new(|cx| StyleNamer::new(lang, cx));
    cx.subscribe(&namer, |this, _, NewStyle(name): &NewStyle, cx| {
        let name = name.clone();
        this.edit(cx, move |c| _ = c.define_style(&name));
        this.editor.update(cx, |view, cx| view.request_focus(cx));
    })
    .detach();
    namer
}

pub(super) fn make_refs_panel(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<RefsPanel> {
    let panel = cx.new(|cx| RefsPanel::new(lang, cx));
    cx.subscribe(&panel, |this, _, event: &RefEvent, cx| {
        let event = event.clone();
        this.edit(cx, move |c| match event {
            RefEvent::Add(name) => _ = c.add_bookmark(&name),
            RefEvent::Delete(id) => c.delete_bookmark(id),
            RefEvent::Bookmark(name) => {
                let text = c.bookmark_text(&name).unwrap_or_else(|| name.clone());
                c.insert_reference(&name, text.trim());
            }
            RefEvent::Heading(para) => c.insert_heading_reference(para),
        });
        this.editor.update(cx, |view, cx| view.request_focus(cx));
    })
    .detach();
    panel
}

pub(super) fn make_case_menu(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<MenuButton> {
    let case_menu = cx.new(|cx| {
        let items = vec![
            MenuItem::Tr(Key::CaseUpper),
            MenuItem::Tr(Key::CaseLower),
            MenuItem::Tr(Key::CaseTitle),
            MenuItem::Tr(Key::CaseSentence),
            MenuItem::Tr(Key::CaseDisplay),
        ];
        MenuButton::new("case", Icon::TextAa, Key::TipCase, items, lang, cx)
    });
    cx.subscribe(
        &case_menu,
        |this, _, MenuEvent::Picked(index): &MenuEvent, cx| {
            let index = *index;
            this.edit(cx, move |c| match index {
                0 => c.change_case(CaseKind::Upper),
                1 => c.change_case(CaseKind::Lower),
                2 => c.change_case(CaseKind::Title),
                3 => c.change_case(CaseKind::Sentence),
                _ => c.toggle_caps(),
            });
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    case_menu
}

pub(super) fn make_spacing_menu(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<MenuButton> {
    let spacing_menu = cx.new(|cx| {
        let items = SPACINGS
            .iter()
            .map(|(label, _)| MenuItem::Text(label))
            .chain([
                MenuItem::Tr(Key::GapBefore),
                MenuItem::Tr(Key::GapAfter),
                MenuItem::Tr(Key::FirstLineMore),
                MenuItem::Tr(Key::FirstLineLess),
            ])
            .collect();
        MenuButton::new(
            "spacing",
            Icon::LineSpacing,
            Key::TipLineSpacing,
            items,
            lang,
            cx,
        )
    });
    cx.subscribe(
        &spacing_menu,
        |this, _, MenuEvent::Picked(index): &MenuEvent, cx| {
            if let Some((_, value)) = SPACINGS.get(*index) {
                let value = *value;
                this.edit(cx, move |c| c.set_line_spacing(Some(value)));
            } else {
                let extra = *index - SPACINGS.len();
                this.edit(cx, move |c| match extra {
                    0 | 1 => {
                        let after = extra == 1;
                        let on = c.para_gap(after).is_none_or(|n| n == 0);
                        c.set_para_gap(after, Some(if on { 12 } else { 0 }));
                    }
                    2 => c.change_first_line(18),
                    _ => c.change_first_line(-18),
                });
            }
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    spacing_menu
}

pub(super) fn make_style_combo(lang: Lang, cx: &mut Context<MainWindow>) -> Entity<Combo> {
    let style_combo = cx.new(|cx| {
        let config = ComboConfig {
            id: "style",
            width: 210.0,
            default_label: None,
            look: Some(style_look),
            items: |_| {
                NamedStyle::ALL
                    .iter()
                    .map(|s| style_label(Lang::default(), *s).to_string())
                    .collect()
            },
        };
        Combo::new(
            config,
            style_label(lang, NamedStyle::Normal).to_string(),
            cx,
        )
    });
    cx.subscribe(
        &style_combo,
        move |this, _, ComboEvent::Picked(label): &ComboEvent, cx| {
            if let Some(style) = label.as_deref().and_then(|l| style_from_label(lang, l)) {
                this.edit(cx, move |c| c.set_named_style(style));
            }
            this.editor.update(cx, |view, cx| view.request_focus(cx));
        },
    )
    .detach();
    style_combo
}

use gpui_kit::*;
use wv_core::TextStats;
use wv_i18n::{Key, t};

use super::state::MainWindow;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{ButtonKind, block, button};

/// The "Statistics" modal: counts of the whole document and, when text is selected, of the
/// selection. A click on the backdrop closes it.
pub fn stats_dialog(
    this: &MainWindow,
    pages: usize,
    cx: &mut Context<MainWindow>,
) -> Option<AnyElement> {
    if !this.stats_open {
        return None;
    }
    let lang = this.lang;
    let muted = cx.global::<Theme>().text_muted;
    let (all, selected) = {
        let core = &this.editor.read(cx).core;
        let (s, e) = core.sel.range();
        (core.doc.stats(), (s != e).then(|| core.doc.stats_in(s, e)))
    };
    let rows = |stats: TextStats, pages: Option<usize>| {
        let line = |label: Key, value: usize| {
            div()
                .flex()
                .gap(space::md())
                .child(
                    div()
                        .w(px(180.))
                        .flex_none()
                        .text_color(muted)
                        .child(t(lang, label)),
                )
                .child(div().flex_1().child(value.to_string()))
        };
        div()
            .flex()
            .flex_col()
            .gap(space::sm())
            .text_sm()
            .children(pages.map(|n| line(Key::StatsPages, n)))
            .child(line(Key::StatsWords, stats.words))
            .child(line(Key::StatsChars, stats.chars))
            .child(line(Key::StatsCharsNoSpaces, stats.chars_no_spaces))
            .child(line(Key::StatsParagraphs, stats.paragraphs))
    };
    let heading = |key: Key| div().font_weight(FontWeight::SEMIBOLD).child(t(lang, key));
    let card = block(cx)
        .id("stats-card")
        .occlude()
        .w(px(400.))
        .p(space::lg())
        .flex()
        .flex_col()
        .gap(space::md())
        .rounded(radius::md())
        .shadow_lg()
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::SEMIBOLD)
                .child(t(lang, Key::MenuStats)),
        )
        .child(rows(all, Some(pages)))
        .children(selected.map(|s| {
            div()
                .flex()
                .flex_col()
                .gap(space::sm())
                .child(heading(Key::StatsSelection))
                .child(rows(s, None))
        }))
        .child(
            div().flex().justify_end().child(
                button(
                    "stats-close",
                    t(lang, Key::AboutClose),
                    ButtonKind::Primary,
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.stats_open = false;
                    cx.notify();
                })),
            ),
        );
    Some(
        div()
            .id("stats-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::hsla(0., 0., 0., 0.35))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.stats_open = false;
                    cx.notify();
                }),
            )
            .child(card)
            .into_any_element(),
    )
}

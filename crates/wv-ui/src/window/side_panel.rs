use std::time::Duration;

use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use wv_core::Position;
use wv_i18n::{Key, t};

use super::state::MainWindow;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::block;

const WIDTH: f32 = 280.;
/// More results than this are counted but not listed.
const MAX_ROWS: usize = 300;

/// One line of the list: where it leads, its page and the text around it.
struct Row {
    page: usize,
    indent: u8,
    before: String,
    hit: String,
    after: String,
    current: bool,
}

/// The text of the line around a match: a little before it, the match itself, a little after.
fn snippet(text: &str, start: usize, end: usize) -> (String, String, String) {
    let (start, end) = (
        start.min(text.len()),
        end.min(text.len()).max(start.min(text.len())),
    );
    let before: Vec<char> = text[..start].chars().rev().take(28).collect();
    let before: String = before.into_iter().rev().collect();
    let hit: String = text[start..end].chars().take(60).collect();
    let after: String = text[end..].chars().take(50).collect();
    (before.trim_start().to_string(), hit, after)
}

/// Right-hand navigation pane (like Word's): the matches of the search with their text and page,
/// or, with no search going on, the headings of the document. Slides in/out with an ease-out
/// animation; it opens by itself while the search bar is open.
pub fn side_panel(
    this: &MainWindow,
    window: &Window,
    cx: &mut Context<MainWindow>,
) -> impl IntoElement {
    let find_open = this.find_bar.read(cx).is_open();
    let open = this.panel_open || find_open;
    let lang = this.lang;
    let th = cx.global::<Theme>();
    let (muted, hover, subtle, accent) =
        (th.text_muted, th.surface_alt, th.accent_subtle, th.accent);

    // Rows: search results when there is a query, headings otherwise.
    let (query, matches, current) = {
        let bar = this.find_bar.read(cx);
        (
            bar.query().to_string(),
            bar.results().to_vec(),
            bar.current(),
        )
    };
    let searching = find_open && !query.is_empty();
    let total = if searching { matches.len() } else { 0 };
    let (title, rows): (Key, Vec<(Row, usize)>) = if !open {
        (Key::NavTitle, Vec::new())
    } else if searching {
        let shown: Vec<_> = matches.iter().take(MAX_ROWS).collect();
        let positions: Vec<Position> = shown.iter().map(|(s, _)| *s).collect();
        let pages = this
            .editor
            .update(cx, |view, _| view.pages_of(&positions, window));
        let doc = &this.editor.read(cx).core.doc;
        let rows = shown
            .iter()
            .enumerate()
            .map(|(i, (start, end))| {
                let text = doc.paragraphs[start.para].text();
                let end_offset = if end.para == start.para {
                    end.offset
                } else {
                    text.len()
                };
                let (before, hit, after) = snippet(&text, start.offset, end_offset);
                (
                    Row {
                        page: pages[i],
                        indent: 0,
                        before,
                        hit,
                        after,
                        current: current == Some(i),
                    },
                    i,
                )
            })
            .collect();
        (Key::NavResults, rows)
    } else {
        let headings = this.editor.read(cx).core.headings();
        let positions: Vec<Position> = headings.iter().map(|h| Position::new(h.para, 0)).collect();
        let pages = this
            .editor
            .update(cx, |view, _| view.pages_of(&positions, window));
        let rows = headings
            .iter()
            .enumerate()
            .map(|(i, h)| {
                (
                    Row {
                        page: pages[i],
                        indent: h.level.saturating_sub(1),
                        before: h.text.clone(),
                        hit: String::new(),
                        after: String::new(),
                        current: false,
                    },
                    h.para,
                )
            })
            .collect();
        (Key::NavHeadings, rows)
    };

    let empty = rows.is_empty();
    let list: Vec<_> = rows
        .into_iter()
        .enumerate()
        .map(|(n, (row, target))| {
            let page = format!("{} {}", t(lang, Key::NavPageShort), row.page);
            div()
                .id(("nav-row", n))
                .w_full()
                .px(space::sm())
                .py(px(5.))
                .pl(space::sm() + px(f32::from(row.indent) * 14.0))
                .flex()
                .flex_col()
                .gap(px(1.))
                .rounded(radius::sm())
                .cursor_pointer()
                .text_sm()
                .when(row.current, |d| d.bg(subtle))
                .hover(move |s| s.bg(hover))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        if searching {
                            this.find_bar.update(cx, |bar, cx| bar.go_to(target, cx));
                        } else {
                            this.edit(cx, move |c| c.set_head(Position::new(target, 0), false));
                        }
                    }),
                )
                .child(div().text_xs().text_color(muted).child(page))
                .child(
                    div()
                        .flex()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(row.before)
                        .when(!row.hit.is_empty(), |d| {
                            d.child(
                                div()
                                    .flex_none()
                                    .px(px(1.))
                                    .rounded(px(2.))
                                    .bg(subtle)
                                    .text_color(accent)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(row.hit),
                            )
                            .child(row.after)
                        }),
                )
        })
        .collect();

    let heading = if searching {
        format!("{} · {total}", t(lang, title))
    } else {
        t(lang, title).to_string()
    };
    let hint = if searching {
        Key::NavNoResults
    } else {
        Key::NavNoHeadings
    };
    let content = block(cx)
        .w(px(WIDTH))
        .h_full()
        .flex_none()
        .p(space::md())
        .flex()
        .flex_col()
        .gap(space::sm())
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child(t(lang, Key::NavTitle)),
        )
        .child(div().text_xs().text_color(muted).child(heading))
        .child(
            div()
                .id("nav-list")
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .children(list)
                .when(empty, |d| {
                    d.child(div().text_sm().text_color(muted).child(t(lang, hint)))
                }),
        );
    div()
        .h_full()
        .flex_none()
        .overflow_hidden()
        .child(content)
        .with_animation(
            SharedString::from(format!("side-panel-{open}")),
            Animation::new(Duration::from_millis(260)).with_easing(ease_out_quint()),
            move |el, delta| {
                let k = if open { delta } else { 1.0 - delta };
                el.w(px(WIDTH * k))
            },
        )
}

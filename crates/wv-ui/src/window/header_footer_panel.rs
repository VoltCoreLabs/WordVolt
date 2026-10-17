use gpui_kit::*;
use wv_core::{Align, BAND_SIZE_PT, CharStyle};
use wv_i18n::{Key, t as tr};

use super::band_controls::check_box;
use super::header_footer::HeaderFooterEditor;
use crate::theme::{Theme, radius, space};
use crate::ui_kit::{ButtonKind, Icon, LineEdit, Tip, button, icon_button, track_bounds};

const FIELD_WIDTH: f32 = 280.0;

/// Text colours offered in the popover (document content, not theme colours).
const SWATCHES: [u32; 7] = [
    0x000000, 0x666666, 0x2563EB, 0xDC2626, 0x16A34A, 0xEA580C, 0x9333EA,
];

impl HeaderFooterEditor {
    pub(super) fn panel(&mut self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = cx.global::<Theme>();
        let (surface, border, accent, muted) = (th.surface, th.border, th.accent, th.text_muted);
        let lang = self.lang;
        let focus_header = self.header_focus.is_focused(window);
        let focus_footer = self.footer_focus.is_focused(window);
        let field =
            |id: &'static str,
             edit: &LineEdit,
             focus: &FocusHandle,
             focused: bool,
             in_header: bool,
             cx: &mut Context<Self>| {
                div()
                    .id(id)
                    .track_focus(focus)
                    .key_context("TabField")
                    .on_action(cx.listener(
                        move |this, _: &crate::actions::FieldNext, window, cx| {
                            this.switch_field(in_header, window, cx)
                        },
                    ))
                    .on_key_down(cx.listener(move |this, ev: &KeyDownEvent, _, cx| {
                        this.on_key(in_header, ev, cx)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            window.focus(
                                if in_header {
                                    &this.header_focus
                                } else {
                                    &this.footer_focus
                                },
                                cx,
                            )
                        }),
                    )
                    .w(px(FIELD_WIDTH))
                    .h(px(32.))
                    .px(space::md())
                    .flex()
                    .items_center()
                    .rounded(radius::md())
                    .border_1()
                    .border_color(if focused { accent } else { border })
                    .text_sm()
                    .cursor_text()
                    .child(edit.view(FIELD_WIDTH - 24.0, window, cx))
            };
        let work = self.work.clone();
        let format_row = |in_header: bool, cx: &mut Context<Self>| {
            type Flag = (
                &'static str,
                Icon,
                Key,
                fn(&CharStyle) -> bool,
                fn(&mut CharStyle, bool),
            );
            let flags: [Flag; 3] = [
                (
                    "b",
                    Icon::TextB,
                    Key::TipBold,
                    |s| s.bold,
                    |s, v| s.bold = v,
                ),
                (
                    "i",
                    Icon::TextItalic,
                    Key::TipItalic,
                    |s| s.italic,
                    |s, v| s.italic = v,
                ),
                (
                    "u",
                    Icon::TextUnderline,
                    Key::TipUnderline,
                    |s| s.underline,
                    |s, v| s.underline = v,
                ),
            ];
            let toggles = flags.map(|(id, icon, tip, get, set)| {
                icon_button(
                    ElementId::from(SharedString::from(format!("band-fmt-{in_header}-{id}"))),
                    icon,
                    self.all_chars(in_header, get),
                    cx,
                )
                .tip(tr(lang, tip))
                .on_click(
                    cx.listener(move |this, _, _, cx| this.toggle_style(in_header, get, set, cx)),
                )
                .into_any_element()
            });
            let sizes = [(-1, "A−", Key::TipSizeDown), (1, "A+", Key::TipSizeUp)].map(
                |(delta, text, tip)| {
                    button(
                        SharedString::from(format!("band-size-{in_header}-{delta}")),
                        text,
                        ButtonKind::Default,
                        cx,
                    )
                    .tip(tr(lang, tip))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.restyle(
                            in_header,
                            move |s| {
                                let size = i32::from(s.size_pt.unwrap_or(BAND_SIZE_PT)) + delta;
                                s.size_pt = Some(size.clamp(6, 72) as u16);
                            },
                            cx,
                        )
                    }))
                    .into_any_element()
                },
            );
            let swatches = std::iter::once(None)
                .chain(SWATCHES.map(Some))
                .enumerate()
                .map(|(i, color)| {
                    let fill = color.map_or(surface, rgb);
                    div()
                        .id(SharedString::from(format!("band-color-{in_header}-{i}")))
                        .size(px(18.))
                        .flex_none()
                        .rounded(radius::sm())
                        .border_1()
                        .border_color(border)
                        .bg(fill)
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_xs()
                        .text_color(muted)
                        .child(if color.is_none() { "×" } else { "" })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.restyle(in_header, move |s| s.color = color, cx)
                        }))
                        .into_any_element()
                });
            let size = if in_header { &work.0 } else { &work.1 }
                .style_at(0)
                .size_pt
                .unwrap_or(BAND_SIZE_PT);
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(space::xs())
                        .children(toggles)
                        .child(
                            div()
                                .px(px(6.))
                                .text_xs()
                                .text_color(muted)
                                .child(format!("{size} pt")),
                        )
                        .children(sizes),
                )
                .child(div().flex().items_center().gap(px(4.)).children(swatches))
        };
        let aligns = (self.work.0.align, self.work.1.align);
        let label = |key: Key, in_header: bool, cx: &mut Context<Self>| {
            let current = if in_header { aligns.0 } else { aligns.1 };
            let buttons = [
                ("l", Icon::AlignLeft, Align::Left),
                ("c", Icon::AlignCenter, Align::Center),
                ("r", Icon::AlignRight, Align::Right),
            ]
            .map(|(id, icon, align)| {
                icon_button(
                    ElementId::from(SharedString::from(format!("band-align-{in_header}-{id}"))),
                    icon,
                    current == align,
                    cx,
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    if in_header {
                        this.work.0.align = align;
                    } else {
                        this.work.1.align = align;
                    }
                    cx.notify();
                }))
            });
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(div().text_xs().text_color(muted).child(tr(lang, key)))
                .child(div().flex().gap(space::xs()).children(buttons))
        };
        // Which header/footer is edited: only offered when a variant is switched on.
        let mut tabs: Vec<(usize, Key)> = vec![(0, Key::BandMain)];
        if self.flags.0 {
            tabs.push((1, Key::BandFirst));
        }
        if self.flags.1 {
            tabs.push((2, Key::BandEven));
        }
        let selector = (tabs.len() > 1).then(|| {
            div()
                .flex()
                .gap(space::xs())
                .children(tabs.into_iter().map(|(i, key)| {
                    let kind = if self.which == i {
                        ButtonKind::Primary
                    } else {
                        ButtonKind::Default
                    };
                    button(("band-set", i), tr(lang, key), kind, cx).on_click(
                        cx.listener(move |this, _, window, cx| this.select(i, window, cx)),
                    )
                }))
        });
        let header = field(
            "band-header",
            &self.header,
            &self.header_focus,
            focus_header,
            true,
            cx,
        );
        let footer = field(
            "band-footer",
            &self.footer,
            &self.footer_focus,
            focus_footer,
            false,
            cx,
        );
        div()
            .id("band-panel")
            .occlude()
            .relative()
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(6.))
            .bg(surface)
            .border_1()
            .border_color(border)
            .rounded(radius::md())
            .shadow_lg()
            .child(track_bounds(self.panel_bounds.clone()))
            .children(selector)
            .child(label(Key::HeaderLabel, true, cx))
            .child(format_row(true, cx))
            .child(header)
            .child(label(Key::FooterLabel, false, cx))
            .child(format_row(false, cx))
            .child(footer)
            .child(
                check_box("band-first", tr(lang, Key::BandDiffFirst), self.flags.0, cx).on_click(
                    cx.listener(|this, _, window, cx| this.toggle_flag(true, window, cx)),
                ),
            )
            .child(
                check_box("band-evenodd", tr(lang, Key::BandOddEven), self.flags.1, cx).on_click(
                    cx.listener(|this, _, window, cx| this.toggle_flag(false, window, cx)),
                ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(tr(lang, Key::BandHint)),
            )
            .child(
                button("band-apply", tr(lang, Key::Apply), ButtonKind::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.apply(cx))),
            )
    }
}

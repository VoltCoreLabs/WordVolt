use gpui_kit::*;
use wv_i18n::{Key, t};

use crate::ui_kit::{ButtonKind, Tip, block_group, button};
use crate::window::state::MainWindow;

/// Review tab: change tracking, accepting and rejecting changes.
pub fn groups(this: &MainWindow, cx: &mut Context<MainWindow>) -> Vec<AnyElement> {
    let lang = this.lang;
    let on = this.editor.read(cx).core.tracking();
    let tracking = block_group(
        t(lang, Key::GroupTracking),
        vec![
            button(
                "track-toggle",
                t(lang, Key::TrackChanges),
                if on {
                    ButtonKind::Primary
                } else {
                    ButtonKind::Default
                },
                cx,
            )
            .tip(t(lang, Key::TipTrackChanges))
            .on_click(cx.listener(|this, _, _, cx| {
                let author = crate::settings::user_name();
                this.editor.update(cx, |view, cx| {
                    let on = !view.core.tracking();
                    view.core.set_author(&author);
                    view.core.set_tracking(on);
                    cx.notify();
                });
                cx.notify();
            }))
            .into_any_element(),
        ],
        cx,
    );
    let changes = block_group(
        t(lang, Key::GroupChanges),
        vec![
            button("change-prev", "←", ButtonKind::Default, cx)
                .tip(t(lang, Key::TipPrevChange))
                .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| _ = c.goto_change(false))))
                .into_any_element(),
            button("change-next", "→", ButtonKind::Default, cx)
                .tip(t(lang, Key::TipNextChange))
                .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| _ = c.goto_change(true))))
                .into_any_element(),
            button(
                "change-accept",
                t(lang, Key::AcceptChange),
                ButtonKind::Default,
                cx,
            )
            .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.accept_changes_here())))
            .into_any_element(),
            button(
                "change-reject",
                t(lang, Key::RejectChange),
                ButtonKind::Default,
                cx,
            )
            .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.reject_changes_here())))
            .into_any_element(),
            button(
                "change-accept-all",
                t(lang, Key::AcceptAll),
                ButtonKind::Default,
                cx,
            )
            .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.accept_all_changes())))
            .into_any_element(),
            button(
                "change-reject-all",
                t(lang, Key::RejectAll),
                ButtonKind::Default,
                cx,
            )
            .on_click(cx.listener(|this, _, _, cx| this.edit(cx, |c| c.reject_all_changes())))
            .into_any_element(),
        ],
        cx,
    );
    vec![tracking.into_any_element(), changes.into_any_element()]
}

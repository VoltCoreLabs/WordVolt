use gpui_kit::*;

use crate::theme::{Theme, space};
use crate::ui_kit::{ButtonKind, block, button};

/// Top block with the command groups (prototype: Home tab).
pub fn ribbon(cx: &App) -> Div {
    let muted = cx.global::<Theme>().text_muted;
    let group = |name: &'static str, buttons: Vec<Stateful<Div>>| {
        block(cx)
            .p(space::sm())
            .flex()
            .flex_col()
            .gap(space::sm())
            .child(div().flex().gap(space::sm()).children(buttons))
            .child(div().text_xs().text_color(muted).child(name))
    };
    div()
        .flex()
        .gap(space::md())
        .child(group(
            "Font",
            vec![
                button("bold", "B", ButtonKind::Default, cx),
                button("italic", "I", ButtonKind::Default, cx),
                button("underline", "U", ButtonKind::Default, cx),
            ],
        ))
        .child(group(
            "Paragraph",
            vec![
                button("left", "Left", ButtonKind::Default, cx),
                button("center", "Center", ButtonKind::Default, cx),
                button("right", "Right", ButtonKind::Default, cx),
            ],
        ))
        .child(group(
            "File",
            vec![button("save", "Save", ButtonKind::Primary, cx)],
        ))
}

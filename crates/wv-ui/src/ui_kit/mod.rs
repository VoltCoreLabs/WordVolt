//! Reusable blocks and controls. All widgets are built from these.

mod block;
mod block_group;
mod button;
mod hover;
mod icon;
mod icon_button;
mod line_edit;
mod tab;
mod window_controls;

use gpui_kit::App;

pub use block::block;
pub use block_group::block_group;
pub use button::{ButtonKind, button};
pub use icon::{Icon, Weight, icon};
pub use icon_button::icon_button;
pub use line_edit::{LineEdit, Outcome};
pub use tab::tab;
pub use window_controls::{CONTROLS_WIDTH, window_controls};

/// Registers the global state widgets need (hover fades). Call once at startup.
pub fn init(cx: &mut App) {
    cx.set_global(hover::HoverMap::default());
}

//! Reusable blocks and controls. All widgets are built from these.

mod block;
mod block_group;
mod button;
pub mod color_math;
mod hover;
mod icon;
mod icon_button;
mod line_edit;
mod pop;
mod recent_colors;
mod tab;
mod tooltip;
mod window_controls;

use gpui_kit::App;

pub use block::block;
pub use block_group::block_group;
pub use button::{ButtonKind, button};
pub use icon::{Icon, Weight, icon};
pub use icon_button::icon_button;
pub use line_edit::{LineEdit, Outcome};
pub use pop::pop_in;
pub use recent_colors::RecentColors;
pub use tab::tab;
pub use tooltip::{Tip, layer as tooltip_layer};
pub use window_controls::{CONTROLS_WIDTH, window_controls};

/// Registers the global state widgets need (hover fades, recent colours). Call once at startup.
pub fn init(cx: &mut App) {
    cx.set_global(hover::HoverMap::default());
    cx.set_global(RecentColors::load());
    tooltip::init(cx);
}

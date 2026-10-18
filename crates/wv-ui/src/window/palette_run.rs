use gpui_kit::*;
use wv_core::{Align, ListKind, Script};

use super::palette_items::Action;
use super::state::MainWindow;

impl MainWindow {
    /// Runs a command chosen in the palette.
    pub(super) fn run_palette(
        &mut self,
        action: Action,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            Action::Menu(command) => self.run_command(command, window, cx),
            Action::Bold => self.edit(cx, |c| c.toggle_bold()),
            Action::Italic => self.edit(cx, |c| c.toggle_italic()),
            Action::Underline => self.edit(cx, |c| c.toggle_underline()),
            Action::Strike => self.edit(cx, |c| c.toggle_strike()),
            Action::Superscript => self.edit(cx, |c| c.toggle_script(Script::Super)),
            Action::Subscript => self.edit(cx, |c| c.toggle_script(Script::Sub)),
            Action::Code => self.edit(cx, |c| c.toggle_mono()),
            Action::ClearFormat => self.edit(cx, |c| c.clear_formatting()),
            Action::AlignLeft => self.edit(cx, |c| c.set_align(Align::Left)),
            Action::AlignCenter => self.edit(cx, |c| c.set_align(Align::Center)),
            Action::AlignRight => self.edit(cx, |c| c.set_align(Align::Right)),
            Action::AlignJustify => self.edit(cx, |c| c.set_align(Align::Justify)),
            Action::Bullets => self.edit(cx, |c| c.toggle_list(ListKind::Bullet)),
            Action::Numbered => self.edit(cx, |c| c.toggle_list(ListKind::Number)),
            Action::IndentMore => self.edit(cx, |c| c.change_indent(1)),
            Action::IndentLess => self.edit(cx, |c| c.change_indent(-1)),
            Action::Quote => self.edit(cx, |c| c.toggle_quote()),
            Action::Style(style) => self.edit(cx, move |c| c.set_named_style(style)),
            Action::PageBreak => self.edit(cx, |c| c.insert_page_break()),
            Action::BlankPage => self.edit(cx, |c| c.insert_blank_page()),
            Action::SectionBreak => self.edit(cx, |c| c.insert_section_break()),
            Action::Footnote => self.edit(cx, |c| c.insert_footnote()),
            Action::PageNumber => self.edit(cx, |c| c.toggle_page_number()),
            Action::Image => self.pick_and_insert_image(window, cx),
            Action::Table => self.edit(cx, |c| c.insert_table(3, 3)),
            Action::UpdateFields => self.editor.update(cx, |view, cx| view.update_fields(cx)),
            Action::TrackChanges => {
                let author = std::env::var("USERNAME").unwrap_or_default();
                self.editor.update(cx, |view, cx| {
                    let on = !view.core.tracking();
                    view.core.set_author(&author);
                    view.core.set_tracking(on);
                    cx.notify();
                });
            }
            Action::AcceptAll => self.edit(cx, |c| c.accept_all_changes()),
            Action::RejectAll => self.edit(cx, |c| c.reject_all_changes()),
            Action::NextChange => self.edit(cx, |c| _ = c.goto_change(true)),
            Action::PrevChange => self.edit(cx, |c| _ = c.goto_change(false)),
            Action::ZoomIn => self.zoom_by(super::ribbon::view::STEP, cx),
            Action::ZoomOut => self.zoom_by(1.0 / super::ribbon::view::STEP, cx),
            Action::ZoomReset => self.editor.update(cx, |view, cx| view.set_zoom(1.0, cx)),
            Action::ZoomFit => self.editor.update(cx, |view, cx| {
                let fit = view.fit_width_zoom();
                view.set_zoom(fit, cx);
            }),
            Action::Marks => self.editor.update(cx, |view, cx| view.toggle_marks(cx)),
            Action::Ruler => self.editor.update(cx, |view, cx| view.toggle_ruler(cx)),
            Action::MultiPage => self
                .editor
                .update(cx, |view, cx| view.toggle_multi_page(cx)),
            Action::NavPanel => {
                self.panel_open = !self.panel_open;
                self.comments_panel = false;
            }
            Action::StylesPanel => self.styles_open = !self.styles_open,
            Action::CommentsPanel => {
                self.comments_panel = !self.comments_panel;
                self.panel_open = false;
            }
            Action::Portrait => self.edit(cx, |c| c.set_page(c.page_here().with_landscape(false))),
            Action::Landscape => self.edit(cx, |c| c.set_page(c.page_here().with_landscape(true))),
            Action::Columns(n) => self.edit(cx, move |c| c.set_page(c.page_here().with_columns(n))),
        }
        self.editor.update(cx, |view, cx| view.request_focus(cx));
        cx.notify();
    }

    fn zoom_by(&mut self, factor: f32, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |view, cx| view.set_zoom(view.zoom * factor, cx));
    }
}

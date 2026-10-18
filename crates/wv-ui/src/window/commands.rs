use gpui_kit::*;

use super::menu_bar::Command;
use super::state::MainWindow;

impl MainWindow {
    /// Runs a title-bar menu command.
    pub(super) fn run_command(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match command {
            Command::New => self.new_document(window, cx),
            Command::Open => self.open_document(window, cx),
            Command::Save => self.save_document(window, cx),
            Command::SaveAs => self.save_document_as(window, cx),
            Command::ExportPdf => self.export_pdf(window, cx),
            Command::Print => self.print_document(window, cx),
            Command::Undo => self.edit(cx, |c| _ = c.undo()),
            Command::Redo => self.edit(cx, |c| _ = c.redo()),
            Command::About => {
                self.about_open = true;
                cx.notify();
            }
            Command::Find => self
                .find_bar
                .update(cx, |bar, cx| bar.open(false, window, cx)),
            Command::Replace => self
                .find_bar
                .update(cx, |bar, cx| bar.open(true, window, cx)),
        }
    }
}

use gpui_kit::*;
use wv_core::{Band, BandVariants, CharStyle};
use wv_i18n::{Key, Lang, t as tr};

use crate::ui_kit::{Icon, LineEdit, Outcome, Tip, icon_button, pop_in};

pub enum HeaderFooterEvent {
    Set(Band, Band, BandVariants),
}

/// Button with a popover for the header and footer text (`{page}` = page number).
pub struct HeaderFooterEditor {
    pub(super) lang: Lang,
    pub(super) open: bool,
    /// Bounds of the open panel (see `track_bounds`).
    pub(super) panel_bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    pub(super) epoch: u64,
    pub(super) current: (Band, Band, BandVariants),
    /// Working copy while the popover is open: main, first-page and even-page (header, footer).
    pub(super) sets: [(Band, Band); 3],
    /// (first page differs, even pages differ).
    pub(super) flags: (bool, bool),
    /// Which of `sets` the fields show.
    pub(super) which: usize,
    /// The (header, footer) of the shown set being edited.
    pub(super) work: (Band, Band),
    pub(super) header: LineEdit,
    pub(super) footer: LineEdit,
    pub(super) header_focus: FocusHandle,
    pub(super) footer_focus: FocusHandle,
}

impl EventEmitter<HeaderFooterEvent> for HeaderFooterEditor {}

impl HeaderFooterEditor {
    pub fn new(lang: Lang, cx: &mut Context<Self>) -> Self {
        Self {
            lang,
            open: false,
            panel_bounds: Default::default(),
            epoch: 0,
            current: Default::default(),
            sets: Default::default(),
            flags: (false, false),
            which: 0,
            work: Default::default(),
            header: LineEdit::default(),
            footer: LineEdit::default(),
            header_focus: cx.focus_handle(),
            footer_focus: cx.focus_handle(),
        }
    }

    /// Texts of the document (pre-fill when the popover opens; ignored while it is open).
    pub fn set_current(&mut self, header: &Band, footer: &Band, variants: &BandVariants) {
        if !self.open {
            self.current = (header.clone(), footer.clone(), variants.clone());
        }
    }

    /// Puts the working copy back into its set.
    fn store_fields(&mut self) {
        self.sets[self.which] = self.work.clone();
    }

    /// Shows set `which` in the fields.
    fn load_fields(&mut self) {
        self.work = self.sets[self.which].clone();
        self.header.set(&self.work.0.text(), false);
        self.footer.set(&self.work.1.text(), false);
    }

    /// The band being edited, its field and the range formatting applies to: the selection, or
    /// the whole text when nothing is selected.
    fn target(&mut self, in_header: bool) -> (&mut Band, usize, usize) {
        let (band, edit) = if in_header {
            (&mut self.work.0, &self.header)
        } else {
            (&mut self.work.1, &self.footer)
        };
        let (start, end) = edit
            .selection()
            .map_or((0, band.para.len()), |r| (r.start, r.end));
        (band, start, end)
    }

    /// Whether the formatting applies to everything that would change (the toggle's state).
    pub(super) fn all_chars(&self, in_header: bool, pred: impl Fn(&CharStyle) -> bool) -> bool {
        let (band, edit) = if in_header {
            (&self.work.0, &self.header)
        } else {
            (&self.work.1, &self.footer)
        };
        let (start, end) = edit
            .selection()
            .map_or((0, band.para.len()), |r| (r.start, r.end));
        band.all_chars(start, end, pred)
    }

    /// Changes the formatting of the selection (or of the whole text).
    pub(super) fn restyle(
        &mut self,
        in_header: bool,
        f: impl Fn(&mut CharStyle),
        cx: &mut Context<Self>,
    ) {
        let (band, start, end) = self.target(in_header);
        band.restyle(start, end, f);
        cx.notify();
    }

    /// Switches a character attribute on, or off when it is already on everywhere.
    pub(super) fn toggle_style(
        &mut self,
        in_header: bool,
        get: fn(&CharStyle) -> bool,
        set: fn(&mut CharStyle, bool),
        cx: &mut Context<Self>,
    ) {
        let on = self.all_chars(in_header, get);
        self.restyle(in_header, move |s| set(s, !on), cx);
    }

    pub(super) fn select(&mut self, which: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.store_fields();
        self.which = which;
        self.load_fields();
        window.focus(&self.header_focus, cx);
        cx.notify();
    }

    pub(super) fn toggle_flag(&mut self, first: bool, window: &mut Window, cx: &mut Context<Self>) {
        if first {
            self.flags.0 = !self.flags.0;
        } else {
            self.flags.1 = !self.flags.1;
        }
        let gone = (self.which == 1 && !self.flags.0) || (self.which == 2 && !self.flags.1);
        if gone {
            self.select(0, window, cx);
        }
        cx.notify();
    }

    fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = !self.open;
        if self.open {
            self.epoch += 1;
            let variants = &self.current.2;
            self.sets = [
                (self.current.0.clone(), self.current.1.clone()),
                variants.first.clone(),
                variants.even.clone(),
            ];
            self.flags = (variants.different_first, variants.odd_even);
            self.which = 0;
            self.load_fields();
            window.focus(&self.header_focus, cx);
        }
        cx.notify();
    }

    pub(super) fn apply(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        self.store_fields();
        let variants = BandVariants {
            different_first: self.flags.0,
            odd_even: self.flags.1,
            first: self.sets[1].clone(),
            even: self.sets[2].clone(),
        };
        let (header, footer) = self.sets[0].clone();
        self.current = (header.clone(), footer.clone(), variants.clone());
        cx.emit(HeaderFooterEvent::Set(header, footer, variants));
        cx.notify();
    }

    pub(super) fn switch_field(
        &mut self,
        in_header: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = if in_header {
            &self.footer_focus
        } else {
            &self.header_focus
        };
        window.focus(target, cx);
        cx.notify();
    }

    pub(super) fn on_key(&mut self, in_header: bool, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let (field, band) = if in_header {
            (&mut self.header, &mut self.work.0)
        } else {
            (&mut self.footer, &mut self.work.1)
        };
        match field.key(ev, cx) {
            Outcome::Submit => self.apply(cx),
            Outcome::Cancel => {
                self.open = false;
                cx.notify();
            }
            Outcome::Changed => {
                band.set_text(field.text());
                cx.notify();
            }
            Outcome::Nothing => cx.notify(),
        }
        cx.stop_propagation();
    }
}

impl Render for HeaderFooterEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.lang;
        let button = icon_button("band-btn", Icon::PageHeader, self.open, cx)
            .tip(tr(lang, Key::TipHeaderFooter))
            .on_click(cx.listener(|this, _, window, cx| this.toggle(window, cx)));
        let pop_key = format!("band-pop-{}", self.epoch);
        let menu = self.open.then(|| {
            deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .offset(point(px(0.), px(36.)))
                    .child(pop_in(self.panel(window, cx), pop_key)),
            )
            .with_priority(10)
        });
        div()
            .relative()
            .on_mouse_down_out(cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                if this.open && !this.panel_bounds.get().contains(&ev.position) {
                    this.open = false;
                    cx.notify();
                }
            }))
            .child(button)
            .children(menu)
    }
}

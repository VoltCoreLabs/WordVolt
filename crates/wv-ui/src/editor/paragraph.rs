use std::ops::Range;

use gpui_kit::*;
use wv_core::{Align, CharStyle, Paragraph};

use super::align::{caret_point, selection_rects};
use super::caret::caret_alpha;
use super::view::EditorView;
use crate::theme::{Theme, ui_font};

const LINE_HEIGHT: f32 = 24.;

/// One paragraph: styled text, selection highlight (as run background) and the caret.
pub fn paragraph_el(
    view: &EditorView,
    ix: usize,
    focused: bool,
    cx: &mut Context<EditorView>,
) -> Stateful<Div> {
    let theme = cx.global::<Theme>();
    let para = &view.core.doc.paragraphs[ix];
    let (text, runs) = text_and_runs(para, theme);
    let styled = StyledText::new(text).with_runs(runs);
    let layout = styled.layout().clone();
    view.layouts.borrow_mut().push(layout.clone());

    let head = view.core.sel.head;
    let align = para.style.align;
    // Drawn by us (GPUI's run background ignores paragraph alignment), under the text.
    let selection = selection_in(view, ix).map(|range| {
        let (layout, color) = (layout.clone(), theme.accent_subtle);
        canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                for rect in selection_rects(&layout, range.clone(), align) {
                    window.paint_quad(fill(rect, color));
                }
            },
        )
        .absolute()
        .size_full()
    });
    let caret = (focused && head.para == ix).then(|| {
        let color = theme.text;
        let (epoch, scroll, reveal) = (
            view.caret_epoch.clone(),
            view.scroll.clone(),
            view.reveal_caret.clone(),
        );
        canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                let Some(at) = caret_point(&layout, head.offset, align) else {
                    return;
                };
                let height = layout.line_height();
                let alpha = caret_alpha(epoch.get().elapsed().as_secs_f32());
                window.paint_quad(fill(
                    Bounds::new(at, size(px(1.5), height)),
                    Rgba { a: alpha, ..color },
                ));
                if reveal.replace(false) {
                    scroll.reveal(at.y, height);
                }
                window.request_animation_frame();
            },
        )
        .absolute()
        .size_full()
    });

    let base = div()
        .id(("para", ix))
        .relative()
        .w_full()
        .min_h(px(LINE_HEIGHT))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, ev, window, cx| this.on_mouse_down(ix, ev, window, cx)),
        );
    let aligned = match align {
        Align::Left | Align::Justify => base,
        Align::Center => base.text_center(),
        Align::Right => base.text_right(),
    };
    aligned.children(selection).child(styled).children(caret)
}

/// Selected byte range inside paragraph `ix`, if any.
fn selection_in(view: &EditorView, ix: usize) -> Option<Range<usize>> {
    let (s, e) = view.core.sel.range();
    if s == e || ix < s.para || ix > e.para {
        return None;
    }
    let start = if ix == s.para { s.offset } else { 0 };
    let end = if ix == e.para {
        e.offset
    } else {
        view.core.doc.paragraphs[ix].len()
    };
    (start < end).then_some(start..end)
}

/// Text plus one run per style span. An empty paragraph shows one space so it keeps its height.
fn text_and_runs(para: &Paragraph, theme: &Theme) -> (SharedString, Vec<TextRun>) {
    if para.is_empty() {
        return (" ".into(), vec![make_run(1, &CharStyle::default(), theme)]);
    }
    let runs = para
        .runs
        .iter()
        .map(|run| make_run(run.text.len(), &run.style, theme))
        .collect();
    (para.text().into(), runs)
}

fn make_run(len: usize, style: &CharStyle, theme: &Theme) -> TextRun {
    let mut font = ui_font();
    if style.bold {
        font.weight = FontWeight::BOLD;
    }
    if style.italic {
        font.style = FontStyle::Italic;
    }
    TextRun {
        len,
        font,
        color: theme.text.into(),
        background_color: None,
        underline: style.underline.then(|| UnderlineStyle {
            thickness: px(1.),
            color: None,
            wavy: false,
        }),
        strikethrough: None,
    }
}

use std::ops::Range;

use gpui_kit::*;
use wv_core::{Align, CharStyle, Paragraph};

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
    let (text, runs) = text_and_runs(para, selection_in(view, ix), theme);
    let styled = StyledText::new(text).with_runs(runs);
    let layout = styled.layout().clone();
    view.layouts.borrow_mut().push(layout.clone());

    let head = view.core.sel.head;
    let caret = (focused && view.caret_visible && head.para == ix).then(|| {
        let color = theme.text;
        let offset = head.offset;
        let layout = layout.clone();
        canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                if let Some(at) = layout.position_for_index(offset) {
                    let caret = Bounds::new(at, size(px(1.5), layout.line_height()));
                    window.paint_quad(fill(caret, color));
                }
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
        )
        .on_mouse_move(cx.listener(move |this, ev, _, cx| this.on_mouse_drag(ix, ev, cx)));
    let aligned = match para.style.align {
        Align::Left | Align::Justify => base,
        Align::Center => base.text_center(),
        Align::Right => base.text_right(),
    };
    aligned.child(styled).children(caret)
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

/// Text plus runs (split at the selection edges). An empty paragraph shows one space so it keeps its height.
fn text_and_runs(
    para: &Paragraph,
    sel: Option<Range<usize>>,
    theme: &Theme,
) -> (SharedString, Vec<TextRun>) {
    if para.is_empty() {
        return (
            " ".into(),
            vec![make_run(1, &CharStyle::default(), false, theme)],
        );
    }
    let mut runs = Vec::new();
    let mut pos = 0;
    for run in &para.runs {
        let end = pos + run.text.len();
        let mut cuts = vec![pos];
        if let Some(sel) = &sel {
            cuts.extend(
                [sel.start, sel.end]
                    .into_iter()
                    .filter(|b| *b > pos && *b < end),
            );
        }
        cuts.sort_unstable();
        cuts.push(end);
        for w in cuts.windows(2) {
            let selected = sel
                .as_ref()
                .is_some_and(|s| w[0] >= s.start && w[1] <= s.end);
            runs.push(make_run(w[1] - w[0], &run.style, selected, theme));
        }
        pos = end;
    }
    (para.text().into(), runs)
}

fn make_run(len: usize, style: &CharStyle, selected: bool, theme: &Theme) -> TextRun {
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
        background_color: selected.then(|| theme.accent_subtle.into()),
        underline: style.underline.then(|| UnderlineStyle {
            thickness: px(1.),
            color: None,
            wavy: false,
        }),
        strikethrough: None,
    }
}

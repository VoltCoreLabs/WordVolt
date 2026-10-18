use crate::Measurer;
use wv_core::Paragraph;

/// Narrowest stretch of a line worth putting text in; narrower gaps beside a picture are skipped.
const MIN_LINE: f32 = 48.0;

/// Something the text goes around: a picture's horizontal span `x0..x1` and vertical span
/// `top..bottom`, from the top-left corner of the paragraph's text box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obstacle {
    pub top: f32,
    pub bottom: f32,
    pub x0: f32,
    pub x1: f32,
}

/// Where one line of a paragraph goes: how far in it starts, how wide it may be and how much
/// empty space comes before it (when no gap beside a picture is wide enough).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LineBox {
    pub inset: f32,
    pub width: f32,
    pub gap: f32,
}

/// The line boxes of a paragraph, worked out line by line as the text is broken: the place of a
/// line depends on the heights of the lines above it.
pub(super) struct Boxes<'a> {
    list: Vec<LineBox>,
    /// Top of the next line to be placed (from the top of the paragraph).
    y: f32,
    /// Estimated height of a line (the real one is only known once it is built).
    height: f32,
    width: f32,
    first: f32,
    obstacles: &'a [Obstacle],
}

impl<'a> Boxes<'a> {
    pub fn new(
        p: &Paragraph,
        line_spacing: f32,
        width: f32,
        first: f32,
        obstacles: &'a [Obstacle],
        m: &dyn Measurer,
    ) -> Self {
        let mut style = p.runs.first().map(|r| r.style.clone()).unwrap_or_default();
        p.style.named.apply(&mut style);
        Self {
            list: Vec::new(),
            y: 0.0,
            height: (m.ascent(&style) + m.descent(&style)) * line_spacing,
            width,
            first,
            obstacles,
        }
    }

    /// The box of line `row` (earlier ones are worked out first).
    pub fn get(&mut self, row: usize) -> LineBox {
        while self.list.len() <= row {
            let lead = if self.list.is_empty() {
                self.first
            } else {
                0.0
            };
            let b = place(self.y, self.height, self.width, lead, self.obstacles);
            self.y += b.gap + self.height;
            self.list.push(b);
        }
        self.list[row]
    }

    pub fn width(&mut self, row: usize) -> f32 {
        self.get(row).width
    }

    pub fn inset(&mut self, row: usize) -> f32 {
        self.get(row).inset
    }

    /// The boxes of the first `rows` lines.
    pub fn finish(mut self, rows: usize) -> Vec<LineBox> {
        if rows > 0 {
            self.get(rows - 1);
        }
        self.list.truncate(rows);
        self.list
    }
}

/// The box for a line of `height` whose top is `y` in a text box `width` wide: the widest stretch
/// the obstacles leave free, or, if none is wide enough, the first line below them.
fn place(mut y: f32, height: f32, width: f32, lead: f32, obstacles: &[Obstacle]) -> LineBox {
    let mut gap = 0.0;
    loop {
        let mut hits: Vec<&Obstacle> = obstacles
            .iter()
            .filter(|o| o.top < y + height && o.bottom > y)
            .collect();
        if hits.is_empty() {
            return LineBox {
                inset: lead,
                width: width - lead,
                gap,
            };
        }
        hits.sort_by(|a, b| a.x0.total_cmp(&b.x0));
        let mut free: Vec<(f32, f32)> = Vec::new();
        let mut cursor = 0.0_f32;
        for o in &hits {
            if o.x0 > cursor {
                free.push((cursor, o.x0));
            }
            cursor = cursor.max(o.x1);
        }
        if cursor < width {
            free.push((cursor, width));
        }
        let best = free
            .iter()
            .map(|&(a, b)| {
                // The first line also keeps its indent when it starts at the left edge.
                let start = if a <= 0.0 { a + lead } else { a };
                (start, b - start)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((inset, w)) = best
            && w >= MIN_LINE
        {
            return LineBox {
                inset,
                width: w,
                gap,
            };
        }
        let bottom = hits.iter().map(|o| o.bottom).fold(y, f32::max);
        gap += bottom - y;
        y = bottom;
    }
}
